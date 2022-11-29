use crate::serialization::ByteStream;
use crate::LunifyError;

pub(crate) trait RepresentInstruction: Sized {
    fn from_byte_stream(byte_stream: &mut ByteStream) -> Result<Self, LunifyError>;
    fn to_u64(&self) -> u64;
}

impl RepresentInstruction for u64 {
    fn from_byte_stream(byte_stream: &mut ByteStream) -> Result<Self, LunifyError> {
        byte_stream.instruction()
    }

    fn to_u64(&self) -> u64 {
        *self
    }
}
