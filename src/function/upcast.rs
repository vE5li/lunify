use super::builder::InstructionBuilder;
use super::constant::{Constant, ConstantManager};
use super::instruction::{lua50, lua51, Bx, Settings, SignedBx, BC};
use crate::LunifyError;

pub(crate) fn upcast(
    instructions: Vec<lua50::Instruction>,
    line_info: Vec<i64>,
    constants: &mut Vec<Constant>,
    maxstacksize: &mut u8,
    parameter_count: u8,
    is_variadic: bool,
    settings: Settings,
) -> Result<(Vec<lua51::Instruction>, Vec<i64>), LunifyError> {
    let mut builder = InstructionBuilder::default();
    let mut constant_manager = ConstantManager { constants };

    #[cfg(feature = "debug")]
    println!("\n======== input ========");

    for (instruction, line_number) in instructions.into_iter().zip(line_info.into_iter()) {
        #[cfg(feature = "debug")]
        println!("[{}] {:?}", builder.get_program_counter(), instruction);

        builder.set_line_number(line_number);

        match instruction {
            lua50::Instruction::Move { a, mode } => builder.instruction(lua51::Instruction::Move { a, mode }),
            lua50::Instruction::LoadK { a, mode } => builder.instruction(lua51::Instruction::LoadK { a, mode }),
            lua50::Instruction::LoadBool { a, mode } => builder.instruction(lua51::Instruction::LoadBool { a, mode }),
            lua50::Instruction::LoadNil { a, mode } => builder.instruction(lua51::Instruction::LoadNil { a, mode }),
            lua50::Instruction::GetUpValue { a, mode } => builder.instruction(lua51::Instruction::GetUpValue { a, mode }),
            lua50::Instruction::GetGlobal { a, mode } => builder.instruction(lua51::Instruction::GetGlobal { a, mode }),
            lua50::Instruction::GetTable { a, mode } => builder.instruction(lua51::Instruction::GetTable { a, mode }),
            lua50::Instruction::SetGlobal { a, mode } => builder.instruction(lua51::Instruction::SetGlobal { a, mode }),
            lua50::Instruction::SetUpValue { a, mode } => builder.instruction(lua51::Instruction::SetUpValue { a, mode }),
            lua50::Instruction::SetTable { a, mode } => builder.instruction(lua51::Instruction::SetTable { a, mode }),
            lua50::Instruction::NewTable { a, mode } => builder.instruction(lua51::Instruction::NewTable { a, mode }),
            lua50::Instruction::_Self { a, mode } => builder.instruction(lua51::Instruction::_Self { a, mode }),
            lua50::Instruction::Add { a, mode } => builder.instruction(lua51::Instruction::Add { a, mode }),
            lua50::Instruction::Subtract { a, mode } => builder.instruction(lua51::Instruction::Subtract { a, mode }),
            lua50::Instruction::Multiply { a, mode } => builder.instruction(lua51::Instruction::Multiply { a, mode }),
            lua50::Instruction::Divide { a, mode } => builder.instruction(lua51::Instruction::Divide { a, mode }),
            lua50::Instruction::Power { a, mode } => builder.instruction(lua51::Instruction::Power { a, mode }),
            lua50::Instruction::Unary { a, mode } => builder.instruction(lua51::Instruction::Unary { a, mode }),
            lua50::Instruction::Not { a, mode } => builder.instruction(lua51::Instruction::Not { a, mode }),
            lua50::Instruction::Concatinate { a, mode } => builder.instruction(lua51::Instruction::Concatinate { a, mode }),
            lua50::Instruction::Jump { a, mode } => builder.instruction(lua51::Instruction::Jump { a, mode }),
            lua50::Instruction::Equals { a, mode } => builder.instruction(lua51::Instruction::Equals { a, mode }),
            lua50::Instruction::LessThan { a, mode } => builder.instruction(lua51::Instruction::LessThan { a, mode }),
            lua50::Instruction::LessEquals { a, mode } => builder.instruction(lua51::Instruction::LessEquals { a, mode }),
            // Test maps to TestSet.
            lua50::Instruction::Test { a, mode } => builder.instruction(lua51::Instruction::TestSet { a, mode }),
            lua50::Instruction::Call { a, mode } => builder.instruction(lua51::Instruction::Call { a, mode }),
            lua50::Instruction::TailCall { a, mode } => builder.instruction(lua51::Instruction::TailCall { a, mode }),
            lua50::Instruction::Return { a, mode } => builder.instruction(lua51::Instruction::Return { a, mode }),
            lua50::Instruction::ForLoop { a, mode } => {
                // Lua 5.1 additionally saves the loop index in RA+3, which Lua 5.0 does
                // not. Therefore we save RA+3 to a global value and restore it afterwards.

                // Create a new constant to hold an idendifier to the global that saves the
                // value in RA+3.
                let global_constant = constant_manager.create_unique(builder.get_program_counter());

                // Instruction to save RA+3.
                builder.instruction(lua51::Instruction::SetGlobal {
                    a: a + 3,
                    mode: Bx(global_constant),
                });

                // Original instruction, but since we will insert another instruction before the
                // destination of our jump, we also pass it an offset that will be applied after
                // adjusting the jump position.
                builder.extra_instruction(lua51::Instruction::ForLoop { a, mode });
                builder.last_instruction_offset(-1);

                // Get the *adjusted* position of the instruction we want to
                // jump to. It is very important that we take the adjusted position because
                // we might have added or remove instructions inside the for loop, which would
                // make the old Bx invalid.
                let position = builder.adjusted_jump_destination(mode.0)?;

                // Instruction to restore RA+3 if we take the jump.
                // This instruction is actually inserted *before* the SETGLOBAL instruction, but
                // this works becaues of the way that for loops are generated in
                // Lua 5.0. There is an initial JMP instruction that moves the
                // program counter to the FORLOOP instruction, meaning this
                // GetGlobal will *always* be called after we already saved RA+3
                // with our SETGLOBAL instruction.
                builder.insert_extra_instruction(position, lua51::Instruction::GetGlobal {
                    a: a + 3,
                    mode: Bx(global_constant),
                });
            }
            lua50::Instruction::TForLoop { a, mode: BC(b, c) } => {
                // The TFORLOOP instruction in Lua 5.0 can move multiple results to the stack
                // below the callbase, but Lua 5.1 can only move one result and
                // the subsequent moves are done by MOVE instructions.

                // If the argument count is 1 (argument count = c - 1), we can just map directly
                // to Lua 5.1 TFORLOOP.
                if c == 0 {
                    builder.instruction(lua51::Instruction::TForLoop { a, mode: BC(b, c + 1) });
                } else {
                    // In order to mimic the Lua 5.0 TFORLOOP instruction we can't insert MOVE
                    // instructions after the TFORLOOP jump like Lua 5.1 does, because that won't
                    // move our results to the stack after the last iteration.

                    let variable_count = c + 1;
                    let call_base = a + variable_count + 2;
                    let constant_nil = constant_manager.constant_nil();

                    // Move the iterator function, the table and the index to our call base.
                    builder.instruction(lua51::Instruction::Move {
                        a: call_base,
                        mode: BC(a, 0),
                    });
                    builder.extra_instruction(lua51::Instruction::Move {
                        a: call_base + 1,
                        mode: BC(a + 1, 0),
                    });
                    builder.extra_instruction(lua51::Instruction::Move {
                        a: call_base + 2,
                        mode: BC(a + 2, 0),
                    });

                    // Call to iterator function (e.g. ipairs).
                    builder.extra_instruction(lua51::Instruction::Call {
                        a: call_base,
                        mode: BC(3, variable_count + 1),
                    });

                    // Move the results of our call back to our control variables. After the call,
                    // our results will be at the call_base and upwards and our control variables
                    // are located at A+2 and upwards.
                    for offset in (0..variable_count).rev() {
                        builder.extra_instruction(lua51::Instruction::Move {
                            a: a + offset + 2,
                            mode: BC(call_base + offset, 0),
                        });
                    }

                    // The control variable for the key/index is located at A+2, so as soon as it
                    // is nil, we are done with the iteration. If it is not nil we jump back and
                    // iterate again. It's not obvious from the code here but following this
                    // instruction will always be a JMP instruction that specifies the destination
                    // of the jump. That JMP instruction doesn't need any modification here.
                    builder.extra_instruction(lua51::Instruction::Equals {
                        a: 0,
                        mode: BC::const_c(a + 2, constant_nil),
                    });
                }
            }
            lua50::Instruction::TForPrep { a, mode } => {
                // Globals for saving RA+1 and RA+2.
                let ra1_constant = constant_manager.create_unique(builder.get_program_counter());
                let ra2_constant = constant_manager.create_unique(builder.get_program_counter() + 1);

                let type_global_constant = constant_manager.constant_for_str("type");
                let table_global_constant = constant_manager.constant_for_str("table");
                let next_global_constant = constant_manager.constant_for_str("next");

                // Instructions to save RA+1 and RA+2.
                builder.instruction(lua51::Instruction::SetGlobal {
                    a: a + 1,
                    mode: Bx(ra1_constant),
                });
                builder.extra_instruction(lua51::Instruction::SetGlobal {
                    a: a + 2,
                    mode: Bx(ra2_constant),
                });

                // Prepare arguments and call the "type" function on the value in RA.
                builder.extra_instruction(lua51::Instruction::GetGlobal {
                    a: a + 1,
                    mode: Bx(type_global_constant),
                });
                builder.extra_instruction(lua51::Instruction::Move { a: a + 2, mode: BC(a, 0) });
                builder.extra_instruction(lua51::Instruction::Call { a: a + 1, mode: BC(2, 2) });

                // Load the string "table" to compare the result of the previous type to.
                builder.extra_instruction(lua51::Instruction::LoadK {
                    a: a + 2,
                    mode: Bx(table_global_constant),
                });

                // If it's not a table we want to restore RA+1 and RA+2, so we jump to that
                // instruction.
                builder.extra_instruction(lua51::Instruction::Equals {
                    a: 0,
                    mode: BC(a + 1, a + 2),
                });
                // Because of the way the builder works, the jump destination in Bx would be
                // moved when re-emitting the instructions. Therefore we use
                // new_bx_fixed so we land on the correct instruction.
                builder.extra_instruction(lua51::Instruction::Jump { a, mode: SignedBx(2) });
                builder.last_instruction_fixed();

                // Move RA to RA+1 and put the global "next" into RA, exactly like TForPrep
                // does. Since we restore RA+1 from ra1_constant afterwards, we don't move the
                // vaule to the stack directly but rather to ra1_constant.
                builder.extra_instruction(lua51::Instruction::SetGlobal { a, mode: Bx(ra1_constant) });
                builder.extra_instruction(lua51::Instruction::GetGlobal {
                    a,
                    mode: Bx(next_global_constant),
                });

                // Restore RA+1 and RA+2.
                builder.extra_instruction(lua51::Instruction::GetGlobal {
                    a: a + 1,
                    mode: Bx(ra1_constant),
                });
                builder.extra_instruction(lua51::Instruction::GetGlobal {
                    a: a + 2,
                    mode: Bx(ra2_constant),
                });

                // Original instruction.
                // Technially this Jump could be removed if it lands on the very next
                // instruction, which will happen it the next instruction is a
                // TForLoop. But I think it's better to keep this here for
                // simplicity.
                builder.extra_instruction(lua51::Instruction::Jump { a, mode });
            }
            lua50::Instruction::SetList { a, mode: Bx(bx) } | lua50::Instruction::SetListO { a, mode: Bx(bx) } => {
                let flat_index = bx + 1;
                let page = flat_index / settings.lua51.fields_per_flush;
                let offset = flat_index % settings.lua51.fields_per_flush;

                // In Lua 5.1 SETLISTO and SETLIST became a single instruction. The behaviour
                // of SETLISTO is used when b is equal to zero.
                let b = match matches!(instruction, lua50::Instruction::SetListO { .. }) {
                    true => 0,
                    false => offset,
                };

                // Good case: we are on the first page and the number of entries is smaller than
                // either LFIELDS_PER_FLUSH, meaning we can just insert a
                // SETLIST instruction without any modification to
                // the previous code.
                if page == 0 && flat_index <= u64::min(settings.lua50.fields_per_flush, settings.lua51.fields_per_flush) {
                    builder.instruction(lua51::Instruction::SetList { a, mode: BC(b, 1) });
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
                        if let lua51::Instruction::SetList { mode: BC(b, c), .. } = *instruction {
                            let mut offset = b as i64;
                            let mut page = c;

                            // Remove the SETLIST instruction.
                            builder.remove_instruction(instruction_index);

                            // Go back up the stack and update the stack positions.
                            let mut instruction_index = instruction_index;
                            while instruction_index < builder.get_program_counter() {
                                let instruction = builder.get_instruction(instruction_index);

                                if let Some(stack_destination) = instruction.stack_destination() {
                                    if offset + stack_destination.start as i64 - 1 == (a + settings.lua51.fields_per_flush) as i64 {
                                        // Add a new SETLIST instruction.
                                        builder.insert_extra_instruction(instruction_index, lua51::Instruction::SetList {
                                            a,
                                            mode: BC(settings.lua51.fields_per_flush, page),
                                        });

                                        offset -= settings.lua51.fields_per_flush as i64;
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
                    mode: BC(b, page + 1),
                });
            }
            lua50::Instruction::Close { a, mode } => builder.instruction(lua51::Instruction::Close { a, mode }),
            lua50::Instruction::Closure { a, mode } => builder.instruction(lua51::Instruction::Closure { a, mode }),
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
        builder.insert_extra_instruction(0, lua51::Instruction::NewTable {
            a: arg_stack_position + 1,
            mode: BC(0, 0),
        });

        // Push all variadic arguments onto the stack.
        builder.insert_extra_instruction(1, lua51::Instruction::VarArg {
            a: arg_stack_position + 2,
            mode: BC(0, 0),
        });

        // Add all values from the stack to the table.
        builder.insert_extra_instruction(2, lua51::Instruction::SetList {
            a: arg_stack_position + 1,
            mode: BC(0, 1),
        });

        // Move the table to the location of the argument.
        builder.insert_extra_instruction(3, lua51::Instruction::Move {
            a: arg_stack_position,
            mode: BC(arg_stack_position + 1, 0),
        });
    }

    builder.finalize(maxstacksize, settings)
}

#[cfg(test)]
mod tests {
    use super::{lua50, lua51, Bx, BC};
    use crate::function::upcast;
    use crate::{LunifyError, Settings};

    fn test_settings() -> Settings<'static> {
        let lua50 = lua50::Settings {
            fields_per_flush: 5,
            ..lua50::Settings::default()
        };

        let lua51 = lua51::Settings {
            fields_per_flush: 8,
            ..lua51::Settings::default()
        };

        Settings { lua50, lua51 }
    }

    fn lua50_setlist(size: u64, settings: Settings) -> Vec<lua50::Instruction> {
        let mut instructions = vec![lua50::Instruction::NewTable { a: 0, mode: BC(0, 0) }];

        for index in 0..size {
            let stack_position = (index % settings.lua50.fields_per_flush) + 1;

            instructions.push(lua50::Instruction::LoadK {
                a: stack_position,
                mode: Bx(0),
            });

            if stack_position == settings.lua50.fields_per_flush || index + 1 == size {
                instructions.push(lua50::Instruction::SetList { a: 0, mode: Bx(index) });
            }
        }

        instructions
    }

    fn lua51_setlist(size: u64, settings: Settings) -> Vec<lua51::Instruction> {
        let mut instructions = vec![lua51::Instruction::NewTable { a: 0, mode: BC(0, 0) }];

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
                    mode: BC(stack_position, page),
                });
            }
        }

        instructions
    }

    fn set_list_test(count: u64) -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = lua50_setlist(count, settings);
        let instruction_count = instructions.len();

        let (instructions, _) = upcast(
            instructions,
            vec![0; instruction_count],
            &mut Vec::new(),
            &mut 2,
            0,
            false,
            settings,
        )?;

        let expected = lua51_setlist(count, settings);

        assert_eq!(instructions, expected);
        Ok(())
    }

    #[test]
    fn upcast_set_list() -> Result<(), LunifyError> {
        set_list_test(4)
    }

    #[test]
    fn upcast_set_list_bigger_than_50_flush() -> Result<(), LunifyError> {
        set_list_test(6)
    }

    #[test]
    fn upcast_set_list_bigger_than_51_flush() -> Result<(), LunifyError> {
        set_list_test(9)
    }

    #[test]
    fn upcast_set_list_large() -> Result<(), LunifyError> {
        set_list_test(20)
    }

    #[test]
    fn upcast_set_list_from_parameters() -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = vec![lua50::Instruction::LoadK { a: 5, mode: Bx(0) }, lua50::Instruction::SetList {
            a: 0,
            mode: Bx(4),
        }];

        let (instructions, _) = upcast(instructions, vec![0; 2], &mut Vec::new(), &mut 2, 0, false, settings)?;
        let expected = vec![lua51::Instruction::LoadK { a: 5, mode: Bx(0) }, lua51::Instruction::SetList {
            a: 0,
            mode: BC(5, 1),
        }];

        assert_eq!(instructions, expected);
        Ok(())
    }

    #[test]
    fn upcast_set_list_from_parameters_bigger_than_50_flush() -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = vec![
            lua50::Instruction::LoadK { a: 5, mode: Bx(0) },
            lua50::Instruction::SetList { a: 0, mode: Bx(4) },
            lua50::Instruction::LoadK { a: 1, mode: Bx(0) },
            lua50::Instruction::SetList { a: 0, mode: Bx(5) },
        ];

        let (instructions, _) = upcast(instructions, vec![0; 12], &mut Vec::new(), &mut 2, 0, false, settings)?;
        let expected = vec![
            lua51::Instruction::LoadK { a: 5, mode: Bx(0) },
            lua51::Instruction::LoadK { a: 6, mode: Bx(0) },
            lua51::Instruction::SetList { a: 0, mode: BC(6, 1) },
        ];

        assert_eq!(instructions, expected);
        Ok(())
    }

    #[test]
    fn variadic() -> Result<(), LunifyError> {
        let settings = test_settings();
        let instructions = vec![lua50::Instruction::LoadK { a: 1, mode: Bx(0) }];

        let (instructions, _) = upcast(instructions, vec![0; 1], &mut Vec::new(), &mut 2, 0, true, settings)?;
        let expected = vec![
            lua51::Instruction::NewTable { a: 1, mode: BC(0, 0) },
            lua51::Instruction::VarArg { a: 2, mode: BC(0, 0) },
            lua51::Instruction::SetList { a: 1, mode: BC(0, 1) },
            lua51::Instruction::Move { a: 0, mode: BC(1, 0) },
            lua51::Instruction::LoadK { a: 1, mode: Bx(0) },
        ];

        assert_eq!(instructions, expected);
        Ok(())
    }
}