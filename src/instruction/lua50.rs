#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::RepresentInstruction;
use crate::stream::ByteStream;
use crate::LunifyError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Settings {
    pub fields_per_flush: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self { fields_per_flush: 32 }
    }
}

opcodes! {
    Move,
    LoadK,
    LoadBool,
    LoadNil,
    GetUpValue,
    GetGlobal,
    GetTable,
    SetGlobal,
    SetUpValue,
    SetTable,
    NewTable,
    _Self,
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Unary,
    Not,
    Concatinate,
    Jump,
    Equals,
    LessThan,
    LessEquals,
    Test,
    Call,
    TailCall,
    Return,
    ForLoop,
    TForLoop,
    TForPrep,
    SetList,
    SetListO,
    Close,
    Closure,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Instruction(pub(super) u64);

#[cfg(test)]
impl Instruction {
    pub(super) fn new(opcode: Opcode, a: u64, b: u64, c: u64) -> Self {
        let mut instruction = opcode as u64;
        instruction |= c << 6;
        instruction |= b << 15;
        instruction |= a << 24;

        #[cfg(feature = "debug")]
        {
            println!("=== generated ===");
            println!("Opcode {:?}", opcode);
            println!("A {}", a);
            println!("B {}", b);
            println!("C {}", c);
        }

        #[cfg(feature = "debug")]
        println!("{:0>32b}\n", instruction);

        Self(instruction)
    }

    pub(super) fn new_bx(opcode: Opcode, a: u64, bx: u64) -> Self {
        let mut instruction = opcode as u64;

        instruction |= a << 24;
        instruction |= bx << 6;

        #[cfg(feature = "debug")]
        println!("{:0>32b}\n", instruction);

        Self(instruction)
    }
}

impl RepresentInstruction for Instruction {
    fn from_byte_stream(byte_stream: &mut ByteStream) -> Result<Self, LunifyError> {
        let instruction = byte_stream.instruction()?;
        Ok(Self(instruction))
    }

    fn to_u64(&self) -> u64 {
        self.0
    }
}
