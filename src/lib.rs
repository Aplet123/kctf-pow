//! A library to solve, check, and generate proof-of-work challenges using [kCTF](https://google.github.io/kctf/)'s scheme.
//! ```rust
//! use kctf_pow::KctfPow;
//! 
//! fn main() {
//!     let pow = KctfPow::new();
//!     // decoding then solving a challenge
//!     let chall = pow.decode_challenge("s.AAAAMg==.H+fPiuL32DPbfN97cpd0nA==").unwrap();
//!     println!("{}", chall.solve());
//!     // decoding then checking a challenge
//!     let chall = pow.decode_challenge("s.AAAAMg==.NDtqORW1uZlIgzszbdMGZA==").unwrap();
//!     let sol = "s.NUH3arymnKB+ysUGdv+67ypDamn4wOKCPORB2ivWE1Yhinam2v4S6q4nAoC5LP97LScdVoq+NuFVF++Win5mNRYZS6bJAs8fk0h8XgvfcC/7JfmFISqeCIo/CIUgIucVAM+eGDjqitRULGXqIOyviJoJjW8DMouMRuJM/3eg/z18kutQHkX0N3sqPeF7Nzkk8S3Bs6aiHUORM30syUKYug==";
//!     assert_eq!(chall.check(sol), Ok(true));
//!     assert_eq!(chall.check("s.asdf"), Ok(false));
//!     // generating a random challenge of difficulty 50
//!     let chall = pow.generate_challenge(50);
//!     println!("{}", chall);
//! }
//! ```

use rand::prelude::*;
use rug::integer::Order;
use rug::ops::Pow;
use rug::Integer;
use std::convert::TryInto;
use std::fmt;

const VERSION: &'static str = "s";

/// A proof-of-work system for kCTF.
///
/// All proof-of-work related methods are on instances of [`KctfPow`] in order to initialize and reuse related constants.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KctfPow {
    modulus: Integer,
    exponent: Integer,
}

/// A proof-of-work challenge.
///
/// Contains a reference to the [`KctfPow`] that created the challenge. If you want to serialize it to a string, use the [`Display`](std::fmt::Display) implementation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Challenge<'a> {
    difficulty: u32,
    val: Integer,
    pow: &'a KctfPow,
}

impl KctfPow {
    /// Create a new instance and initialize necessary constants.
    pub fn new() -> Self {
        let modulus: Integer = Integer::from(2).pow(1279) - 1;
        let exponent = (modulus.clone() + 1) / 4;
        Self { modulus, exponent }
    }

    /// Decodes a challenge from a string and returns it.
    ///
    /// For optimization purposes, the difficulty of the challenge must be able to fit in a [`u32`].
    /// This shouldn't be an issue, since difficulties that can't fit into a [`u32`] will probably take too long anyways.
    pub fn decode_challenge(&self, chall_string: &str) -> Result<Challenge, &'static str> {
        let mut parts = chall_string.split('.');
        if parts.next() != Some(VERSION) {
            return Err("Incorrect version");
        }
        let data: Vec<_> = parts.collect();
        if data.len() != 2 {
            return Err("Incorrect number of parts");
        }
        let decoded_data: Vec<_> = data
            .into_iter()
            .map(|x| base64::decode(x).map_err(|_| "Parts aren't valid base64"))
            .collect::<Result<_, _>>()?;
        let difficulty_bytes = &decoded_data[0];
        let difficulty: u32;
        if difficulty_bytes.len() > 4 {
            let (first, last) = difficulty_bytes.split_at(difficulty_bytes.len() - 4);
            // if difficulty is 0-padded to longer than 4 bytes it should still work
            if first.iter().any(|&x| x != 0) {
                return Err("Difficulty is too large");
            }
            difficulty = u32::from_be_bytes(last.try_into().unwrap())
        } else {
            let mut difficulty_array = [0; 4];
            difficulty_array[4 - difficulty_bytes.len()..].copy_from_slice(difficulty_bytes);
            difficulty = u32::from_be_bytes(difficulty_array);
        }
        Ok(Challenge {
            pow: self,
            val: Integer::from_digits(&decoded_data[1], Order::Msf),
            difficulty,
        })
    }

    /// Generates a random challenge given a difficulty.
    pub fn generate_challenge(&self, difficulty: u32) -> Challenge {
        let mut bytes: [u8; 16] = [0; 16];
        thread_rng().fill(&mut bytes[..]);
        Challenge {
            pow: self,
            val: Integer::from_digits(&bytes, Order::Msf),
            difficulty,
        }
    }
}

impl Default for KctfPow {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Challenge<'a> {
    /// Solves a challenge and returns the solution.
    pub fn solve(mut self) -> String {
        for _ in 0..self.difficulty {
            // guaranteed to succeed so ignore the result
            let _ = self.val.pow_mod_mut(&self.pow.exponent, &self.pow.modulus);
            self.val ^= 1;
        }
        format!(
            "{}.{}",
            VERSION,
            base64::encode(self.val.to_digits(Order::Msf))
        )
    }

    /// Checks a solution to see if it satisfies the challenge.
    pub fn check(&self, sol: &str) -> Result<bool, &'static str> {
        let mut parts = sol.split('.');
        if parts.next() != Some(VERSION) {
            return Err("Incorrect version");
        }
        let data = match parts.next() {
            Some(x) => x,
            None => return Err("Incorrect number of parts"),
        };
        if let Some(_) = parts.next() {
            return Err("Incorrect number of parts");
        }
        let decoded_data = base64::decode(data).map_err(|_| "Parts aren't valid base64")?;
        let mut sol_val = Integer::from_digits(&decoded_data, Order::Msf);
        for _ in 0..self.difficulty {
            sol_val ^= 1;
            // guaranteed to succeed so ignore the result
            let _ = sol_val.pow_mod_mut(&2.into(), &self.pow.modulus);
        }
        Ok(self.val == sol_val || Integer::from(&self.pow.modulus - &self.val) == sol_val)
    }
}

impl<'a> fmt::Display for Challenge<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}.{}.{}",
            VERSION,
            base64::encode(&self.difficulty.to_be_bytes()),
            base64::encode(&self.val.to_digits(Order::Msf))
        )
    }
}
