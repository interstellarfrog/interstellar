#![feature(pattern)]
#![feature(stmt_expr_attributes)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use std::{
    env::{self},
    ffi::OsString,
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use bootloader::{BiosBoot, BootConfig, UefiBoot};
use conquer_once::spin::OnceCell;
use regex::Regex;

pub struct Logger {
    pub verbose: bool,
}

impl Logger {
    /// Create a new [Logger]
    pub fn new(verbose: bool) -> Logger {
        Logger { verbose }
    }
    /// Log to stdout
    pub fn log(&self, message: &str) {
        println!("{}", message);
    }
    /// Log to stdout if verbose flag set
    pub fn verbose_log(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }
    }
}

/// Static logger for use by test runner
pub static LOGGER: OnceCell<Logger> = OnceCell::uninit();

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        println!("{}...", core::any::type_name::<T>());
        self();
        println!("[ok]");
    }
}

/// Finds, compiles, and runs all the tests for the kernel in QEMU
pub fn test_runner(other_tests: &[&dyn Testable]) {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut custom_tests: Option<Vec<String>> = None;

    let mut verbose = false;
    let mut next_arg_test = false;
    let mut uefi = false;
    let mut bios = false;

    for arg in args.clone() {
        if next_arg_test {
            println!("next arg is a test found");

            if let Some(ref mut custom_tests) = custom_tests {
                custom_tests.append(&mut vec![arg]);
            } else {
                custom_tests = Some(vec![arg]);
            }
            next_arg_test = false;
        } else if arg == *"--test" {
            println!("found test arg");
            next_arg_test = true;
        } else if arg == *"--verbose" {
            verbose = true;
        } else if arg == *"--uefi" {
            uefi = true;
        } else if arg == *"--bios" {
            bios = true;
        }
    }

    if !bios && !uefi {
        bios = true;
        uefi = true;
    }

    LOGGER.init_once(|| Logger::new(verbose));

    LOGGER.get().unwrap().verbose_log("Using Verbose Mode");

    LOGGER
        .get()
        .unwrap()
        .verbose_log(&format!("Args passed to test runner: {:#?}", args));

    LOGGER.get().unwrap().verbose_log(&format!(
        "Custom test args passed: {:#?}",
        custom_tests.clone()
    ));

    let target_dir: PathBuf = env!("OUT_DIR").into();

    LOGGER
        .get()
        .unwrap()
        .verbose_log(&format!("Target Dir: {}", target_dir.display()));

    let tests_dir: PathBuf =
        format!("{}/x86_64-unknown-none/debug/deps", target_dir.display()).into();

    LOGGER
        .get()
        .unwrap()
        .verbose_log(&format!("Tests Dir: {}", tests_dir.display()));

    LOGGER.get().unwrap().verbose_log("Cleaning tests");

    clean(); // Clean tests

    if uefi {
        println!("Compiling test/s...");
        compile_tests(&target_dir, true);
        let os_tests = find_kernel_tests(&tests_dir, custom_tests.clone()).unwrap();
        let total_os_tests = os_tests.len();
        let mut succeeded_os_tests = 0;
        let mut failed_os_tests = 0;
        let mut failed_os_tests_names: Vec<String> = vec![];

        println!("Using UEFI");
        println!("Running {} OS test/s...", total_os_tests);

        for test in &os_tests {
            let disk = build_test_disk(&target_dir, &tests_dir.join(test), true);
            match run_in_qemu(&disk, true) {
                Ok(_) => succeeded_os_tests += 1,
                Err(_) => {
                    failed_os_tests += 1;
                    failed_os_tests_names.append(&mut vec![String::from(test.to_str().unwrap())]);
                
                },
            }
        }

        println!(
            "UEFI test results: {} total, {} succeeded, {} failed",
            total_os_tests, succeeded_os_tests, failed_os_tests
        );

        LOGGER.get().unwrap().verbose_log("Cleaning test/s");

        clean(); // Clean tests

        if failed_os_tests > 0 {
            for name in failed_os_tests_names {
                println!("{} Failed", name);
            }
            panic!("Error: Some Tests Failed!");
        }
    }

    if bios {
        println!("Compiling test/s...");
        compile_tests(&target_dir, false);
        let os_tests = find_kernel_tests(&tests_dir, custom_tests).unwrap();

        let total_os_tests = os_tests.len();
        let mut succeeded_os_tests = 0;
        let mut failed_os_tests = 0;

        println!("Using BIOS");
        println!("Running {} OS test/s...", total_os_tests);

        for test in &os_tests {
            let disk = build_test_disk(&target_dir, &tests_dir.join(test), false);
            match run_in_qemu(&disk, false) {
                Ok(_) => succeeded_os_tests += 1,
                Err(_) => failed_os_tests += 1,
            }
        }

        println!(
            "BIOS test results: {} total, {} succeeded, {} failed",
            total_os_tests, succeeded_os_tests, failed_os_tests
        );

        LOGGER.get().unwrap().verbose_log("Cleaning test/s");

        clean(); // Clean tests

        if failed_os_tests > 0 {
            panic!("Some test/s failed, exiting");
        }
    }

    println!("Running {} other test/s", other_tests.len());
    for test in other_tests {
        test.run();
    }
}

/// Compiles all of the tests in a separate directory with the test feature enabled
fn compile_tests(target_dir: &Path, uefi: bool) {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("test")
        .arg("--package")
        .arg("interstellar_os")
        .arg("--target")
        .arg("x86_64-unknown-none")
        .arg("--test=*")
        .arg("--target-dir")
        .arg(target_dir)
        .arg("--no-default-features")
        .arg("--features=test")
        .arg("--no-run")
        .arg("--quiet");

    if uefi {
        cmd.arg("--features").arg("UEFI");
    }

    let status = cmd.status().expect("Failed to compile tests.");
    assert!(status.success());
}

/// Returns all files in a directory as a Result<Vec<OsString>, io::Error>
fn get_files(dir: &Path) -> Result<Vec<OsString>, io::Error> {
    let files = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name())
        .collect();
    Ok(files)
}

///finds the kernel tests by using regex
fn find_kernel_tests(
    tests_dir: &PathBuf,
    test_args: Option<Vec<String>>,
) -> Result<Vec<OsString>, io::Error> {
    let files = get_files(Path::new(tests_dir))?;
    Ok(files
        .iter()
        .cloned()
        .filter(|file| {
            let test_pattern = Regex::new(r"\-[0-9a-z]{16}$").unwrap(); // exactly 16 lowercase alphanumeric characters on the end for some reason

            if test_args.is_some() {
                let f = file.to_str();

                for arg in test_args.clone().unwrap() {
                    let found_test = f
                        .filter(|name| name.starts_with(&arg.clone()))
                        .map(|name| test_pattern.is_match(name))
                        .unwrap_or(false);
                    if found_test {
                        return found_test;
                    }
                }

                false
            } else {
                file.to_str()
                    .filter(|name| !name.starts_with("interstellar_os"))
                    .map(|name| test_pattern.is_match(name))
                    .unwrap_or(false)
            }
        })
        .collect())
}

/// Makes the kernel tests into disk images
fn build_test_disk(disks_path: &Path, test_path: &Path, uefi: bool) -> PathBuf {
    let mut boot_config = BootConfig::default();
    boot_config.serial_logging = false;
    boot_config.frame_buffer_logging = false;

    let out = disks_path.join(test_path).with_extension("img");

    if uefi {
        UefiBoot::new(test_path)
            .set_boot_config(&boot_config)
            .create_disk_image(&out)
            .expect("Failed to create UEFI disk image");
        out
    } else {
        BiosBoot::new(test_path)
            .set_boot_config(&boot_config)
            .create_disk_image(&out)
            .expect("Failed to create BIOS disk image");
        out
    }
}

/// runs the tests in QEMU
fn run_in_qemu(disk_path: &Path, uefi: bool) -> Result<(), ()> {
    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04")
        .arg("-display")
        .arg("none")
        .arg("-serial")
        .arg("stdio") // Redirect Serial To STDIO
        .arg("--no-reboot")
        .arg("-drive")
        .arg(format!("format=raw,file={}", disk_path.display()))
        .arg("-boot")
        .arg("order=c")
        .arg("-cpu")
        .arg("max") // Enables all features supported by the accelerator in the current host; Needed for RDSEED
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped());

    if uefi {
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    }

    let mut child = cmd.spawn().expect("Failed to spawn QEMU process.");

    let stdout = child.stdout.take().expect("Failed to get child stdout.");
    let stderr = child.stderr.take().expect("Failed to get child stderr.");

    let stdout_pipe = std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            println!("{}", line.unwrap());
        }
    });

    let stderr_pipe = std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stderr);
        for line in reader.lines() {
            println!("{}", line.unwrap());
        }
    });

    stdout_pipe.join().expect("Failed to join stdout thread.");
    stderr_pipe.join().expect("Failed to join stderr thread.");

    let status = child.wait().expect("Failed to wait for child process.");
    if status.code() == Some(33) {
        Ok(())
    } else {
        Err(())
    }
}

/// Cleans the tests directory
///
/// This is important because rebuilding for other features produces new binaries  
pub fn clean() {
    let target_dir: PathBuf = env!("OUT_DIR").into();
    let tests_dir: PathBuf =
        format!("{}/x86_64-unknown-none/debug/deps", target_dir.display()).into();

    let mut cmd = Command::new("cargo");
    cmd.arg("clean")
        .arg("--target-dir")
        .arg(format!("{}", tests_dir.display()))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn cargo clean process.");

    let stdout = child.stdout.take().expect("Failed to get child stdout.");
    let stderr = child.stderr.take().expect("Failed to get child stderr.");

    let stdout_pipe = std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            println!("{}", line.unwrap());
        }
    });

    let stderr_pipe = std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stderr);
        for line in reader.lines() {
            println!("{}", line.unwrap());
        }
    });

    stdout_pipe.join().expect("Failed to join stdout thread.");
    stderr_pipe.join().expect("Failed to join stderr thread.");

    let _ = child.wait().expect("Failed to wait for child process.");
}
