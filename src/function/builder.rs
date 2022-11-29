use super::Settings;
use crate::lua51::Instruction;
use crate::LunifyError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InstructionContext {
    instruction: Instruction,
    line_weight: i64,
    final_offset: i64,
    is_fixed: bool,
}

impl InstructionContext {
    pub fn new(instruction: Instruction) -> Self {
        Self {
            instruction,
            line_weight: 0,
            final_offset: 0,
            is_fixed: false,
        }
    }

    pub fn new_extra(instruction: Instruction) -> Self {
        Self {
            instruction,
            line_weight: 1,
            final_offset: 0,
            is_fixed: false,
        }
    }
}

#[derive(Default)]
pub(super) struct InstructionBuilder {
    contexts: Vec<InstructionContext>,
    line_info: Vec<i64>,
    line_number: i64,
}

impl InstructionBuilder {
    pub(super) fn set_line_number(&mut self, line_number: i64) {
        self.line_number = line_number;
    }

    pub(super) fn instruction(&mut self, instruction: Instruction) {
        self.contexts.push(InstructionContext::new(instruction));
        self.line_info.push(self.line_number);
    }

    pub(super) fn extra_instruction(&mut self, instruction: Instruction) {
        self.contexts.push(InstructionContext::new_extra(instruction));
        self.line_info.push(self.line_number);
    }

    pub(super) fn insert_extra_instruction(&mut self, index: usize, instruction: Instruction) {
        let line_number = self.line_info[index];
        self.contexts.insert(index, InstructionContext::new_extra(instruction));
        self.line_info.insert(index, line_number);
    }

    pub(super) fn remove_instruction(&mut self, index: usize) {
        let removed = self.contexts.remove(index);
        self.line_info.remove(index);
        self.contexts[index].line_weight += removed.line_weight - 1;
    }

    pub(super) fn last_instruction_fixed(&mut self) {
        self.contexts.last_mut().unwrap().is_fixed = true;
    }

    pub(super) fn last_instruction_offset(&mut self, final_offset: i64) {
        self.contexts.last_mut().unwrap().final_offset = final_offset;
    }

    pub(super) fn get_instruction(&mut self, index: usize) -> &mut Instruction {
        &mut self.contexts[index].instruction
    }

    pub(super) fn get_program_counter(&self) -> usize {
        self.contexts.len()
    }

    fn jump_destination(&self, context_index: usize, mut bx: i64, final_offset: i64) -> Result<i64, LunifyError> {
        let (mut steps, mut offset) = match bx.is_positive() {
            true => (bx + 1, 1),
            false => (bx.abs(), 0),
        };

        while steps != 0 {
            let index = match bx.is_positive() {
                true => context_index + offset,
                false => context_index - offset,
            };
            let context = self.contexts[index];

            bx += context.line_weight * bx.signum();
            steps += context.line_weight - 1;
            offset += 1;
        }

        bx += final_offset;
        // TODO: Make sure that Bx is still in bounds.
        Ok(bx)
    }

    pub(super) fn adjusted_jump_destination(&self, bx: i64) -> Result<usize, LunifyError> {
        if bx.is_positive() {
            return Err(LunifyError::UnexpectedForwardJump);
        }

        let program_counter = self.get_program_counter();
        let new_bx = self.jump_destination(program_counter - 1, bx, 0)?;

        Ok(((program_counter as i64) + new_bx) as usize)
    }

    pub(super) fn finalize(mut self, maxstacksize: &mut u8, settings: Settings) -> Result<(Vec<Instruction>, Vec<i64>), LunifyError> {
        for context_index in 0..self.contexts.len() {
            // The stack positions might have changed significantly, so go over every
            // instruction and make sure that the maxstacksize is big enough. If the stack
            // had to be popped out too much in the conversion, we return an error.
            // We also know that values on the stack will only be used after they have been
            // put there by anther instruction, meaning if we make space for the
            // instructions that push the values onto the stack, the stack will never
            // overflow.
            if let Some(destination) = self.contexts[context_index].instruction.stack_destination() {
                let new_stack_size = destination.end + 1;
                match new_stack_size <= settings.lua51.stack_limit {
                    true => *maxstacksize = (*maxstacksize).max(new_stack_size as u8),
                    false => return Err(LunifyError::StackTooLarge(new_stack_size)),
                }
            }

            let new_bx = {
                let context = &self.contexts[context_index];
                let is_fixed = context.is_fixed;
                let final_offset = context.final_offset;

                match &context.instruction {
                    Instruction::Jump { mode, .. } | Instruction::ForLoop { mode, .. } | Instruction::ForPrep { mode, .. } if !is_fixed => {
                        Some(self.jump_destination(context_index, mode.0, final_offset)?)
                    }
                    _ => None,
                }
            };

            if let Some(bx) = new_bx {
                match &mut self.contexts[context_index].instruction {
                    Instruction::Jump { mode, .. } | Instruction::ForLoop { mode, .. } | Instruction::ForPrep { mode, .. } => {
                        mode.0 = bx;
                    }
                    _ => unreachable!(),
                }
            }

            #[cfg(feature = "debug")]
            {
                let context = &self.contexts[context_index];
                println!();
                println!("[{}] {:?}", context_index, context.instruction);
                println!(" -> {:?}", context.line_weight);
            }
        }

        let instructions = self.contexts.into_iter().map(|context| context.instruction).collect();
        Ok((instructions, self.line_info))
    }
}

#[cfg(test)]
mod tests {
    use super::InstructionBuilder;
    use crate::{lua51, LunifyError, function::{builder::InstructionContext, instruction::{Bx, SignedBx}}};

    #[test]
    fn instruction_context_new() {
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };
        let context = InstructionContext::new(instruction);
        let expected = InstructionContext {
            instruction,
            line_weight: 0,
            final_offset: 0,
            is_fixed: false,
        };

        assert_eq!(context, expected);
    }

    #[test]
    fn instruction_context_new_extra() {
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };
        let context = InstructionContext::new_extra(instruction);
        let expected = InstructionContext {
            instruction,
            line_weight: 1,
            final_offset: 0,
            is_fixed: false,
        };

        assert_eq!(context, expected);
    }

    #[test]
    fn set_line_number() {
        let mut builder = InstructionBuilder::default();

        builder.set_line_number(9);

        assert_eq!(builder.line_number, 9);
    }

    #[test]
    fn line_number_applies() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };

        builder.set_line_number(9);
        builder.instruction(instruction);

        assert_eq!(&builder.line_info[..], &[9]);
    }

    #[test]
    fn instruction() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };

        builder.instruction(instruction);

        assert_eq!(&builder.contexts[..], &[InstructionContext::new(instruction)]);
        assert_eq!(&builder.line_info[..], &[0]);
    }

    #[test]
    fn extra_instruction() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };

        builder.extra_instruction(instruction);

        assert_eq!(&builder.contexts[..], &[InstructionContext::new_extra(instruction)]);
        assert_eq!(&builder.line_info[..], &[0]);
    }

    #[test]
    fn insert_extra_instruction() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };
        let extra_instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(10) };

        builder.instruction(instruction);
        builder.set_line_number(9);
        builder.instruction(instruction);
        builder.insert_extra_instruction(1, extra_instruction);

        let expected = [
            InstructionContext::new(instruction),
            InstructionContext::new_extra(extra_instruction),
            InstructionContext::new(instruction),
        ];

        assert_eq!(&builder.contexts[..], &expected);
        assert_eq!(&builder.line_info[..], &[0, 9, 9]);
    }

    #[test]
    fn remove_instruction() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };
        let removed_instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(10) };

        builder.instruction(instruction);
        builder.instruction(removed_instruction);
        builder.instruction(instruction);
        builder.remove_instruction(1);

        let expected = [InstructionContext::new(instruction), InstructionContext {
            line_weight: -1,
            ..InstructionContext::new(instruction)
        }];

        assert_eq!(&builder.contexts[..], &expected);
        assert_eq!(&builder.line_info[..], &[0, 0]);
    }

    #[test]
    fn remove_extra_instruction() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };
        let removed_instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(10) };

        builder.instruction(instruction);
        builder.extra_instruction(removed_instruction);
        builder.instruction(instruction);
        builder.remove_instruction(1);

        let expected = [InstructionContext::new(instruction), InstructionContext::new(instruction)];
        assert_eq!(&builder.contexts[..], &expected);
        assert_eq!(&builder.line_info[..], &[0, 0]);
    }

    #[test]
    fn last_instruction_fixed() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };

        builder.instruction(instruction);
        builder.last_instruction_fixed();

        assert!(builder.contexts.last().unwrap().is_fixed);
    }

    #[test]
    fn last_instruction_offset() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::Jump { a: 0, mode: SignedBx(-4) };

        builder.instruction(instruction);
        builder.last_instruction_offset(-9);

        assert_eq!(builder.contexts.last().unwrap().final_offset, -9);
    }

    #[test]
    fn jump_destination_negative() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };

        builder.instruction(instruction);
        builder.extra_instruction(instruction);

        let result = builder.jump_destination(builder.get_program_counter() - 1, -1, 0);
        assert_eq!(result, Ok(-2));
    }

    #[test]
    fn jump_destination_zero() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };

        builder.extra_instruction(instruction);

        let result = builder.jump_destination(0, 0, 0);
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn jump_destination_positive() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };

        builder.extra_instruction(instruction);
        builder.instruction(instruction);
        builder.instruction(instruction);

        let result = builder.jump_destination(0, 1, 0);
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn jump_destination_offset_applies() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };

        builder.instruction(instruction);
        builder.instruction(instruction);

        let result = builder.jump_destination(builder.get_program_counter() - 1, -2, 2);
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn adjusted_jump_destination() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };

        builder.instruction(instruction);
        builder.extra_instruction(instruction);
        builder.instruction(instruction);

        assert_eq!(builder.adjusted_jump_destination(-2), Ok(0));
    }

    #[test]
    fn adjusted_jump_destination_positive() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };

        builder.instruction(instruction);
        builder.extra_instruction(instruction);
        builder.instruction(instruction);

        assert_eq!(builder.adjusted_jump_destination(1), Err(LunifyError::UnexpectedForwardJump));
    }

    #[test]
    fn finalize_expands_stack() -> Result<(), LunifyError> {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 10, mode: Bx(1) };
        let mut maxstacksize = 0;

        builder.instruction(instruction);
        builder.finalize(&mut maxstacksize, Default::default())?;

        assert_eq!(maxstacksize, 11);
        Ok(())
    }

    #[test]
    fn finalize_expands_stack_too_large() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 250, mode: Bx(1) };
        let mut maxstacksize = 0;

        builder.instruction(instruction);

        let result = builder.finalize(&mut maxstacksize, Default::default());
        assert_eq!(result, Result::Err(LunifyError::StackTooLarge(251)));
    }

    #[test]
    fn finalize_adjusts_jump_destinations() -> Result<(), LunifyError> {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };
        let jump_instruction = lua51::Instruction::Jump { a: 0, mode: SignedBx(-1) };

        builder.instruction(instruction);
        builder.extra_instruction(instruction);
        builder.extra_instruction(jump_instruction);
        let (instructions, _) = builder.finalize(&mut 0, Default::default())?;

        let lua51::Instruction::Jump { mode, .. } = instructions.last().unwrap() else {
            panic!()
        };
        assert_eq!(mode.0, -3);
        Ok(())
    }
}
