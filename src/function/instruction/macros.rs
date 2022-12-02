macro_rules! lua_instructions {
    ($($vname:ident ( $mode:ident ),)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(crate) enum Instruction {
            $($vname { a: u64, mode: $mode },)*
        }

        impl super::RepresentInstruction for Instruction {
            // Needed because the compiler sees index as never being read for some reason.
            #[allow(dead_code, unused_assignments)]
            fn from_byte_stream(byte_stream: &mut crate::ByteStream, settings: &super::settings::Settings, layout: &InstructionLayout) -> Result<Self, crate::LunifyError> {
                use super::operant::OperantRead;

                let value = byte_stream.instruction()?;
                let opcode: Opcode = OperantRead::<Self>::parse(value, settings, layout);
                let a = OperantRead::<Self>::parse(value, settings, layout);
                let mut index = 0;

                $(
                    if opcode.0 == index {
                        return Ok(Instruction::$vname { a, mode: OperantRead::<Self>::parse(value, settings, layout) });
                    }
                    index += 1;
                )*

                Err(crate::LunifyError::InvalidOpcode(opcode.0))
            }

            #[allow(dead_code, unused_assignments)]
            fn to_u64(&self, settings: &super::settings::Settings) -> Result<u64, LunifyError> {
                use super::operant::OperantWrite;

                let mut index = 0;

                $(
                    if let Instruction::$vname { a, mode } = self {
                        let mut instruction = 0;
                        instruction |= Opcode(index).write( settings)?;
                        instruction |= a.write( settings)?;
                        instruction |= mode.write( settings)?;
                        return Ok(instruction);
                    }
                    index += 1;
                )*

                unreachable!()
            }
        }
    }
}
