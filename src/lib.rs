//! A library to solve, check, and generate proof-of-work challenges using [kCTF](https://google.github.io/kctf/)'s scheme.
//!
//! ```rust
//! use kctf_pow::ChallengeParams;
//!
//! // decoding then solving a challenge
//! let chall = ChallengeParams::decode_challenge("s.AAAAMg==.H+fPiuL32DPbfN97cpd0nA==").unwrap();
//! println!("{}", chall.solve());
//! // decoding then checking a challenge
//! let chall = ChallengeParams::decode_challenge("s.AAAAMg==.NDtqORW1uZlIgzszbdMGZA==").unwrap();
//! let sol = "s.NUH3arymnKB+ysUGdv+67ypDamn4wOKCPORB2ivWE1Yhinam2v4S6q4nAoC5LP97LScdVoq+NuFVF++Win5mNRYZS6bJAs8fk0h8XgvfcC/7JfmFISqeCIo/CIUgIucVAM+eGDjqitRULGXqIOyviJoJjW8DMouMRuJM/3eg/z18kutQHkX0N3sqPeF7Nzkk8S3Bs6aiHUORM30syUKYug==";
//! assert_eq!(chall.check(sol), Ok(true));
//! assert_eq!(chall.check("s.asdf"), Ok(false));
//! // generating a random challenge of difficulty 50
//! let chall = ChallengeParams::generate_challenge(50);
//! println!("{}", chall);
//! ```

use base64::prelude::*;
use rand::prelude::*;
use std::convert::TryInto;
use std::fmt;

mod bigint;
use bigint::BigInt;

const VERSION: &str = "s";

/// The parameters for a proof-of-work challenge.
///
/// This contains most of the logic, however [`KctfPow`] and [`Challenge`] should be used instead as they provide a nicer API.
/// If you want to serialize it to a string, use the [`Display`](std::fmt::Display) implementation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChallengeParams {
    /// The difficulty of the challenge.
    pub difficulty: u32,
    /// The starting value of the challenge.
    pub val: BigInt,
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
            val: BigInt::from_be_bytes(&decoded_data[1]),
            difficulty,
        })
    }

    /// Generates a random challenge given a difficulty.
    pub fn generate_challenge(difficulty: u32) -> ChallengeParams {
        let mut bytes = [0; 16];
        rand::rng().fill(&mut bytes[..]);
        Self {
            val: BigInt::from_be_bytes(&bytes),
            difficulty,
        }
    }

    /// Solves a challenge given a proof-of-work system and returns the solution.
    pub fn solve(mut self) -> String {
        for _ in 0..self.difficulty {
            // guaranteed to succeed so ignore the result
            for _ in 0..1277 {
                self.val.square_mod();
            }
            self.val.xor_one();
        }
        format!(
            "{}.{}",
            VERSION,
            BASE64_STANDARD.encode(self.val.to_be_bytes())
        )
    }

    /// Checks a solution to see if it satisfies the challenge under a given proof-of-work system.
    pub fn check(&self, sol: &str) -> Result<bool, &'static str> {
        let mut parts = sol.split('.');
        if parts.next() != Some(VERSION) {
            return Err("Incorrect version");
        }
        let Some(data) = parts.next() else {
            return Err("Incorrect number of parts");
        };
        if parts.next().is_some() {
            return Err("Incorrect number of parts");
        }
        let decoded_data = BASE64_STANDARD
            .decode(data)
            .map_err(|_| "Parts aren't valid base64")?;
        let mut sol_val = BigInt::from_be_bytes(&decoded_data);
        for _ in 0..self.difficulty {
            sol_val.xor_one();
            sol_val.square_mod();
        }

        Ok(self.val == sol_val || self.val.negate_mod() == sol_val)
    }
}

impl fmt::Display for ChallengeParams {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}.{}.{}",
            VERSION,
            BASE64_STANDARD.encode(self.difficulty.to_be_bytes()),
            BASE64_STANDARD.encode(self.val.to_be_bytes())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHALLENGE: &str = "s.AAAAZA==.KskOPzEduBg+z0cbeBsA1A==";
    const VALID_SOL: &str = "s.aNL1WVbhmQGstx2jyfYcYTmguYKzEugAMwYL9LP4Z36Q5CVgAbvbCpZNSfHWclBqUWSBFswrQcHoEGmhFK+QCIFNEdDY5diLBXNQ3CWO8DGceDmMqyVmEU0r7bBUtuXAFfP7c2CPUYHm7FviyUW3PTprzJMk3hjBINvW76j2Vu5pM931Ex0RTTBMYge8/Zjnlz/KuSiqptUDiaRHtbxkqQ==";
    const INVALID_SOL: &str = "s.FR4SdG/xcaMJ+AtZyq7uSbc21c2V8oENE1J926og8tBrFmIwWqsOtKUIPbnLffcdYk8n9p2FYX62jjjKVAvIRnrhThjbA0gi5XznkuupMzIoX9BbD/2y5qL0D9L9ZYSMGeYIxvyoZ/9YKnDVnGRdU+PapGA3oo/IDaK1Ev74w+uknalVqj4rnd9PFmlDmOU614H7mfC1gxGBMmlzurhEEQ==";

    #[test]
    fn test_solve() {
        let chall = ChallengeParams::decode_challenge(CHALLENGE).unwrap();
        let solve = chall.solve();
        assert_eq!(&solve, VALID_SOL);
    }

    #[test]
    fn test_check() {
        let chall = ChallengeParams::decode_challenge(CHALLENGE).unwrap();
        assert!(chall.clone().check(VALID_SOL).unwrap());
        assert!(!chall.check(INVALID_SOL).unwrap());
    }

    #[test]
    fn test_gen() {
        let chall = ChallengeParams::generate_challenge(100);
        assert!(chall.to_string().len() <= 35);
        assert!(chall.difficulty == 100);
        assert!(chall.check(&chall.clone().solve()).unwrap());
    }
}
