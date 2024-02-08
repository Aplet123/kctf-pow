//! A library to solve, check, and generate proof-of-work challenges using [kCTF](https://google.github.io/kctf/)'s scheme.
//!
//! ```rust
//! use kctf_pow::KctfPow;
//!
//! let pow = KctfPow::new();
//! // decoding then solving a challenge
//! let chall = pow.decode_challenge("s.AAAAMg==.H+fPiuL32DPbfN97cpd0nA==").unwrap();
//! println!("{}", chall.solve());
//! // decoding then checking a challenge
//! let chall = pow.decode_challenge("s.AAAAMg==.NDtqORW1uZlIgzszbdMGZA==").unwrap();
//! let sol = "s.NUH3arymnKB+ysUGdv+67ypDamn4wOKCPORB2ivWE1Yhinam2v4S6q4nAoC5LP97LScdVoq+NuFVF++Win5mNRYZS6bJAs8fk0h8XgvfcC/7JfmFISqeCIo/CIUgIucVAM+eGDjqitRULGXqIOyviJoJjW8DMouMRuJM/3eg/z18kutQHkX0N3sqPeF7Nzkk8S3Bs6aiHUORM30syUKYug==";
//! assert_eq!(chall.check(sol), Ok(true));
//! assert_eq!(chall.check("s.asdf"), Ok(false));
//! // generating a random challenge of difficulty 50
//! let chall = pow.generate_challenge(50);
//! println!("{}", chall);
//! ```

use base64::prelude::*;
use rand::prelude::*;
use rug::integer::Order;
use rug::ops::Pow;
use rug::Integer;
use std::convert::TryInto;
use std::fmt;

const VERSION: &str = "s";

/// A proof-of-work system for kCTF.
///
/// All proof-of-work related methods are on instances of [`KctfPow`] in order to initialize and reuse related constants.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KctfPow {
    /// The modulus of the proof-of-work. kCTF uses `2**1279 - 1`.
    pub modulus: Integer,
    /// The exponent of the proof-of-work. kCTF uses `(modulus + 1) / 4`.
    pub exponent: Integer,
}

/// The parameters for a proof-of-work challenge.
///
/// This contains most of the logic, however [`KctfPow`] and [`Challenge`] should be used instead as they provide a nicer API.
/// If you want to serialize it to a string, use the [`Display`](std::fmt::Display) implementation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChallengeParams {
    /// The difficulty of the challenge.
    pub difficulty: u32,
    /// The starting value of the challenge.
    pub val: Integer,
}

/// A proof-of-work challenge.
///
/// Contains a reference to the [`KctfPow`] that created the challenge. If you want to serialize it to a string, use the [`Display`](std::fmt::Display) implementation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Challenge<'a> {
    /// The parameters of the challenge.
    pub params: ChallengeParams,
    /// The proof-of-work system with associated constants pre-initialized.
    pub pow: &'a KctfPow,
}

impl ChallengeParams {
    /// Decodes a challenge from a string and returns it.
    ///
    /// For optimization purposes, the difficulty of the challenge must be able to fit in a [`u32`].
    /// This shouldn't be an issue, since difficulties that can't fit into a [`u32`] will probably take too long anyways.
    pub fn decode_challenge(chall_string: &str) -> Result<ChallengeParams, &'static str> {
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
            .map(|x| {
                BASE64_STANDARD
                    .decode(x)
                    .map_err(|_| "Parts aren't valid base64")
            })
            .collect::<Result<_, _>>()?;
        let difficulty_bytes = &decoded_data[0];
        let difficulty: u32 = if difficulty_bytes.len() > 4 {
            let (first, last) = difficulty_bytes.split_at(difficulty_bytes.len() - 4);
            // if difficulty is 0-padded to longer than 4 bytes it should still work
            if first.iter().any(|&x| x != 0) {
                return Err("Difficulty is too large");
            }
            u32::from_be_bytes(last.try_into().unwrap())
        } else {
            let mut difficulty_array = [0; 4];
            difficulty_array[4 - difficulty_bytes.len()..].copy_from_slice(difficulty_bytes);
            u32::from_be_bytes(difficulty_array)
        };
        Ok(Self {
            val: Integer::from_digits(&decoded_data[1], Order::Msf),
            difficulty,
        })
    }

    /// Generates a random challenge given a difficulty.
    pub fn generate_challenge(difficulty: u32) -> ChallengeParams {
        let mut bytes: [u8; 16] = [0; 16];
        thread_rng().fill(&mut bytes[..]);
        Self {
            val: Integer::from_digits(&bytes, Order::Msf),
            difficulty,
        }
    }

    /// Solves a challenge given a proof-of-work system and returns the solution.
    pub fn solve(mut self, pow: &KctfPow) -> String {
        for _ in 0..self.difficulty {
            // guaranteed to succeed so ignore the result
            let _ = self.val.pow_mod_mut(&pow.exponent, &pow.modulus);
            self.val ^= 1;
        }
        format!(
            "{}.{}",
            VERSION,
            BASE64_STANDARD.encode(self.val.to_digits(Order::Msf))
        )
    }

    /// Checks a solution to see if it satisfies the challenge under a given proof-of-work system.
    pub fn check(&self, pow: &KctfPow, sol: &str) -> Result<bool, &'static str> {
        let mut parts = sol.split('.');
        if parts.next() != Some(VERSION) {
            return Err("Incorrect version");
        }
        let data = match parts.next() {
            Some(x) => x,
            None => return Err("Incorrect number of parts"),
        };
        if parts.next().is_some() {
            return Err("Incorrect number of parts");
        }
        let decoded_data = BASE64_STANDARD
            .decode(data)
            .map_err(|_| "Parts aren't valid base64")?;
        let mut sol_val = Integer::from_digits(&decoded_data, Order::Msf);
        for _ in 0..self.difficulty {
            sol_val ^= 1;
            // guaranteed to succeed so ignore the result
            let _ = sol_val.pow_mod_mut(&2.into(), &pow.modulus);
        }
        Ok(self.val == sol_val || Integer::from(&pow.modulus - &self.val) == sol_val)
    }
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
        Ok(Challenge {
            params: ChallengeParams::decode_challenge(chall_string)?,
            pow: self,
        })
    }

    /// Generates a random challenge given a difficulty.
    pub fn generate_challenge(&self, difficulty: u32) -> Challenge {
        Challenge {
            params: ChallengeParams::generate_challenge(difficulty),
            pow: self,
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
    pub fn solve(self) -> String {
        self.params.solve(self.pow)
    }

    /// Checks a solution to see if it satisfies the challenge.
    pub fn check(&self, sol: &str) -> Result<bool, &'static str> {
        self.params.check(self.pow, sol)
    }
}

impl fmt::Display for ChallengeParams {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}.{}.{}",
            VERSION,
            BASE64_STANDARD.encode(self.difficulty.to_be_bytes()),
            BASE64_STANDARD.encode(self.val.to_digits(Order::Msf))
        )
    }
}

impl<'a> fmt::Display for Challenge<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.params)
    }
}
