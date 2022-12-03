#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::operant::{Bx, Generic, Opcode, SignedBx, A, BC};
use super::{ConstantRegister, InstructionLayout, OperantType, Register, Unused};
use crate::LunifyError;

/// Lua 5.0 compile constants. The Lua interpreter is compiled with certain
/// predefined constants that affect how the bytecode is generated. This
/// structure represents a small subset of the constants that are relevant for
/// Lunify. If the bytecode you are trying to modify was complied with
/// non-standard constants, you can use these settings to make it compatible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Settings<'a> {
    /// Maximum number of elements that can be on the stack at the same time
    /// (`MAXSTACK`).
    pub stack_limit: u64,
    /// Number of elements to put on the stack before inserting a SETLIST
    /// instruction (`LFIELDS_PER_FLUSH`).
    pub fields_per_flush: u64,
    /// Lua binary file signature (`LUA_SIGNATURE`).
    pub binary_signature: &'a str,
    /// Memory layout of instructions inside the Lua bytecode (`SIZE_*`,
    /// `POS_*`).
    pub layout: InstructionLayout,
}

impl<'a> Default for Settings<'a> {
    fn default() -> Self {
        Self {
            stack_limit: 250,
            fields_per_flush: 32,
            binary_signature: "\x1bLua",
            layout: InstructionLayout::from_specification([
                OperantType::Opcode(6),
                OperantType::C(9),
                OperantType::B(9),
                OperantType::A(8),
            ])
            .unwrap(),
        }
    }
}

lua_instructions! {
    Move(BC<Register, Unused>, true),
    LoadK(Bx, true),
    LoadBool(BC<Generic, Generic>, true),
    LoadNil(BC<Register, Unused>, true),
    GetUpValue(BC<Generic, Unused>, true),
    GetGlobal(Bx, true),
    GetTable(BC<Register, ConstantRegister>, true),
    SetGlobal(Bx, true),
    SetUpValue(BC<Generic, Unused>, true),
    SetTable(BC<ConstantRegister, ConstantRegister>, true),
    NewTable(BC<Unused, Unused>, true),
    _Self(BC<Register, ConstantRegister>, true),
    Add(BC<ConstantRegister, ConstantRegister>, true),
    Subtract(BC<ConstantRegister, ConstantRegister>, true),
    Multiply(BC<ConstantRegister, ConstantRegister>, true),
    Divide(BC<ConstantRegister, ConstantRegister>, true),
    Power(BC<ConstantRegister, ConstantRegister>, true),
    Unary(BC<Register, ConstantRegister>, true),
    Not(BC<Register, Unused>, true),
    Concatinate(BC<Register, Register>, true),
    Jump(SignedBx, false),
    Equals(BC<ConstantRegister, ConstantRegister>, false),
    LessThan(BC<ConstantRegister, ConstantRegister>, false),
    LessEquals(BC<ConstantRegister, ConstantRegister>, false),
    Test(BC<Register, Generic>, true),
    Call(BC<Generic, Generic>, true),
    TailCall(BC<Generic, Generic>, true),
    Return(BC<Generic, Unused>, true),
    ForLoop(SignedBx, true),
    TForLoop(BC<Unused, Generic>, true),
    TForPrep(SignedBx, true),
    SetList(Bx, true),
    SetListO(Bx, true),
    Close(BC<Unused, Unused>, true),
    Closure(Bx, true),
}
