use super::InstructionLayout;
use crate::serialization::ByteStream;
use crate::{LunifyError, Settings};

pub(crate) trait RepresentInstruction: Sized {
    fn from_byte_stream(byte_stream: &mut ByteStream, settings: &Settings, layout: &InstructionLayout) -> Result<Self, LunifyError>;
    fn to_u64(&self, settings: &Settings) -> Result<u64, LunifyError>;
}
