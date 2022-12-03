macro_rules! lua_instructions {
    ($($vname:ident ( $mode:ty, $move_a:literal ),)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(crate) enum Instruction {
            $($vname { a: u64, mode: $mode },)*
        }

        impl super::LuaInstruction for Instruction {
            // Needed because the compiler sees these functions as never being used and index as never being read.
            #[allow(dead_code, unused_assignments)]
            fn from_byte_stream(byte_stream: &mut crate::ByteStream, settings: &super::settings::Settings, layout: &InstructionLayout) -> Result<Self, crate::LunifyError> {
                use super::operant::OperantGet;

                let value = byte_stream.instruction()?;
                let opcode: Opcode = OperantGet::<Self>::get(value, settings, layout);
                let a: A = OperantGet::<Self>::get(value, settings, layout);
                let mut index = 0;

                $(
                    if opcode.0 == index {
                        return Ok(Instruction::$vname { a: a.0, mode: OperantGet::<Self>::get(value, settings, layout) });
                    }
                    index += 1;
                )*

                Err(crate::LunifyError::InvalidOpcode(opcode.0))
            }

            #[allow(dead_code)]
            fn move_stack_accesses(&mut self, stack_start: u64, offset: i64) {
                use super::operant::OperantOffset;

                match self {
                    $(Self::$vname { a, mode } => {
                        if $move_a {
                            let value = *a;
                            if value >= stack_start {
                                *a = (value as i64 + offset) as u64;
                            }
                        }
                        mode.offset(stack_start, offset);
                    },)*
                }
            }

            #[allow(dead_code, unused_assignments)]
            fn to_u64(&self, settings: &super::settings::Settings) -> Result<u64, LunifyError> {
                use super::operant::OperantPut;

                let mut index = 0;

                $(
                    if let Instruction::$vname { a, mode } = self {
                        let mut instruction = 0;
                        instruction |= Opcode(index).put( settings)?;
                        instruction |= A(*a).put( settings)?;
                        instruction |= mode.put( settings)?;
                        return Ok(instruction);
                    }
                    index += 1;
                )*

                unreachable!()
            }
        }
    }
}
