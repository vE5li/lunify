#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LunifyError {
    IncorrectSignature,
    TooShort,
    UnsupportedVersion(u8),
    InvalidFileName,
    TooLong,
    InvalidConstantType(u8),
    UnsupportedSizeTSize(u8),
    UnsupportedIntegerSize(u8),
    UnsupportedInstructionSize(u8),
    UnsupportedNumberSize(u8),
}
