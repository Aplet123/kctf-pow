use rug::Integer;
use rug::integer::Order;
use rug::ops::Pow;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BigInt {
    inner: Integer,
}

impl BigInt {
    /// Squares a BigInt in-place modulo 2^1279-1
    pub fn square_mod(&mut self) {
        let n = &mut self.inner;
        n.square_mut();
        let high = Integer::from(&*n >> 1279);
        n.keep_bits_mut(1279);
        *n += high;
        if n.get_bit(1279) {
            n.set_bit(1279, false);
            *n += 1;
        }
    }

    /// Negates a BigInt modulo 2^1279-1
    pub fn negate_mod(&self) -> Self {
        BigInt {
            inner: Integer::from(2).pow(1279) - 1 - &self.inner,
        }
    }

    /// Xors a BigInt in-place by 1
    pub fn xor_one(&mut self) {
        self.inner ^= 1u8;
    }

    /// Constructs a BigInt from a slice of big endian bytes.
    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        Self {
            inner: Integer::from_digits(bytes, Order::Msf),
        }
    }

    /// Converts a BigInt to an array of big endian bytes.
    pub fn to_be_bytes(&self) -> Vec<u8> {
        self.inner.to_digits(Order::Msf)
    }
}
