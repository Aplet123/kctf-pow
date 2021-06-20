use rand::prelude::*;
use rug::integer::Order;
use rug::ops::Pow;
use rug::Integer;
use std::fmt;

const VERSION: &'static str = "s";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KctfPow {
    modulus: Integer,
    exponent: Integer,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Challenge<'a> {
    difficulty: u32,
    val: Integer,
    pow: &'a KctfPow,
}

impl KctfPow {
    pub fn new() -> Self {
        let modulus: Integer = Integer::from(2).pow(1279) - 1;
        let exponent = (modulus.clone() + 1) / 4;
        Self { modulus, exponent }
    }

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
        if difficulty_bytes.len() > 4 {
            return Err("Difficulty is too large");
        }
        let mut difficulty_array = [0; 4];
        difficulty_array[4 - difficulty_bytes.len()..].copy_from_slice(difficulty_bytes);
        Ok(Challenge {
            pow: self,
            val: Integer::from_digits(&decoded_data[1], Order::Msf),
            difficulty: u32::from_be_bytes(difficulty_array),
        })
    }

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

    pub fn check(&self, sol: &str) -> Result<bool, &'static str> {
        let mut parts = sol.split('.');
        if parts.next() != Some(VERSION) {
            return Err("Incorrect version");
        }
        let data = match parts.next() {
            Some(x) => x,
            None => return Err("Incorrect number of parts")
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
