use super::builder::FunctionBuilder;
use crate::function::instruction::{Generic, LuaInstruction, BC};
use crate::{lua51, LunifyError, Settings};

pub(crate) fn convert(
    instructions: Vec<lua51::Instruction>,
    line_info: Vec<i64>,
    maximum_stack_size: &mut u8,
    settings: &Settings,
) -> Result<(Vec<lua51::Instruction>, Vec<i64>), LunifyError> {
    // If `fields_per_flush` is the same, there is nothing to convert, so return
    // early.
    if settings.lua51.fields_per_flush == settings.output.fields_per_flush {
        return Ok((instructions, line_info));
    }

    let mut builder = FunctionBuilder::default();

    for (instruction, line_number) in instructions.into_iter().zip(line_info.into_iter()) {
        #[cfg(feature = "debug")]
        println!("[{}] {:?}", builder.get_program_counter(), instruction);

        builder.set_line_number(line_number);

        match instruction {
            lua51::Instruction::SetList { a, mode: BC(b, c) } => {
                let flat_index = b.0 + (settings.lua51.fields_per_flush * (c.0 - 1));
                let page = flat_index / settings.output.fields_per_flush;
                let offset = flat_index % settings.output.fields_per_flush;

                // If b was 0 before, we need to keep it that way.
                let b = match b.0 {
                    0 => 0,
                    _ => offset,
                };

                // Good case: we are on the first page and the number of entries is smaller than
                // either `LFIELDS_PER_FLUSH`, meaning we can just insert a `SETLIST` instruction
                // without any modification to the previous code.
                if page == 0 && flat_index <= u64::min(settings.lua51.fields_per_flush, settings.output.fields_per_flush) {
                    builder.instruction(lua51::Instruction::SetList {
                        a,
                        mode: BC(Generic(b), Generic(1)),
                    });
                    continue;
                }

                // Go back until we find some instruction that moves data to a stack position
                // that is the same as our A, because that is where the setup starts.
                for instruction_index in (0..(builder.get_program_counter() - 1)).rev() {
                    let instruction = builder.get_instruction(instruction_index);

                    // It might technically be possible for the element on slot A to be on the stack
                    // already before any instructions if it is a parameter to a function call. So
                    // we make sure that at least the first instruction will always match.
                    // I am unsure that code like this can actually be emitted by the Lua compiler,
                    // because any assignment of a table should start with a `NEWTABLE` instruction,
                    // but better safe than sorry.
                    if matches!(instruction.stack_destination(), Some(destination) if destination.start == a) || instruction_index == 0 {
                        // Should either be `NEWTABLE` or `SETLIST`.
                        if let lua51::Instruction::SetList { mode: BC(b, c), .. } = *instruction {
                            let mut offset = b.0 as i64;
                            let mut page = c.0;

                            // Remove the `SETLIST` instruction.
                            builder.remove_instruction(instruction_index);

                            // Go back up the stack and update the stack positions.
                            let mut instruction_index = instruction_index;
                            while instruction_index < builder.get_program_counter() {
                                let instruction = builder.get_instruction(instruction_index);

                                if let Some(stack_destination) = instruction.stack_destination() {
                                    if offset + stack_destination.start as i64 - 1 == (a + settings.output.fields_per_flush) as i64 {
                                        // Add a new `SETLIST` instruction.
                                        builder.insert_extra_instruction(instruction_index, lua51::Instruction::SetList {
                                            a,
                                            mode: BC(Generic(settings.output.fields_per_flush), Generic(page)),
                                        });

                                        offset -= settings.output.fields_per_flush as i64;
                                        page += 1;
                                        instruction_index += 1;
                                        continue;
                                    }
                                }

                                builder.get_instruction(instruction_index).move_stack_accesses(a, offset);
                                instruction_index += 1;
                            }
                        }

                        break;
                    }
                }

                // Append the original instruction.
                builder.instruction(lua51::Instruction::SetList {
                    a,
                    mode: BC(Generic(b), Generic(page + 1)),
                });
            }
            instruction => builder.instruction(instruction),
        };
    }

    builder.finalize(maximum_stack_size, settings)
}

#[cfg(test)]
mod tests {
    use super::{lua51, BC};
    use crate::function::convert;
    use crate::function::instruction::{Bx, Generic, Unused};
    use crate::{lua50, LunifyError, Settings};

    fn test_settings() -> Settings<'static> {
        let lua50 = lua50::Settings::default();

        let lua51 = lua51::Settings {
            fields_per_flush: 5,
            ..lua51::Settings::default()
        };

        let output = lua51::Settings {
            fields_per_flush: 8,
            ..lua51::Settings::default()
        };

        Settings { lua50, lua51, output }
    }

    fn lua51_setlist(size: u64, settings: Settings) -> Vec<lua51::Instruction> {
        let mut instructions = vec![lua51::Instruction::NewTable {
            a: 0,
            mode: BC(Unused, Unused),
        }];

        for index in 0..size {
            let stack_position = (index % settings.lua51.fields_per_flush) + 1;
            let page = (index / settings.lua51.fields_per_flush) + 1;

            instructions.push(lua51::Instruction::LoadK {
                a: stack_position,
                mode: Bx(0),
            });

            if stack_position == settings.lua51.fields_per_flush || index + 1 == size {
                instructions.push(lua51::Instruction::SetList {
                    a: 0,
                    mode: BC(Generic(stack_position), Generic(page)),
                });
            }
        }

        instructions
    }

    fn output_setlist(size: u64, settings: Settings) -> Vec<lua51::Instruction> {
        let mut instructions = vec![lua51::Instruction::NewTable {
            a: 0,
            mode: BC(Unused, Unused),
        }];

        for index in 0..size {
            let stack_position = (index % settings.output.fields_per_flush) + 1;
            let page = (index / settings.output.fields_per_flush) + 1;

            instructions.push(lua51::Instruction::LoadK {
                a: stack_position,
                mode: Bx(0),
            });

            if stack_position == settings.output.fields_per_flush || index + 1 == size {
                instructions.push(lua51::Instruction::SetList {
                    a: 0,
                    mode: BC(Generic(stack_position), Generic(page)),
                });
            }
        }

        instructions
    }

    fn set_list_test(count: u64) -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = lua51_setlist(count, settings);
        let instruction_count = instructions.len();

        let (instructions, _) = convert(instructions, vec![0; instruction_count], &mut 2, &settings)?;
        let expected = output_setlist(count, settings);

        assert_eq!(instructions, expected);
        Ok(())
    }

    #[test]
    fn convert_set_list() -> Result<(), LunifyError> {
        set_list_test(4)
    }

    #[test]
    fn convert_set_list_bigger_than_50_flush() -> Result<(), LunifyError> {
        set_list_test(6)
    }

    #[test]
    fn convert_set_list_bigger_than_51_flush() -> Result<(), LunifyError> {
        set_list_test(9)
    }

    #[test]
    fn convert_set_list_large() -> Result<(), LunifyError> {
        set_list_test(20)
    }

    #[test]
    fn convert_set_list_from_parameters_bigger_than_50_flush() -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = vec![
            lua51::Instruction::LoadK { a: 5, mode: Bx(0) },
            lua51::Instruction::SetList {
                a: 0,
                mode: BC(Generic(5), Generic(1)),
            },
            lua51::Instruction::LoadK { a: 1, mode: Bx(0) },
            lua51::Instruction::SetList {
                a: 0,
                mode: BC(Generic(1), Generic(2)),
            },
        ];

        let (instructions, _) = convert(instructions, vec![0; 12], &mut 2, &settings)?;
        let expected = vec![
            lua51::Instruction::LoadK { a: 5, mode: Bx(0) },
            lua51::Instruction::LoadK { a: 6, mode: Bx(0) },
            lua51::Instruction::SetList {
                a: 0,
                mode: BC(Generic(6), Generic(1)),
            },
        ];

        assert_eq!(instructions, expected);
        Ok(())
    }
}
