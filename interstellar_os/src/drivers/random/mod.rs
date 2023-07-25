//This file contains code for interstellar OS - https://github.com/interstellarfrog/interstellar
//Copyright (C) 2023  contributors of the interstellar OS project
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

use alloc::format;

use crate::other::log::LOGGER;
use core::arch::asm;

const LETTER_LIST: [&str; 26] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z",
];

/// Random number generator struct
pub struct RandomNumberGenerator {
    a: u128,
    c: u128,
    m: u128,
    x: Option<u128>,
}

impl Default for RandomNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomNumberGenerator {
    /// Returns a new [RandomNumberGenerator]
    pub fn new() -> RandomNumberGenerator {
        let a: u128 = 48271;
        let c: u128 = 0;
        let m: u128 = 2147483647;
        let seed: Option<u128> = rdseed();
        let mut x: Option<u128> = None;

        if let Some(s) = seed {
            x = Some((a * s + c) % m); // Calculate first random number
        }

        RandomNumberGenerator { a, c, m, x }
    }

    /// Generate a random [u128] number
    ///
    /// Between `0` and `2,147,483,646`
    ///
    /// # Example
    /// ```
    /// let mut rng = RandomNumberGenerator::default();
    /// for _ in 0..10 {
    ///     let number = rng.generate_number(Some(0), Some(10));
    ///     if number.is_some() {
    ///         println!("number: {}", number);
    ///     }
    /// }
    pub fn generate_number(&mut self, min: Option<u128>, max: Option<u128>) -> Option<u128> {
        let min = min.unwrap_or(0);
        let max = max.unwrap_or(self.m - 1);

        if max > (self.m - 1) {
            LOGGER.get().unwrap().lock().warn(&format!(
                "Max random number range: {} Bigger than limit: {}",
                max,
                self.m - 1
            ))
        }

        if min > max {
            LOGGER
                .get()
                .unwrap()
                .lock()
                .warn(&format!("Min range: {} is Bigger than max: {}", min, max))
        }

        if self.x.is_none() {
            LOGGER
                .get()
                .unwrap()
                .lock()
                .warn("Last generated seed not available in random number generator");
            let temp_x = rdseed();

            if let Some(temp_x) = temp_x {
                self.x = Some((self.a * temp_x + self.c) % self.m);
            } else {
                LOGGER
                    .get()
                    .unwrap()
                    .lock()
                    .warn("Failed to get another seed for x value");
                return None;
            }
        }

        self.x = Some((self.a * self.x.unwrap() + self.c) % self.m); // Calculate random number

        self.x.map(|x| min + ((x * (max - min + 1)) / self.m)) // Scale to range and return
    }

    /// Generate a random letter
    ///
    /// Between `a` and `z`
    ///
    /// Always lowercase
    ///
    /// # Example
    /// ```
    /// let mut rng = RandomNumberGenerator::default();
    /// for _ in 0..10 {
    ///     let letter = rng.generate_letter();
    ///     if letter.is_some() {
    ///         println!("letter: {}", letter);
    ///     }
    /// }
    pub fn generate_letter(&mut self) -> Option<&str> {
        let number = self.generate_number(Some(0), Some(25));

        if let Some(number) = number {
            Some(LETTER_LIST[number as usize])
        } else {
            LOGGER
                .get()
                .unwrap()
                .lock()
                .warn("Random number generator gave a 'None' value to the letter generator");
            None
        }
    }
}

/// Generates a random seed for the random number generator
fn rdseed() -> Option<u128> {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Generating Random Seed", file!(), line!());
    if !check_random_support() {
        // Maybe change this to just check once and save to variable instead of calling again
        LOGGER.get().unwrap().lock().warn(
            "RDSEED instruction is not supported for this CPU random numbers cannot be generated",
        );
        return None;
    }
    let mut result: u64;
    unsafe {
        asm!("rdseed {0:r}", out(reg) result);
    }
    Some(result as u128)
}

/// Checks CPUID for if the CPU supports RDSEED instruction
fn check_random_support() -> bool {
    LOGGER.get().unwrap().lock().trace(
        "Checking if CPU supports RDSEED instruction",
        file!(),
        line!(),
    );
    let cpuid = raw_cpuid::CpuId::new();

    match cpuid.get_feature_info() {
        Some(feat_info) => {
            feat_info.has_rdrand() // Unsure if this also means it supports RDSEED but no other options for it
        }
        None => false,
    }
}
