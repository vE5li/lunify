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
                use super::operant::Operant;

                let value = byte_stream.instruction()?;
                let opcode = value & 0b111111;
                let a = Operant::parse(value, settings, layout);
                let mut index = 0;

                $(
                    if opcode == index {
                        return Ok(Instruction::$vname { a, mode: Operant::parse(value, settings, layout) });
                    }
                    index += 1;
                )*

                Err(crate::LunifyError::InvalidOpcode(value))
            }

            #[allow(dead_code, unused_assignments)]
            fn to_u64(&self, settings: &super::settings::Settings, layout: &InstructionLayout) -> u64 {
                use super::operant::Operant;

                let mut index = 0;

                $(
                    if let Instruction::$vname { a, mode } = self {
                        // opcode
                        let mut instruction = index;
                        instruction |= Operant::write(*a, settings, layout);
                        instruction |= Operant::write(*mode, settings, layout);
                        return instruction;
                    }
                    index += 1;
                )*

                unreachable!()
            }
        }
    }
}
