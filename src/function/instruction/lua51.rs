use std::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::operant::{Bx, Opcode, SignedBx, BC};
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

    pub(crate) fn move_stack_accesses(&mut self, stack_start: u64, offset: i64, settings: &Settings) -> Result<(), LunifyError> {
        let offset_a = |position: &mut u64| {
            let value = *position;
            if value >= stack_start {
                *position = (value as i64 + offset) as u64;
            }
        };

        let offset_constant = |position: &mut u64| {
            let value = *position;
            let is_constant = value & settings.get_constant_bit() != 0;

            if value >= stack_start && !is_constant {
                *position = (value as i64 + offset) as u64;

                // If the stack position is bigger than the maximum constant index, we exeeded
                // the maximum stack index, meaning our value would now be
                // interpreted as a constant. At this point we can only return
                // an error.
                if *position > settings.get_maximum_constant_index() {
                    return Err(LunifyError::ValueTooTooBigForOperant);
                }
            }

            Ok(())
        };

        match self {
            Instruction::Move { a, mode: BC(b, _) } => {
                offset_a(a);
                offset_constant(b)?;
            }
            Instruction::LoadK { a, .. } => offset_a(a),
            Instruction::LoadBool { a, .. } => offset_a(a),
            Instruction::LoadNil { a, mode: BC(b, _) } => {
                offset_a(a);
                offset_constant(b)?;
            }
            Instruction::GetUpValue { a, .. } => offset_a(a),
            Instruction::GetGlobal { a, .. } => offset_a(a),
            Instruction::GetTable { a, mode: BC(b, c) } => {
                offset_a(a);
                offset_constant(b)?;
                offset_constant(c)?;
            }
            Instruction::SetGlobal { a, .. } => offset_a(a),
            Instruction::SetUpValue { a, .. } => offset_a(a),
            Instruction::SetTable { a, mode: BC(b, c) } => {
                offset_a(a);
                offset_constant(b)?;
                offset_constant(c)?;
            }
            Instruction::NewTable { a, .. } => offset_a(a),
            Instruction::_Self { a, mode: BC(b, c) } => {
                offset_a(a);
                offset_constant(b)?;
                offset_constant(c)?;
            }
            Instruction::Add { a, mode: BC(b, c) }
            | Instruction::Subtract { a, mode: BC(b, c) }
            | Instruction::Multiply { a, mode: BC(b, c) }
            | Instruction::Divide { a, mode: BC(b, c) }
            | Instruction::Modulo { a, mode: BC(b, c) }
            | Instruction::Power { a, mode: BC(b, c) } => {
                offset_a(a);
                offset_constant(b)?;
                offset_constant(c)?;
            }
            Instruction::Unary { a, mode: BC(b, _) }
            | Instruction::Not { a, mode: BC(b, _) }
            | Instruction::Length { a, mode: BC(b, _) } => {
                offset_a(a);
                offset_constant(b)?;
            }
            Instruction::Concatinate { a, mode: BC(b, c) } => {
                offset_a(a);
                offset_constant(b)?;
                offset_constant(c)?;
            }
            Instruction::Jump { .. } => {}
            Instruction::Equals { mode: BC(b, c), .. }
            | Instruction::LessThan { mode: BC(b, c), .. }
            | Instruction::LessEquals { mode: BC(b, c), .. } => {
                offset_constant(b)?;
                offset_constant(c)?;
            }
            Instruction::Test { a, .. } => offset_a(a),
            Instruction::TestSet { a, mode: BC(b, _) } => {
                offset_a(a);
                offset_constant(b)?;
            }
            Instruction::Call { a, .. } => offset_a(a),
            Instruction::TailCall { a, .. } => offset_a(a),
            Instruction::Return { a, .. } => offset_a(a),
            Instruction::ForLoop { a, .. } => offset_a(a),
            Instruction::ForPrep { a, .. } => offset_a(a),
            Instruction::TForLoop { a, .. } => offset_a(a),
            Instruction::SetList { a, .. } => offset_a(a),
            Instruction::Close { a, .. } => offset_a(a),
            Instruction::Closure { a, .. } => offset_a(a),
            Instruction::VarArg { a, .. } => offset_a(a),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Instruction, Settings};
    use crate::function::instruction::BC;
    use crate::LunifyError;

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

    #[test]
    fn instruction_move_stack_access() -> Result<(), LunifyError> {
        let settings = Settings::default();
        let mut instruction = Instruction::GetTable { a: 1, mode: BC(1, 1) };

        instruction.move_stack_accesses(0, 8, &settings)?;
        let Instruction::GetTable { a, mode: BC(b, c) } = instruction else {
            unreachable!();
        };

        assert_eq!(a, 9);
        assert_eq!(b, 9);
        assert_eq!(c, 9);
        Ok(())
    }

    #[test]
    fn instruction_move_stack_access_below() -> Result<(), LunifyError> {
        let settings = Settings::default();
        let mut instruction = Instruction::GetTable { a: 9, mode: BC(9, 9) };

        instruction.move_stack_accesses(10, 9, &settings)?;
        let Instruction::GetTable { a, mode: BC(b, c) } = instruction else {
            unreachable!();
        };

        assert_eq!(a, 9);
        assert_eq!(b, 9);
        assert_eq!(c, 9);
        Ok(())
    }

    #[test]
    fn instruction_move_stack_access_constant() -> Result<(), LunifyError> {
        let settings = Settings::default();
        let mut instruction = Instruction::GetTable {
            a: 1,
            mode: BC(1 | settings.get_constant_bit(), 1 | settings.get_constant_bit()),
        };

        instruction.move_stack_accesses(0, 8, &settings)?;
        let Instruction::GetTable { a, mode: BC(b, c) } = instruction else {
            unreachable!();
        };

        assert_eq!(a, 9);
        assert_eq!(b, 1 | settings.get_constant_bit());
        assert_eq!(c, 1 | settings.get_constant_bit());
        Ok(())
    }

    #[test]
    fn instruction_move_stack_access_overflow() {
        let settings = Settings::default();
        let mut instruction = Instruction::GetTable { a: 1, mode: BC(1, 1) };

        let result = instruction.move_stack_accesses(0, settings.get_maximum_constant_index() as i64, &settings);
        assert_eq!(result, Err(LunifyError::ValueTooTooBigForOperant));
    }
}
