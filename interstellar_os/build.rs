//Copyright (C) <2023>  <interstellarfrog>
//
//This program is free software: you can redistribute it and/or modify
//it under the terms of the GNU General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//(at your option) any later version.
//
//This program is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.
//
//You should have received a copy of the GNU General Public License
//along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::env;

fn main() {
    let cwd_path = env::current_dir().expect("can't get current directory");
    let cwd = cwd_path.display();
    println!("cargo:rerun-if-changed={cwd}/build.rs");
    println!("cargo:rerun-if-changed={cwd}/Cargo.toml");
    println!("cargo:rerun-if-changed={cwd}/Cargo.lock");
    println!("cargo:rerun-if-changed={cwd}/src");
    println!("cargo:rerun-if-changed={cwd}/.cargo");
    println!("cargo:rerun-if-changed={cwd}/tests");
    println!("cargo:rerun-if-changed={cwd}/rust-toolchain");
}
