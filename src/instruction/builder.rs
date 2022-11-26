use super::Settings;
use crate::lua51::Instruction;
use crate::LunifyError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Metadata {
    instruction: Instruction,
    line_weight: i64,
    final_offset: i64,
    is_fixed: bool,
}

impl Metadata {
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
    instructions: Vec<Metadata>,
    line_info: Vec<i64>,
    line_number: i64,
}

impl InstructionBuilder {
    pub(super) fn set_line_number(&mut self, line_number: i64) {
        self.line_number = line_number;
    }

    pub(super) fn instruction(&mut self, instruction: Instruction) {
        self.instructions.push(Metadata::new(instruction));
        self.line_info.push(self.line_number);
    }

    pub(super) fn extra_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(Metadata::new_extra(instruction));
        self.line_info.push(self.line_number);
    }

    pub(super) fn insert_extra_instruction(&mut self, index: usize, instruction: Instruction) {
        let line_number = self.line_info[index];
        self.instructions.insert(index, Metadata::new_extra(instruction));
        self.line_info.insert(index, line_number);
    }

    pub(super) fn remove_instruction(&mut self, index: usize) {
        let removed = self.instructions.remove(index);
        self.line_info.remove(index);
        self.instructions[index].line_weight += removed.line_weight - 1;
    }

    pub(super) fn last_instruction_fixed(&mut self) {
        self.instructions.last_mut().unwrap().is_fixed = true;
    }

    pub(super) fn last_instruction_offset(&mut self, final_offset: i64) {
        self.instructions.last_mut().unwrap().final_offset = final_offset;
    }

    pub(super) fn get_instruction(&mut self, index: usize) -> &mut Instruction {
        &mut self.instructions[index].instruction
    }

    pub(super) fn get_program_counter(&self) -> usize {
        self.instructions.len()
    }

    pub(super) fn adjusted_jump_destination(&self, mut bx: i64) -> usize {
        let instruction_index = self.get_program_counter() - 1;

        let (mut steps, mut offset) = match bx.is_positive() {
            true => (bx + 1, 1),
            false => (bx.abs(), 0),
        };

        while steps != 0 {
            let index = match bx.is_positive() {
                true => instruction_index + offset,
                false => instruction_index - offset,
            };
            let instruction = &self.instructions[index];

            bx += instruction.line_weight * bx.signum();
            steps += instruction.line_weight - 1;
            offset += 1;
        }

        // TODO: figure out what to to if final_offset is != 0

        ((instruction_index as i64) + bx) as usize
    }

    pub(super) fn finalize(mut self, maxstacksize: &mut u8, settings: Settings) -> Result<(Vec<Instruction>, Vec<i64>), LunifyError> {
        let cloned = self.instructions.clone();

        for (instruction_index, instruction) in self.instructions.iter_mut().enumerate() {
            // The stack positions might have changed significantly, so go over every
            // instruction and make sure that the maxstacksize is big enough. If the stack
            // had to be popped out too much in the conversion, we return an error.
            // We also know that values on the stack will only be used after they have been
            // put there by anther instruction, meaning if we make space for the
            // instructions that push the values onto the stack, the stack will never
            // overflow.
            if let Some(destination) = instruction.instruction.stack_destination() {
                let new_stack_size = destination.end + 1;
                match new_stack_size <= settings.lua51.stack_limit {
                    true => *maxstacksize = (*maxstacksize).max(new_stack_size as u8),
                    false => return Err(LunifyError::StackTooLarge(new_stack_size)),
                }
            }

            let is_fixed = instruction.is_fixed;
            let final_offset = instruction.final_offset;

            // TODO: rework this code
            match &mut instruction.instruction {
                Instruction::Jump { mode, .. } | Instruction::ForLoop { mode, .. } | Instruction::ForPrep { mode, .. } if !is_fixed => {
                    let mut bx = mode.0;

                    let (mut steps, mut offset) = match bx.is_positive() {
                        true => (bx + 1, 1),
                        false => (bx.abs(), 0),
                    };

                    while steps != 0 {
                        let index = match bx.is_positive() {
                            true => instruction_index + offset,
                            false => instruction_index - offset,
                        };
                        let instruction = cloned[index];

                        bx += instruction.line_weight * bx.signum();
                        steps += instruction.line_weight - 1;
                        offset += 1;
                    }

                    bx += final_offset;
                    // TODO: Make sure that Bx is still in bounds.
                    mode.0 = bx;
                }
                _ => {}
            }

            #[cfg(feature = "debug")]
            {
                println!();
                println!("[{}] {:?}", instruction_index, instruction.instruction);
                println!(" -> {:?}", instruction.line_weight);
            }
        }

        let instructions = self.instructions.into_iter().map(|instruction| instruction.instruction).collect();
        Ok((instructions, self.line_info))
    }
}

#[cfg(test)]
mod tests {
    use super::InstructionBuilder;
    use crate::instruction::builder::Metadata;
    use crate::instruction::Bx;
    use crate::{lua51, LunifyError};

    #[test]
    fn metadata_new() {
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };
        let metadata = Metadata::new(instruction);
        let expected = Metadata {
            instruction,
            line_weight: 0,
            final_offset: 0,
            is_fixed: false,
        };

        assert_eq!(metadata, expected);
    }

    #[test]
    fn metadata_new_extra() {
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };
        let metadata = Metadata::new_extra(instruction);
        let expected = Metadata {
            instruction,
            line_weight: 1,
            final_offset: 0,
            is_fixed: false,
        };

        assert_eq!(metadata, expected);
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

        assert_eq!(&builder.instructions[..], &[Metadata::new(instruction)]);
        assert_eq!(&builder.line_info[..], &[0]);
    }

    #[test]
    fn extra_instruction() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 0, mode: Bx(1) };

        builder.extra_instruction(instruction);

        assert_eq!(&builder.instructions[..], &[Metadata::new_extra(instruction)]);
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
            Metadata::new(instruction),
            Metadata::new_extra(extra_instruction),
            Metadata::new(instruction),
        ];

        assert_eq!(&builder.instructions[..], &expected);
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

        let expected = [Metadata::new(instruction), Metadata {
            line_weight: -1,
            ..Metadata::new(instruction)
        }];

        assert_eq!(&builder.instructions[..], &expected);
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

        let expected = [Metadata::new(instruction), Metadata::new(instruction)];
        assert_eq!(&builder.instructions[..], &expected);
        assert_eq!(&builder.line_info[..], &[0, 0]);
    }

    #[test]
    fn finalize_expands_stack() -> Result<(), LunifyError> {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::LoadK { a: 10, mode: Bx(1) };
        builder.instruction(instruction);

        let mut maxstacksize = 0;
        builder.finalize(&mut maxstacksize, Default::default())?;

        assert_eq!(maxstacksize, 11);
        Ok(())
    }
}
