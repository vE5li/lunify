#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Error during [unify](super::unify).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LunifyError {
    UnsupportedVersion(u8),
    IncorrectSignature,
    TooShort,
    TooLong,
    InvalidOpcode(u64),
    InvalidConstantType(u8),
    UnsupportedInstructionFormat([u8; 4]),
    UnsupportedSizeTSize(u8),
    UnsupportedIntegerSize(u8),
    UnsupportedInstructionSize(u8),
    UnsupportedNumberSize(u8),
}
