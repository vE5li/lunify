#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::function::Constant;
use crate::stream::ByteStream;
use crate::LunifyError;

macro_rules! opcodes {
    ($($vname:ident,)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(super) enum Opcode {
            $($vname,)*
        }

        impl std::convert::TryFrom<u64> for Opcode {
            type Error = super::LunifyError;

            fn try_from(value: u64) -> Result<Self, Self::Error> {
                match value {
                    $(x if x == Opcode::$vname as u64 => Ok(Opcode::$vname),)*
                    _ => Err(Self::Error::InvalidOpcode(value)),
                }
            }
        }
    }
}

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

mod builder;
mod constant;
pub mod lua50;
pub mod lua51;

use builder::InstructionBuilder;

use self::constant::ConstantManager;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Settings {
    pub lua50: lua50::Settings,
    pub lua51: lua51::Settings,
}

pub(crate) fn upcast(
    instructions: Vec<lua50::Instruction>,
    line_info: Vec<i64>,
    constants: &mut Vec<Constant>,
    maxstacksize: &mut u8,
    parameter_count: u8,
    is_variadic: bool,
    settings: Settings,
) -> Result<(Vec<u64>, Vec<i64>), LunifyError> {
    let mut builder = InstructionBuilder::default();
    let mut constant_manager = ConstantManager { constants };

    for (instruction, line_number) in instructions.into_iter().zip(line_info.into_iter()) {
        let opcode = instruction.0 & 0b111111;
        let c = (instruction.0 >> 6) & 0b111111111;
        let b = (instruction.0 >> 15) & 0b111111111;
        let a = (instruction.0 >> 24) & 0b11111111;
        let bx = (instruction.0 >> 6) & 0b111111111111111111;
        let opcode = lua50::Opcode::try_from(opcode)?;

        #[cfg(feature = "debug")]
        {
            println!("=== input ===");
            println!("PC {}", builder.get_program_counter());
            println!("Opcode {:?}", opcode);
            println!("A {}", a);
            println!("B {}", b);
            println!("C {}", c);
            println!("BX {}", bx);
            println!("BX as i8 {}", bx as i8);
        }

        builder.set_line_number(line_number);

        match opcode {
            lua50::Opcode::Move => builder.instruction(lua51::Instruction::new(lua51::Opcode::Move, a, b, c)),
            lua50::Opcode::LoadK => builder.instruction(lua51::Instruction::new_bx(lua51::Opcode::LoadK, a, bx)),
            lua50::Opcode::LoadBool => builder.instruction(lua51::Instruction::new(lua51::Opcode::LoadBool, a, b, c)),
            lua50::Opcode::LoadNil => builder.instruction(lua51::Instruction::new(lua51::Opcode::LoadNil, a, b, c)),
            lua50::Opcode::GetUpValue => builder.instruction(lua51::Instruction::new(lua51::Opcode::GetUpValue, a, b, c)),
            lua50::Opcode::GetGlobal => builder.instruction(lua51::Instruction::new(lua51::Opcode::GetGlobal, a, b, c)),
            lua50::Opcode::GetTable => builder.instruction(lua51::Instruction::new(lua51::Opcode::GetTable, a, b, c)),
            lua50::Opcode::SetGlobal => builder.instruction(lua51::Instruction::new(lua51::Opcode::SetGlobal, a, b, c)),
            lua50::Opcode::SetUpValue => builder.instruction(lua51::Instruction::new(lua51::Opcode::SetUpValue, a, b, c)),
            lua50::Opcode::SetTable => builder.instruction(lua51::Instruction::new(lua51::Opcode::SetTable, a, b, c)),
            lua50::Opcode::NewTable => builder.instruction(lua51::Instruction::new(lua51::Opcode::NewTable, a, b, c)),
            lua50::Opcode::_Self => builder.instruction(lua51::Instruction::new(lua51::Opcode::_Self, a, b, c)),
            lua50::Opcode::Add => builder.instruction(lua51::Instruction::new(lua51::Opcode::Add, a, b, c)),
            lua50::Opcode::Subtract => builder.instruction(lua51::Instruction::new(lua51::Opcode::Subtract, a, b, c)),
            lua50::Opcode::Multiply => builder.instruction(lua51::Instruction::new(lua51::Opcode::Multiply, a, b, c)),
            lua50::Opcode::Divide => builder.instruction(lua51::Instruction::new(lua51::Opcode::Divide, a, b, c)),
            lua50::Opcode::Power => builder.instruction(lua51::Instruction::new(lua51::Opcode::Power, a, b, c)),
            lua50::Opcode::Unary => builder.instruction(lua51::Instruction::new(lua51::Opcode::Unary, a, b, c)),
            lua50::Opcode::Not => builder.instruction(lua51::Instruction::new(lua51::Opcode::Not, a, b, c)),
            lua50::Opcode::Concatinate => builder.instruction(lua51::Instruction::new(lua51::Opcode::Concatinate, a, b, c)),
            lua50::Opcode::Jump => builder.instruction(lua51::Instruction::new_bx(lua51::Opcode::Jump, a, bx)),
            lua50::Opcode::Equals => builder.instruction(lua51::Instruction::new(lua51::Opcode::Equals, a, b, c)),
            lua50::Opcode::LessThan => builder.instruction(lua51::Instruction::new(lua51::Opcode::LessThan, a, b, c)),
            lua50::Opcode::LessEquals => builder.instruction(lua51::Instruction::new(lua51::Opcode::LessEquals, a, b, c)),
            // Test maps to TestSet.
            lua50::Opcode::Test => builder.instruction(lua51::Instruction::new(lua51::Opcode::TestSet, a, b, c)),
            lua50::Opcode::Call => builder.instruction(lua51::Instruction::new(lua51::Opcode::Call, a, b, c)),
            lua50::Opcode::TailCall => builder.instruction(lua51::Instruction::new(lua51::Opcode::TailCall, a, b, c)),
            lua50::Opcode::Return => builder.instruction(lua51::Instruction::new(lua51::Opcode::Return, a, b, c)),
            lua50::Opcode::ForLoop => {
                // Lua 5.1 additionally saves the loop index in RA+3, which Lua 5.0 does
                // not. Therefore we save RA+3 to a global value and restore it afterwards.

                // Create a new constant to hold an idendifier to the global that saves the
                // value in RA+3.
                let global_constant = constant_manager.create_unique(builder.get_program_counter());

                // Instruction to save RA+3.
                builder.instruction(lua51::Instruction::new_bx(lua51::Opcode::SetGlobal, a + 3, global_constant));

                // Original instruction, but since we will insert another instruction before the
                // destination of our jump, we also pass it an offset that will be applied after
                // adjusting the jump position.
                builder.extra_instruction(lua51::Instruction::new_bx_offset(lua51::Opcode::ForLoop, a, bx, -1));

                // Get the *adjusted* position of the instruction we want to
                // jump to. It is very important that we take the adjusted position because
                // we might have added or remove instructions inside the for loop, which would
                // make the old Bx invalid.
                let position = builder.adjusted_jump_destination(bx) + 1;

                // Instruction to restore RA+3 if we take the jump.
                // This instruction is actually inserted *before* the SETGLOBAL instruction, but
                // this works becaues of the way that for loops are generated in
                // Lua 5.0. There is an initial JMP instruction that moves the
                // program counter to the FORLOOP instruction, meaning this
                // GetGlobal will *always* be called after we already saved RA+3
                // with our SETGLOBAL instruction.
                builder.insert_extra_instruction(
                    position,
                    lua51::Instruction::new_bx(lua51::Opcode::GetGlobal, a + 3, global_constant),
                );
            }
            lua50::Opcode::TForLoop => {
                // The TFORLOOP instruction in Lua 5.0 can move multiple results to the stack
                // below the callbase, but Lua 5.1 can only move one result and
                // the subsequent moves are done by MOVE instructions.

                // If the argument count is 1 (argument count = c - 1), we can just map directly
                // to Lua 5.1 TFORLOOP.
                if c == 0 {
                    builder.instruction(lua51::Instruction::new(lua51::Opcode::TForLoop, a, b, c + 1));
                } else {
                    // In order to mimic the Lua 5.0 TFORLOOP instruction we can't insert MOVE
                    // instructions after the TFORLOOP jump like Lua 5.1 does, because that won't
                    // move our results to the stack after the last iteration.

                    let variable_count = c + 1;
                    let call_base = a + variable_count + 2;
                    let constant_nil = constant_manager.constant_nil();

                    // Move the iterator function, the table and the index to our call base.
                    builder.instruction(lua51::Instruction::new(lua51::Opcode::Move, call_base, a, 0));
                    builder.extra_instruction(lua51::Instruction::new(lua51::Opcode::Move, call_base + 1, a + 1, 0));
                    builder.extra_instruction(lua51::Instruction::new(lua51::Opcode::Move, call_base + 2, a + 2, 0));

                    // Call to iterator function (e.g. ipairs).
                    builder.extra_instruction(lua51::Instruction::new(lua51::Opcode::Call, call_base, 3, variable_count + 1));

                    // Move the results of our call back to our control variables. After the call,
                    // our results will be at the call_base and upwards and our control variables
                    // are located at A+2 and upwards.
                    for offset in (0..variable_count).rev() {
                        builder.extra_instruction(lua51::Instruction::new(
                            lua51::Opcode::Move,
                            a + offset + 2,
                            call_base + offset,
                            0,
                        ));
                    }

                    // The control variable for the key/index is located at A+2, so as soon as it
                    // is nil, we are done with the iteration. If it is not nil we jump back and
                    // iterate again. It's not obvious from the code here but following this
                    // instruction will always be a JMP instruction that specifies the destination
                    // of the jump. That JMP instruction doesn't need any modification here.
                    builder.extra_instruction(lua51::Instruction::new(
                        lua51::Opcode::Equals,
                        0,
                        a + 2,
                        constant_nil | (1 << (8)),
                    ));
                }
            }
            lua50::Opcode::TForPrep => {
                // Globals for saving RA+1 and RA+2.
                let ra1_constant = constant_manager.create_unique(builder.get_program_counter());
                let ra2_constant = constant_manager.create_unique(builder.get_program_counter() + 1);

                let type_global_constant = constant_manager.constant_for_str("type");
                let table_global_constant = constant_manager.constant_for_str("table");
                let next_global_constant = constant_manager.constant_for_str("next");

                // Instructions to save RA+1 and RA+2.
                builder.instruction(lua51::Instruction::new_bx(lua51::Opcode::SetGlobal, a + 1, ra1_constant));
                builder.extra_instruction(lua51::Instruction::new_bx(lua51::Opcode::SetGlobal, a + 2, ra2_constant));

                // Prepare arguments and call the "type" function on the value in RA.
                builder.extra_instruction(lua51::Instruction::new_bx(
                    lua51::Opcode::GetGlobal,
                    a + 1,
                    type_global_constant,
                ));
                builder.extra_instruction(lua51::Instruction::new(lua51::Opcode::Move, a + 2, a, 0));
                builder.extra_instruction(lua51::Instruction::new(lua51::Opcode::Call, a + 1, 2, 2));

                // Load the string "table" to compare the result of the previous type to.
                builder.extra_instruction(lua51::Instruction::new_bx(lua51::Opcode::LoadK, a + 2, table_global_constant));

                // If it's not a table we want to restore RA+1 and RA+2, so we jump to that
                // instruction.
                builder.extra_instruction(lua51::Instruction::new(lua51::Opcode::Equals, 0, a + 1, a + 2));
                // TODO: remove this
                let signed_offset = 0b11111111111111111;
                // Because of the way the builder works, the jump destination in Bx would be
                // moved when re-emitting the instructions. Therefore we use
                // new_bx_fixed so we land on the correct instruction.
                builder.extra_instruction(lua51::Instruction::new_bx_fixed(lua51::Opcode::Jump, a, 2 + signed_offset));

                // Move RA to RA+1 and put the global "next" into RA, exactly like TForPrep
                // does. Since we restore RA+1 from ra1_constant afterwards, we don't move the
                // vaule to the stack directly but rather to ra1_constant.
                builder.extra_instruction(lua51::Instruction::new_bx(lua51::Opcode::SetGlobal, a, ra1_constant));
                builder.extra_instruction(lua51::Instruction::new_bx(lua51::Opcode::GetGlobal, a, next_global_constant));

                // Restore RA+1 and RA+2.
                builder.extra_instruction(lua51::Instruction::new_bx(lua51::Opcode::GetGlobal, a + 1, ra1_constant));
                builder.extra_instruction(lua51::Instruction::new_bx(lua51::Opcode::GetGlobal, a + 2, ra2_constant));

                // Original instruction.
                // Technially this Jump could be removed if it lands on the very next
                // instruction, which will happen it the next instruction is a
                // TForLoop. But I think it's better to keep this here for
                // simplicity.
                builder.extra_instruction(lua51::Instruction::new_bx(lua51::Opcode::Jump, a, bx));
            }
            lua50::Opcode::SetList => {
                let flat_index = bx + 1;
                let page = flat_index / settings.lua51.fields_per_flush;
                let offset = flat_index % settings.lua51.fields_per_flush;

                // Good case: we are on the first page and the number of entries is smaller than
                // either LFIELDS_PER_FLUSH, meaning we can just insert a
                // SETLIST instruction without any modification to
                // the previous code.
                if page == 0 && flat_index <= u64::min(settings.lua50.fields_per_flush, settings.lua51.fields_per_flush) {
                    builder.instruction(lua51::Instruction::new(lua51::Opcode::SetList, a, flat_index, 1));
                    continue;
                }

                // Go back until we find some instruction that moves data to a stack position
                // that is the same as our A, because that is where the setup starts.
                for instruction_index in (0..(builder.get_program_counter() - 1)).rev() {
                    let instruction = builder.get_instruction(instruction_index);

                    // It might technially be possible for the element on slot A to be on the stack
                    // already before any instructions if it is a parameter to a function call. So
                    // we make sure that at least the first instruction will always match.
                    // I am unsure that code like this can actually be emitted by the Lua compiler,
                    // because any assignment of a table should start with a NEWTABLE instruction,
                    // but better safe than sorry.
                    if matches!(instruction.stack_destination(), Some(destination) if destination.start == a) || instruction_index == 0 {
                        // Should either be NEWTABLE or SETLIST.
                        if instruction.opcode == lua51::Opcode::SetList {
                            let mut offset = instruction.b as i64;
                            let mut page = instruction.c;

                            // Remove the SETLIST instruction.
                            builder.remove_instruction(instruction_index);

                            // Go back up the stack and update the stack positions.
                            let mut instruction_index = instruction_index;
                            while instruction_index < builder.get_program_counter() {
                                let instruction = builder.get_instruction(instruction_index);

                                if let Some(stack_destination) = instruction.stack_destination() {
                                    if offset + stack_destination.start as i64 - 1 == (a + settings.lua51.fields_per_flush) as i64 {
                                        // Add a new SETLIST instruction.
                                        let instruction =
                                            lua51::Instruction::new(lua51::Opcode::SetList, a, settings.lua51.fields_per_flush, page);
                                        builder.insert_extra_instruction(instruction_index, instruction);

                                        offset -= settings.lua51.fields_per_flush as i64;
                                        page += 1;
                                        instruction_index += 1;
                                        continue;
                                    }
                                }

                                builder.get_instruction(instruction_index).move_stack_accesses(offset);
                                instruction_index += 1;
                            }
                        }

                        break;
                    }
                }

                // Append the original instruction.
                builder.instruction(lua51::Instruction::new(lua51::Opcode::SetList, a, offset, page + 1))
            }
            // TODO: Pretty sure that this is correct but validate anyway.
            lua50::Opcode::SetListO => builder.instruction(lua51::Instruction::new(lua51::Opcode::SetList, a, 0, c)),
            lua50::Opcode::Close => builder.instruction(lua51::Instruction::new(lua51::Opcode::Close, a, b, c)),
            lua50::Opcode::Closure => builder.instruction(lua51::Instruction::new(lua51::Opcode::Closure, a, b, c)),
        };
    }

    // Lua 5.0 used to collect variadic arguments in a table and store them in a
    // local variable 'args'. Lua 5.1 does things a bit differently, so for
    // variadic functions we insert instructions that are the eqivalent of
    // 'local args = {...}'. Since we are at the very beginning of our function
    // call, we don't need to worry about saving the stack above our
    // arguments.
    // TODO: find out how much of this we actually need, since Lua 5.1 seems to be
    // able to use "arg" inside functions.
    if is_variadic {
        let arg_stack_position = parameter_count as u64;

        // Create a new empty table to hold our arguments.
        builder.insert_extra_instruction(
            0,
            lua51::Instruction::new(lua51::Opcode::NewTable, arg_stack_position + 1, 0, 0),
        );

        // Push all variadic arguments onto the stack.
        builder.insert_extra_instruction(1, lua51::Instruction::new_bx(lua51::Opcode::VarArg, arg_stack_position + 2, 0));

        // Add all values from the stack to the table.
        builder.insert_extra_instruction(2, lua51::Instruction::new(lua51::Opcode::SetList, arg_stack_position + 1, 0, 1));

        // Move the table to the location of the argument.
        builder.insert_extra_instruction(
            3,
            lua51::Instruction::new(lua51::Opcode::Move, arg_stack_position, arg_stack_position + 1, 0),
        );
    }

    builder.finalize(maxstacksize, settings)
}

#[cfg(test)]
mod tests {
    use super::{lua50, lua51};
    use crate::instruction::{upcast, Settings};
    use crate::LunifyError;

    fn test_settings() -> Settings {
        let lua50 = lua50::Settings { fields_per_flush: 5 };
        let lua51 = lua51::Settings {
            fields_per_flush: 8,
            ..lua51::Settings::default()
        };
        Settings { lua50, lua51 }
    }

    #[test]
    fn upcast_set_list() -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = vec![
            lua50::Instruction::new(lua50::Opcode::NewTable, 0, 0, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 1, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 2, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 3, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 4, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 3),
        ];

        let previous_size = instructions.len();
        let (instructions, _) = upcast(instructions, vec![0; 6], &mut Vec::new(), &mut 2, 0, false, settings)?;
        let expected = lua51::Instruction::new(lua51::Opcode::SetList, 0, 4, 1).to_u64();

        assert_eq!(instructions.last().cloned().unwrap(), expected);
        assert_eq!(instructions.len(), previous_size);

        Ok(())
    }

    #[test]
    fn upcast_set_list_bigger_than_50_flush() -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = vec![
            lua50::Instruction::new(lua50::Opcode::NewTable, 0, 0, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 1, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 2, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 3, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 4, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 5, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 4),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 1, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 5),
        ];

        let previous_size = instructions.len();
        let (instructions, _) = upcast(instructions, vec![0; 9], &mut Vec::new(), &mut 2, 0, false, settings)?;
        let expected = lua51::Instruction::new(lua51::Opcode::SetList, 0, 6, 1).to_u64();

        assert_eq!(instructions.last().cloned().unwrap(), expected);
        assert_eq!(instructions.len(), previous_size - 1);

        Ok(())
    }

    #[test]
    fn upcast_set_list_bigger_than_51_flush() -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = vec![
            lua50::Instruction::new(lua50::Opcode::NewTable, 0, 0, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 1, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 2, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 3, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 4, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 5, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 4),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 1, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 2, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 3, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 4, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 8),
        ];

        let previous_size = instructions.len();
        let (instructions, _) = upcast(instructions, vec![0; 12], &mut Vec::new(), &mut 2, 0, false, settings)?;
        let expected = lua51::Instruction::new(lua51::Opcode::SetList, 0, 1, 2).to_u64();

        assert_eq!(instructions.last().cloned().unwrap(), expected);
        assert_eq!(instructions.len(), previous_size);

        Ok(())
    }

    #[test]
    fn upcast_set_list_large() -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = vec![
            lua50::Instruction::new(lua50::Opcode::NewTable, 0, 0, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 1, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 2, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 3, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 4, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 5, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 4),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 1, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 2, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 3, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 4, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 5, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 9),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 1, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 2, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 3, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 4, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 5, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 14),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 1, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 2, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 3, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 4, 0),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 5, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 19),
        ];

        let previous_size = instructions.len();
        let (instructions, _) = upcast(instructions, vec![0; 25], &mut Vec::new(), &mut 2, 0, false, settings)?;
        let expected = lua51::Instruction::new(lua51::Opcode::SetList, 0, 4, 3).to_u64();

        assert_eq!(instructions.last().cloned().unwrap(), expected);
        assert_eq!(instructions.len(), previous_size - 1);

        Ok(())
    }

    #[test]
    fn upcast_set_list_from_parameters() -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = vec![
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 5, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 4),
        ];

        let previous_size = instructions.len();
        let (instructions, _) = upcast(instructions, vec![0; 12], &mut Vec::new(), &mut 2, 0, false, settings)?;

        let expected = vec![
            lua51::Instruction::new_bx(lua51::Opcode::LoadK, 5, 0).to_u64(),
            lua51::Instruction::new(lua51::Opcode::SetList, 0, 5, 1).to_u64(),
        ];

        assert_eq!(instructions, expected);
        assert_eq!(instructions.len(), previous_size);

        Ok(())
    }

    #[test]
    fn upcast_set_list_from_parameters_bigger_than_50_flush() -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = vec![
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 5, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 4),
            lua50::Instruction::new_bx(lua50::Opcode::LoadK, 1, 0),
            lua50::Instruction::new_bx(lua50::Opcode::SetList, 0, 5),
        ];

        let previous_size = instructions.len();
        let (instructions, _) = upcast(instructions, vec![0; 12], &mut Vec::new(), &mut 2, 0, false, settings)?;

        let expected = vec![
            lua51::Instruction::new_bx(lua51::Opcode::LoadK, 5, 0).to_u64(),
            lua51::Instruction::new_bx(lua51::Opcode::LoadK, 6, 0).to_u64(),
            lua51::Instruction::new(lua51::Opcode::SetList, 0, 6, 1).to_u64(),
        ];

        assert_eq!(instructions, expected);
        assert_eq!(instructions.len(), previous_size - 1);

        Ok(())
    }
}
