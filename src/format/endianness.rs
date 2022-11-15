#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::LunifyError;

/// List of supported Lua versions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Endianness {
    /// Least significant byte first.
    Big,
    /// Most significant byte first.
    Little,
}

impl TryFrom<u8> for Endianness {
    type Error = LunifyError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Endianness::Big),
            1 => Ok(Endianness::Little),
            endianness => Err(LunifyError::InvaildEndianness(endianness)),
        }
    }
}

impl From<Endianness> for u8 {
    fn from(value: Endianness) -> Self {
        match value {
            Endianness::Big => 0,
            Endianness::Little => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Endianness, LunifyError};

    #[test]
    fn big_endian() {
        assert_eq!(0.try_into(), Ok(Endianness::Big));
    }

    #[test]
    fn small_endian() {
        assert_eq!(1.try_into(), Ok(Endianness::Little));
    }

    #[test]
    fn unsupported_endianness() {
        assert_eq!(Endianness::try_from(2), Err(LunifyError::InvaildEndianness(2)));
    }
}
