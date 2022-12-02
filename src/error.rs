#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Error during [unify](super::unify).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LunifyError {
    /// The specified instruction layout is not valid. This can happen when size
    /// limitations are not respected, when operant types are specified more
    /// than once, or when the B and C operants are not adjacent.
    InvalidInstructionLayout,
    /// The provided bytecode does not start with the signature `\[27]Lua`.
    IncorrectSignature,
    /// The version of Lua that the bytecode was compiled for is not supported
    /// by Lunify.
    UnsupportedVersion(u8),
    /// The specified format does not specify a valid endianness.
    InvaildEndianness(u8),
    /// The instruction memory layout is not supported by Lunify. This error can
    /// only occur in Lua 5.0.
    UnsupportedInstructionFormat([u8; 4]),
    /// The pointer width of the system that the bytecode was compiled for is
    /// not supported by Lunify.
    UnsupportedSizeTWidth(u8),
    /// The integer width of the system that the bytecode was compiled for is
    /// not supported by Lunify.
    UnsupportedIntegerWidth(u8),
    /// The instruction width of the system that the bytecode was compiled for
    /// is not supported by Lunify.
    UnsupportedInstructionWidth(u8),
    /// The number width of the system that the bytecode was compiled for
    /// is not supported by Lunify.
    UnsupportedNumberWidth(u8),
    /// The bytecode contains an instruction that is not recognized by Lunify.
    InvalidOpcode(u64),
    /// The bytecode contains a constant with a type that is not recognized by
    /// Lunify.
    InvalidConstantType(u8),
    /// The bytecode contains a number with a non-integral value that can't be
    /// represented when `is_number_integral` is set to true.
    FloatPrecisionLoss,
    /// The bytecode contains an integral value that is too big to be
    /// represented when `is_number_integral` is set to false.
    IntegerOverflow,
    /// The bytecode is truncated.
    InputTooShort,
    /// The bytecode has access padding.
    InputTooLong,
    /// The bytecode generated by converting is using stack values that are
    /// bigger than Lua 5.1 `MAXSTACK`.
    StackTooLarge(u64),
    /// The bytecode generated by converting is using more constants than Lua
    /// 5.1 supports `MAXINDEXRK`.
    TooManyConstants(u64),
    /// The bytecode generated by converting to Lua 5.1 needs to store a value
    /// in an operant that exeed the maximum possible value.
    ValueTooTooBigForOperant,
    /// The Lua 5.0 `FORLOOP` instruction specified a positive jump, even though
    /// we expect it to always be negative.
    UnexpectedForwardJump,
}
