use super::InstructionLayout;
use crate::serialization::ByteStream;
use crate::{LunifyError, Settings};

pub(crate) trait RepresentInstruction: Sized {
    fn from_byte_stream(byte_stream: &mut ByteStream, settings: &Settings, layout: &InstructionLayout) -> Result<Self, LunifyError>;
    fn to_u64(&self, settings: &Settings, layout: &InstructionLayout) -> u64;
}

impl RepresentInstruction for u64 {
    fn from_byte_stream(byte_stream: &mut ByteStream, _settings: &Settings, _layout: &InstructionLayout) -> Result<Self, LunifyError> {
        byte_stream.instruction()
    }

    fn to_u64(&self, _settings: &Settings, _layout: &InstructionLayout) -> u64 {
        *self
    }
}
