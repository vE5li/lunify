use super::InstructionLayout;
use crate::serialization::ByteStream;
use crate::{LunifyError, Settings};

pub(crate) trait LuaInstruction: Sized {
    fn from_byte_stream(byte_stream: &mut ByteStream, settings: &Settings, layout: &InstructionLayout) -> Result<Self, LunifyError>;
    fn move_stack_accesses(&mut self, stack_start: u64, offset: i64);
    fn to_u64(&self, settings: &Settings) -> Result<u64, LunifyError>;
}
