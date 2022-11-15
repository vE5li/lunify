use super::Settings;
use crate::lua51::{Instruction, Opcode};
use crate::LunifyError;

#[derive(Default)]
pub(super) struct InstructionBuilder {
    instructions: Vec<(Instruction, i64, i64)>,
    line_info: Vec<i64>,
    line_number: i64,
}

impl InstructionBuilder {
    pub(super) fn set_line_number(&mut self, line_number: i64) {
        self.line_number = line_number;
    }

    pub(super) fn instruction(&mut self, instruction: Instruction) {
        self.instructions.push((instruction, 0, 0));
        self.line_info.push(self.line_number);
    }

    pub(super) fn extra_instruction(&mut self, instruction: Instruction) {
        self.instructions.push((instruction, 1, 0));
        self.line_info.push(self.line_number);
    }

    pub(super) fn insert_extra_instruction(&mut self, index: usize, instruction: Instruction) {
        let line_number = self.line_info[index];
        self.instructions.insert(index, (instruction, 1, 0));
        self.line_info.insert(index, line_number);
    }

    pub(super) fn remove_instruction(&mut self, index: usize) {
        let removed = self.instructions.remove(index);
        self.line_info.remove(index);
        self.instructions[index].1 += removed.1 - 1;
    }

    pub(super) fn get_instruction(&mut self, index: usize) -> &mut Instruction {
        &mut self.instructions[index].0
    }

    pub(super) fn get_program_counter(&self) -> usize {
        self.instructions.len()
    }

    pub(super) fn set_final_offset(&mut self, index: usize, offset: i64) {
        self.instructions[index].2 = offset;
    }

    pub(super) fn adjusted_jump_destination(&self, bx: u64) -> usize {
        // TODO: rework to internally share code with the finalize function
        let instruction_index = self.get_program_counter() - 1;
        let mut direction = bx as i64 - 0b11111111111111111;
        let sign = direction.signum();

        let (mut steps, mut offset) = match direction.is_positive() {
            true => (direction + 1, 1),
            false => (direction.abs(), 0),
        };

        while steps != 0 {
            let index = match direction.is_positive() {
                true => instruction_index + offset,
                false => instruction_index - offset,
            };
            let (_, jump_offset, _) = self.instructions[index];

            direction += jump_offset * sign;
            steps += jump_offset - 1;
            offset += 1;
        }

        ((instruction_index as i64) + direction) as usize
    }

    pub(super) fn finalize(mut self, maxstacksize: &mut u8, settings: Settings) -> Result<(Vec<u64>, Vec<i64>), LunifyError> {
        for instruction_index in 0..self.instructions.len() {
            // The stack positions might have changed significantly, so go over every
            // instruction and make sure that the maxstacksize is big enough. If the stack
            // had to be popped out too much in the conversion, we return an error.
            // We also know that values on the stack will only be used after they have been
            // put there by anther instruction, meaning if we make space for the
            // instructions that push the values onto the stack, the stack will never
            // overflow.
            if let Some(destination) = self.instructions[instruction_index].0.stack_destination() {
                let new_stack_size = destination.end + 1;
                match new_stack_size <= settings.lua51.stack_limit {
                    true => *maxstacksize = (*maxstacksize).max(new_stack_size as u8),
                    false => return Err(LunifyError::StackTooLarge(new_stack_size)),
                }
            }

            // TODO: rework this code
            match self.instructions[instruction_index].0.opcode {
                Opcode::Jump | Opcode::ForLoop | Opcode::ForPrep => {
                    let mut bx = self.instructions[instruction_index].0.bx as i64;
                    // TODO: figure this out completely, sometimes the value comes out as 0 even
                    // though it should be one
                    // Maybe just add one to it?
                    let direction = bx - 0b11111111111111111;
                    let sign = direction.signum();

                    let (mut steps, mut offset) = match direction.is_positive() {
                        true => (direction + 1, 1),
                        false => (direction.abs(), 0),
                    };

                    while steps != 0 {
                        let index = match direction.is_positive() {
                            true => instruction_index + offset,
                            false => instruction_index - offset,
                        };
                        let (_, jump_offset, final_offset) = self.instructions[index];

                        bx += jump_offset * sign;
                        steps += jump_offset - 1;
                        offset += 1;

                        if steps == 0 {
                            // This final offset is needed because by design,ny inserted
                            // instructions will not change the instruction a jump lands on, only
                            // where that instruction in inside the binary. But sometimes we need
                            // to change the instruction a jump lands on, like when trying to save
                            // RA+3 before executing the FORLOOP instruction.
                            bx += final_offset;
                        }
                    }

                    // TODO: Make sure that Bx is still in bounds.
                    self.instructions[instruction_index].0.bx = bx as u64;

                    #[cfg(feature = "debug")]
                    println!("modified:  {:?}", self.instructions[instruction_index].0);
                }
                _ => {}
            }

            #[cfg(feature = "debug")]
            {
                let instruction = self.instructions[instruction_index];
                println!();
                println!("[{}] {:?}", instruction_index, instruction.0);
                println!(" -> {:?} (final: {})", instruction.1, instruction.2);
            }
        }

        let instructions = self.instructions.into_iter().map(|instruction| instruction.0.to_u64()).collect();
        Ok((instructions, self.line_info))
    }
}

#[cfg(test)]
mod tests {
    use super::InstructionBuilder;
    use crate::{lua51, LunifyError};

    #[test]
    fn set_line_number() {
        let mut builder = InstructionBuilder::default();

        builder.set_line_number(9);

        assert_eq!(builder.line_number, 9);
    }

    #[test]
    fn line_number_applies() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::new_bx(lua51::Opcode::LoadK, 0, 1);

        builder.set_line_number(9);
        builder.instruction(instruction);

        assert_eq!(&builder.line_info[..], &[9]);
    }

    #[test]
    fn instruction() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::new_bx(lua51::Opcode::LoadK, 0, 1);

        builder.instruction(instruction);

        assert_eq!(&builder.instructions[..], &[(instruction, 0, 0)]);
        assert_eq!(&builder.line_info[..], &[0]);
    }

    #[test]
    fn extra_instruction() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::new_bx(lua51::Opcode::LoadK, 0, 1);

        builder.extra_instruction(instruction);

        assert_eq!(&builder.instructions[..], &[(instruction, 1, 0)]);
        assert_eq!(&builder.line_info[..], &[0]);
    }

    #[test]
    fn insert_extra_instruction() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::new_bx(lua51::Opcode::LoadK, 0, 1);
        let inserted_instruction = lua51::Instruction::new_bx(lua51::Opcode::LoadK, 0, 10);

        builder.instruction(instruction);
        builder.set_line_number(9);
        builder.instruction(instruction);
        builder.insert_extra_instruction(1, inserted_instruction);

        let expected = [(instruction, 0, 0), (inserted_instruction, 1, 0), (instruction, 0, 0)];
        assert_eq!(&builder.instructions[..], &expected);
        assert_eq!(&builder.line_info[..], &[0, 9, 9]);
    }

    #[test]
    fn remove_instruction() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::new_bx(lua51::Opcode::LoadK, 0, 1);
        let removed_instruction = lua51::Instruction::new_bx(lua51::Opcode::LoadK, 0, 10);

        builder.instruction(instruction);
        builder.instruction(removed_instruction);
        builder.instruction(instruction);
        builder.remove_instruction(1);

        let expected = [(instruction, 0, 0), (instruction, -1, 0)];
        assert_eq!(&builder.instructions[..], &expected);
        assert_eq!(&builder.line_info[..], &[0, 0]);
    }

    #[test]
    fn remove_extra_instruction() {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::new_bx(lua51::Opcode::LoadK, 0, 1);
        let removed_instruction = lua51::Instruction::new_bx(lua51::Opcode::LoadK, 0, 10);

        builder.instruction(instruction);
        builder.extra_instruction(removed_instruction);
        builder.instruction(instruction);
        builder.remove_instruction(1);

        let expected = [(instruction, 0, 0), (instruction, 0, 0)];
        assert_eq!(&builder.instructions[..], &expected);
        assert_eq!(&builder.line_info[..], &[0, 0]);
    }

    #[test]
    fn finalize_expands_stack() -> Result<(), LunifyError> {
        let mut builder = InstructionBuilder::default();
        let instruction = lua51::Instruction::new_bx(lua51::Opcode::LoadK, 10, 1);
        builder.instruction(instruction);

        let mut maxstacksize = 0;
        builder.finalize(&mut maxstacksize, Default::default())?;

        assert_eq!(maxstacksize, 11);
        Ok(())
    }
}
