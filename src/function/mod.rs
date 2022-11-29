mod builder;
mod constant;
mod instruction;
mod local;
mod upcast;

use self::constant::Constant;
use self::instruction::RepresentInstruction;
pub use self::instruction::{lua50, lua51, Settings};
use self::local::LocalVariable;
use self::upcast::upcast;
use crate::format::LuaVersion;
use crate::serialization::{ByteStream, ByteWriter};
use crate::LunifyError;

pub struct Function {
    source_file: String,
    line_defined: i64,
    last_line_defined: i64,
    parameter_count: u8,
    is_variadic: u8,
    maxstacksize: u8,
    instructions: Vec<u64>,
    constants: Vec<Constant>,
    functions: Vec<Function>,
    local_variables: Vec<LocalVariable>,
    line_info: Vec<i64>,
    upvalues: Vec<String>,
}

impl Function {
    fn get_instructions<T: RepresentInstruction>(byte_stream: &mut ByteStream) -> Result<Vec<T>, LunifyError> {
        let instruction_count = byte_stream.integer()?;
        let mut instructions = Vec::new();

        #[cfg(feature = "debug")]
        println!("instruction_count: {}", instruction_count);

        for _index in 0..instruction_count as usize {
            let instruction = T::from_byte_stream(byte_stream)?;
            instructions.push(instruction);
        }

        Ok(instructions)
    }

    fn get_constants(byte_stream: &mut ByteStream) -> Result<Vec<Constant>, LunifyError> {
        let constant_count = byte_stream.integer()?;
        let mut constants = Vec::new();

        #[cfg(feature = "debug")]
        println!("constant_count: {}", constant_count);

        for _index in 0..constant_count as usize {
            let constant_type = byte_stream.byte()?;

            match constant_type {
                0 => {
                    constants.push(Constant::Nil);

                    #[cfg(feature = "debug")]
                    println!("constant[{}] (nil)", _index);
                }

                1 => {
                    let boolean = byte_stream.byte()?;

                    #[cfg(feature = "debug")]
                    println!("constant[{}] (bool): {:?}", _index, boolean);

                    constants.push(Constant::Boolean(boolean));
                }

                3 => {
                    let number = byte_stream.number()?;

                    #[cfg(feature = "debug")]
                    println!("constant[{}] (number): {:?}", _index, number);

                    constants.push(Constant::Number(number));
                }

                4 => {
                    let string = byte_stream.string()?;

                    #[cfg(feature = "debug")]
                    println!("constant[{}] (string) ({}): {:?}", _index, string.len(), string);

                    constants.push(Constant::String(string));
                }

                invalid => return Err(LunifyError::InvalidConstantType(invalid)),
            }
        }

        Ok(constants)
    }

    fn get_functions(byte_stream: &mut ByteStream, version: LuaVersion, settings: Settings) -> Result<Vec<Function>, LunifyError> {
        let function_count = byte_stream.integer()?;
        let mut functions = Vec::new();

        #[cfg(feature = "debug")]
        println!("function_count: {}", function_count);

        for _index in 0..function_count as usize {
            let function = Function::from_byte_stream(byte_stream, version, settings)?;
            functions.push(function);
        }

        Ok(functions)
    }

    fn get_local_variables(byte_stream: &mut ByteStream) -> Result<Vec<LocalVariable>, LunifyError> {
        let local_variable_count = byte_stream.integer()?;
        let mut local_variables = Vec::new();

        #[cfg(feature = "debug")]
        println!("local_variable_count: {}", local_variable_count);

        for _index in 0..local_variable_count as usize {
            let name = byte_stream.string()?;
            let start_program_counter = byte_stream.integer()?;
            let end_program_counter = byte_stream.integer()?;

            #[cfg(feature = "debug")]
            println!(
                "local variable[{}] ({} - {}): {:?}",
                _index, start_program_counter, end_program_counter, name
            );

            let local_variable = LocalVariable {
                name,
                start_program_counter,
                end_program_counter,
            };

            local_variables.push(local_variable);
        }

        Ok(local_variables)
    }

    fn get_line_info(byte_stream: &mut ByteStream) -> Result<Vec<i64>, LunifyError> {
        let line_info_count = byte_stream.integer()?;
        let mut line_info = Vec::new();

        #[cfg(feature = "debug")]
        println!("line_info_count: {}", line_info_count);

        for _index in 0..line_info_count as usize {
            let line = byte_stream.integer()?;

            #[cfg(feature = "debug")]
            println!("intruction {}: line {}", _index, line);

            line_info.push(line);
        }

        Ok(line_info)
    }

    fn get_upvalues(byte_stream: &mut ByteStream) -> Result<Vec<String>, LunifyError> {
        let upvalue_count = byte_stream.integer()?;
        let mut upvalues = Vec::new();

        #[cfg(feature = "debug")]
        println!("upvalue_count: {}", upvalue_count);

        for _index in 0..upvalue_count as usize {
            let upvalue = byte_stream.string()?;

            #[cfg(feature = "debug")]
            println!("upvalue[{}]: {:?}", _index, upvalue);

            upvalues.push(upvalue);
        }

        Ok(upvalues)
    }

    pub(crate) fn from_byte_stream(byte_stream: &mut ByteStream, version: LuaVersion, settings: Settings) -> Result<Self, LunifyError> {
        let source_file = byte_stream.string()?;
        let line_defined = byte_stream.integer()?;

        let last_line_defined = match version {
            LuaVersion::Lua51 => byte_stream.integer()?,
            LuaVersion::Lua50 => line_defined,
        };

        let _upvalue_count = byte_stream.byte()?;
        let parameter_count = byte_stream.byte()?;
        let mut is_variadic = byte_stream.byte()?;
        let mut maxstacksize = byte_stream.byte()?;

        #[cfg(feature = "debug")]
        {
            println!("source_file: {}", source_file);
            println!("line_defined: {}", line_defined);
            println!("last_line_defined: {}", last_line_defined);
            println!("upvalue_count: {}", _upvalue_count);
            println!("parameter_count: {}", parameter_count);
            println!("is_variadic: {}", is_variadic);
            println!("maxstacksize: {}", maxstacksize);
        }

        let (instructions, constants, functions, line_info, local_variables, upvalues) = if version == LuaVersion::Lua51 {
            let instructions = Self::get_instructions(byte_stream)?;
            let constants = Self::get_constants(byte_stream)?;
            let functions = Self::get_functions(byte_stream, version, settings)?;
            let line_info = Self::get_line_info(byte_stream)?;
            let local_variables = Self::get_local_variables(byte_stream)?;
            let upvalues = Self::get_upvalues(byte_stream)?;

            (instructions, constants, functions, line_info, local_variables, upvalues)
        } else {
            let line_info = Self::get_line_info(byte_stream)?;
            let local_variables = Self::get_local_variables(byte_stream)?;
            let upvalues = Self::get_upvalues(byte_stream)?;
            let mut constants = Self::get_constants(byte_stream)?;
            let functions = Self::get_functions(byte_stream, version, settings)?;
            let instructions = Self::get_instructions::<lua50::Instruction>(byte_stream)?;

            // TODO: confirm
            // TODO: document
            is_variadic *= 2;

            // Upcast instructions from Lua 5.0 to Lua 5.1.
            let (instructions, line_info) = upcast(
                instructions,
                line_info,
                &mut constants,
                &mut maxstacksize,
                parameter_count,
                is_variadic != 0,
                settings,
            )?;

            let instructions = instructions.into_iter().map(|instruction| instruction.to_u64()).collect();

            (instructions, constants, functions, line_info, local_variables, upvalues)
        };

        Ok(Self {
            source_file,
            line_defined,
            last_line_defined,
            parameter_count,
            is_variadic,
            maxstacksize,
            instructions,
            constants,
            functions,
            local_variables,
            line_info,
            upvalues,
        })
    }

    pub(crate) fn write(self, byte_writer: &mut ByteWriter) -> Result<(), LunifyError> {
        // function
        byte_writer.string(&self.source_file);
        byte_writer.integer(self.line_defined);
        byte_writer.integer(self.last_line_defined);
        byte_writer.byte(self.upvalues.len() as u8);
        byte_writer.byte(self.parameter_count);
        byte_writer.byte(self.is_variadic);
        byte_writer.byte(self.maxstacksize);

        // instructions
        byte_writer.integer(self.instructions.len() as i64);
        for instruction in self.instructions {
            byte_writer.instruction(instruction);
        }

        // constants
        byte_writer.integer(self.constants.len() as i64);
        for constant in self.constants {
            match constant {
                Constant::Nil => {
                    byte_writer.byte(0);
                }

                Constant::Boolean(boolean) => {
                    byte_writer.byte(1);
                    byte_writer.byte(boolean);
                }

                Constant::Number(number) => {
                    byte_writer.byte(3);
                    byte_writer.number(number)?;
                }

                Constant::String(string) => {
                    byte_writer.byte(4);
                    byte_writer.string(&string);
                }
            }
        }

        // functions
        byte_writer.integer(self.functions.len() as i64);
        for function in self.functions {
            function.write(byte_writer)?;
        }

        // line info
        byte_writer.integer(self.line_info.len() as i64);
        for line_info in self.line_info {
            byte_writer.integer(line_info);
        }

        // local variables
        byte_writer.integer(self.local_variables.len() as i64);
        for local_variable in self.local_variables {
            byte_writer.string(&local_variable.name);
            byte_writer.integer(local_variable.start_program_counter);
            byte_writer.integer(local_variable.end_program_counter);
        }

        // upvalues
        byte_writer.integer(self.upvalues.len() as i64);
        for upvalue in self.upvalues {
            byte_writer.string(&upvalue);
        }

        Ok(())
    }
}
