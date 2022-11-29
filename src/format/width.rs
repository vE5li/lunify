use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The width of Lua-internal types in bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BitWidth {
    /// 32 bits
    Bit32,
    /// 64 bits
    Bit64,
}

impl TryFrom<u8> for BitWidth {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            4 => Ok(BitWidth::Bit32),
            8 => Ok(BitWidth::Bit64),
            width => Err(width),
        }
    }
}

impl From<BitWidth> for u8 {
    fn from(value: BitWidth) -> Self {
        match value {
            BitWidth::Bit32 => 4,
            BitWidth::Bit64 => 8,
        }
    }
}

impl Display for BitWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitWidth::Bit32 => write!(f, "32 bit"),
            BitWidth::Bit64 => write!(f, "64 bit"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::BitWidth;

    #[test]
    fn bit32() {
        assert_eq!(4.try_into(), Ok(BitWidth::Bit32));
    }

    #[test]
    fn bit64() {
        assert_eq!(8.try_into(), Ok(BitWidth::Bit64));
    }

    #[test]
    fn unsupported_width() {
        assert_eq!(BitWidth::try_from(6), Err(6));
    }

    #[test]
    fn format_bit32() {
        assert_eq!(format!("{}", BitWidth::Bit32).as_str(), "32 bit");
    }

    #[test]
    fn format_bit64() {
        assert_eq!(format!("{}", BitWidth::Bit64).as_str(), "64 bit");
    }
}
