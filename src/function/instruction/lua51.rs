use std::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::operant::{Bx, ConstantRegister, Generic, Opcode, Register, SignedBx, Unused, A, BC};
use super::{InstructionLayout, OperantType};
use crate::LunifyError;

/// Lua 5.1 compile constants. The Lua interpreter is compiled with certain
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
    /// Number of elements to put on the stack before inserting a `SETLIST`
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
            fields_per_flush: 50,
            stack_limit: 250,
            binary_signature: "\x1bLua",
            layout: InstructionLayout::from_specification([
                OperantType::Opcode(6),
                OperantType::A(8),
                OperantType::C(9),
                OperantType::B(9),
            ])
            .unwrap(),
        }
    }
}

impl<'a> Settings<'a> {
    pub(crate) fn get_constant_bit(&self) -> u64 {
        1 << (self.layout.b.size - 1)
    }

    pub(crate) fn get_maximum_constant_index(&self) -> u64 {
        self.get_constant_bit() - 1
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
    Modulo(BC<ConstantRegister, ConstantRegister>, true),
    Power(BC<ConstantRegister, ConstantRegister>, true),
    Unary(BC<Register, ConstantRegister>, true),
    Not(BC<Register, Unused>, true),
    Length(BC<Register, Unused>, true),
    Concatinate(BC<Register, Register>, true),
    Jump(SignedBx, false),
    Equals(BC<ConstantRegister, ConstantRegister>, false),
    LessThan(BC<ConstantRegister, ConstantRegister>, false),
    LessEquals(BC<ConstantRegister, ConstantRegister>, false),
    Test(BC<Register, Generic>, true),
    TestSet(BC<ConstantRegister, Generic>, true),
    Call(BC<Generic, Generic>, true),
    TailCall(BC<Generic, Generic>, true),
    Return(BC<Generic, Unused>, true),
    ForLoop(SignedBx, true),
    ForPrep(SignedBx, true),
    TForLoop(BC<Unused, Generic>, true),
    SetList(BC<Generic, Generic>, true),
    Close(BC<Unused, Unused>, true),
    Closure(Bx, true),
    VarArg(BC<Generic, Unused>, true),
}

impl Instruction {
    /// Get the stack index that a given instruction will move data into.
    /// SetTable and SetList are technically not moving any data, but rather
    /// modifying it, but we need this behaviour for detecting the correct
    /// SetList instruction inside the upcast function.
    pub(crate) fn stack_destination(&self) -> Option<Range<u64>> {
        match *self {
            Instruction::Move { a, .. } => Some(a..a),
            Instruction::LoadK { a, .. } => Some(a..a),
            Instruction::LoadBool { a, .. } => Some(a..a),
            Instruction::LoadNil { a, mode: BC(b, _) } => Some(a..b.0),
            Instruction::GetUpValue { a, .. } => Some(a..a),
            Instruction::GetGlobal { a, .. } => Some(a..a),
            Instruction::GetTable { a, .. } => Some(a..a),
            Instruction::SetGlobal { .. } => None,
            Instruction::SetUpValue { .. } => None,
            Instruction::SetTable { a, .. } => Some(a..a),
            Instruction::NewTable { a, .. } => Some(a..a),
            Instruction::_Self { a, .. } => Some(a..a + 1),
            Instruction::Add { a, .. } => Some(a..a),
            Instruction::Subtract { a, .. } => Some(a..a),
            Instruction::Multiply { a, .. } => Some(a..a),
            Instruction::Divide { a, .. } => Some(a..a),
            Instruction::Modulo { a, .. } => Some(a..a),
            Instruction::Power { a, .. } => Some(a..a),
            Instruction::Unary { a, .. } => Some(a..a),
            Instruction::Not { a, .. } => Some(a..a),
            Instruction::Length { a, .. } => Some(a..a),
            Instruction::Concatinate { a, .. } => Some(a..a),
            Instruction::Jump { .. } => None,
            Instruction::Equals { .. } => None,
            Instruction::LessThan { .. } => None,
            Instruction::LessEquals { .. } => None,
            Instruction::Test { .. } => None,
            Instruction::TestSet { a, .. } => Some(a..a),
            Instruction::Call { a, mode: BC(_, c) } => Some(a..a + c.0 - 1),
            Instruction::TailCall { .. } => None,
            Instruction::Return { .. } => None,
            Instruction::ForLoop { a, .. } => Some(a..a + 3),
            Instruction::ForPrep { a, .. } => Some(a..a + 2),
            Instruction::TForLoop { a, mode: BC(_, c) } => Some(a..a + 2 + c.0),
            Instruction::SetList { a, .. } => Some(a..a),
            Instruction::Close { .. } => None,
            Instruction::Closure { a, .. } => Some(a..a),
            Instruction::VarArg { a, mode: BC(b, _) } => Some(a..a.max((a + b.0).saturating_sub(1))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;

    #[test]
    fn settings_get_constant_bit() {
        let settings = Settings::default();
        assert_eq!(settings.get_constant_bit(), 1 << 8);
    }

    #[test]
    fn settings_get_maximum_constant_index() {
        let settings = Settings::default();
        assert_eq!(settings.get_maximum_constant_index(), (1 << 8) - 1);
    }
}
