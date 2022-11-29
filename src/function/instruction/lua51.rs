use std::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::operant::{Bx, SignedBx, BC};

/// Lua 5.1 compile constants. The Lua interpreter is compiled with certain
/// predefined constants that affect how the bytecode is generated. This
/// structure represents a small subset of the constants that are relevant for
/// Lunify. If the bytecode you are trying to modify was complied with
/// non-standard constants, you can use these settings to make it compatible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Settings {
    /// Number of elements to put on the stack before insterting a SETLIST
    /// instruction (LFIELDS_PER_FLUSH).
    pub fields_per_flush: u64,
    /// Maximum number of elements that can be on the stack at the same time
    /// (MAXSTACK).
    pub stack_limit: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fields_per_flush: 50,
            stack_limit: 250,
        }
    }
}

lua_instructions! {
    Move(BC),
    LoadK(Bx),
    LoadBool(BC),
    LoadNil(BC),
    GetUpValue(BC),
    GetGlobal(Bx),
    GetTable(BC),
    SetGlobal(Bx),
    SetUpValue(BC),
    SetTable(BC),
    NewTable(BC),
    _Self(BC),
    Add(BC),
    Subtract(BC),
    Multiply(BC),
    Divide(BC),
    Modulo(BC),
    Power(BC),
    Unary(BC),
    Not(BC),
    Length(BC),
    Concatinate(BC),
    Jump(SignedBx),
    Equals(BC),
    LessThan(BC),
    LessEquals(BC),
    Test(BC),
    TestSet(BC),
    Call(BC),
    TailCall(BC),
    Return(BC),
    ForLoop(SignedBx),
    ForPrep(SignedBx),
    TForLoop(BC),
    SetList(BC),
    Close(BC),
    Closure(Bx),
    VarArg(BC),
}

impl Instruction {
    pub(crate) fn stack_destination(&self) -> Option<Range<u64>> {
        match *self {
            Instruction::Move { a, .. } => Some(a..a),
            Instruction::LoadK { a, .. } => Some(a..a),
            Instruction::LoadBool { a, .. } => Some(a..a),
            Instruction::LoadNil { a, mode: BC(b, _) } => Some(a..b),
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
            Instruction::Call { a, mode: BC(_, c) } => Some(a..a + c - 1),
            Instruction::TailCall { .. } => None,
            Instruction::Return { .. } => None,
            Instruction::ForLoop { a, .. } => Some(a..a + 3),
            Instruction::ForPrep { a, .. } => Some(a..a + 2),
            Instruction::TForLoop { a, mode: BC(_, c) } => Some(a..a + 2 + c),
            Instruction::SetList { a, .. } => Some(a..a),
            Instruction::Close { .. } => None,
            Instruction::Closure { a, .. } => Some(a..a),
            Instruction::VarArg { a, mode: BC(b, _) } => Some(a..a.max((a + b).saturating_sub(1))),
        }
    }

    pub(crate) fn move_stack_accesses(&mut self, stack_start: u64, offset: i64) {
        let offset = |position: &mut u64| {
            let value = *position;
            if value >= stack_start && value & super::operant::LUA_CONST_BIT == 0 {
                *position = (value as i64 + offset) as u64;
            }
        };

        match self {
            Instruction::Move { a, mode: BC(b, _) } => {
                offset(a);
                offset(b);
            }
            Instruction::LoadK { a, .. } => offset(a),
            Instruction::LoadBool { a, .. } => offset(a),
            Instruction::LoadNil { a, mode: BC(b, _) } => {
                offset(a);
                offset(b);
            }
            Instruction::GetUpValue { a, .. } => offset(a),
            Instruction::GetGlobal { a, .. } => offset(a),
            Instruction::GetTable { a, mode: BC(b, c) } => {
                offset(a);
                offset(b);
                offset(c);
            }
            Instruction::SetGlobal { a, .. } => offset(a),
            Instruction::SetUpValue { a, .. } => offset(a),
            Instruction::SetTable { a, mode: BC(b, c) } => {
                offset(a);
                offset(b);
                offset(c);
            }
            Instruction::NewTable { a, .. } => offset(a),
            Instruction::_Self { a, mode: BC(b, c) } => {
                offset(a);
                offset(b);
                offset(c);
            }
            Instruction::Add { a, mode: BC(b, c) }
            | Instruction::Subtract { a, mode: BC(b, c) }
            | Instruction::Multiply { a, mode: BC(b, c) }
            | Instruction::Divide { a, mode: BC(b, c) }
            | Instruction::Modulo { a, mode: BC(b, c) }
            | Instruction::Power { a, mode: BC(b, c) } => {
                offset(a);
                offset(b);
                offset(c);
            }
            Instruction::Unary { a, mode: BC(b, _) }
            | Instruction::Not { a, mode: BC(b, _) }
            | Instruction::Length { a, mode: BC(b, _) } => {
                offset(a);
                offset(b);
            }
            Instruction::Concatinate { a, mode: BC(b, c) } => {
                offset(a);
                offset(b);
                offset(c);
            }
            Instruction::Jump { .. } => {}
            Instruction::Equals { mode: BC(b, c), .. }
            | Instruction::LessThan { mode: BC(b, c), .. }
            | Instruction::LessEquals { mode: BC(b, c), .. } => {
                offset(b);
                offset(c);
            }
            Instruction::Test { a, .. } => offset(a),
            Instruction::TestSet { a, mode: BC(b, _) } => {
                offset(a);
                offset(b);
            }
            Instruction::Call { a, .. } => offset(a),
            Instruction::TailCall { a, .. } => offset(a),
            Instruction::Return { a, .. } => offset(a),
            Instruction::ForLoop { a, .. } => offset(a),
            Instruction::ForPrep { a, .. } => offset(a),
            Instruction::TForLoop { a, .. } => offset(a),
            Instruction::SetList { a, .. } => offset(a),
            Instruction::Close { a, .. } => offset(a),
            Instruction::Closure { a, .. } => offset(a),
            Instruction::VarArg { a, .. } => offset(a),
        }
    }
}
