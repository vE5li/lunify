use crate::stream::ByteStream;
use crate::writer::ByteWriter;
use crate::LunifyError;

enum Constant {
    Nil,
    Boolean(u8),
    Number(i64),
    String(String),
}

struct LocalVariable {
    name: String,
    start_program_counter: i64,
    end_program_counter: i64,
}

pub struct Function {
    source_file: String,
    line_defined: i64,
    last_line_defined: i64,
    nups: u8,
    parameter_count: u8,
    is_variardic: u8,
    maxstacksize: u8,
    instructions: Vec<u64>,
    constants: Vec<Constant>,
    functions: Vec<Function>,
    local_variables: Vec<LocalVariable>,
    line_info: Vec<i64>,
    upvalues: Vec<String>,
}

impl Function {

    pub(crate) fn from_byte_stream(byte_stream: &mut ByteStream) -> Result<Self, LunifyError> {

        let source_file = byte_stream.string()?;
        let line_defined = byte_stream.integer()?;
        let last_line_defined = byte_stream.integer()?;
        let nups = byte_stream.byte()?;
        let parameter_count = byte_stream.byte()?;
        let is_variardic = byte_stream.byte()?;
        let maxstacksize = byte_stream.byte()?;

        #[cfg(feature = "debug")]
        println!("source_file: {}", source_file);
        #[cfg(feature = "debug")]
        println!("line_defined: {}", line_defined);
        #[cfg(feature = "debug")]
        println!("last_line_defined: {}", last_line_defined);
        #[cfg(feature = "debug")]
        println!("nups: {}", nups);
        #[cfg(feature = "debug")]
        println!("parameter_count: {}", parameter_count);
        #[cfg(feature = "debug")]
        println!("is_variardic: {}", is_variardic);
        #[cfg(feature = "debug")]
        println!("maxstacksize: {}", maxstacksize);

        let mut instructions = Vec::new();
        let mut constants = Vec::new();
        let mut functions = Vec::new();
        let mut local_variables = Vec::new();
        let mut line_info = Vec::new();
        let mut upvalues = Vec::new();

        let instruction_count = byte_stream.integer()?;
        #[cfg(feature = "debug")]
        println!("instruction_count: {}", instruction_count);

        for _index in 0..instruction_count as usize {

            let instruction = byte_stream.instruction()?;
            #[cfg(feature = "debug")]
            println!("instruction[{}]: {:x?}", _index, instruction);
            instructions.push(instruction);
        }

        let constant_count = byte_stream.integer()?;
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

                2 => {
                    panic!();
                }

                3 => {

                    let number = byte_stream.number()?;

                    #[cfg(feature = "debug")]
                    println!("constant[{}] (int): {:?}", _index, number);

                    constants.push(Constant::Number(number));
                }

                4 => {

                    // string
                    let string = byte_stream.string()?; // TODO: find a way to make
                    //
                    #[cfg(feature = "debug")]
                    println!("constant[{}] (string) ({}): {:?}", _index, string.len(), string);

                    constants.push(Constant::String(string));
                }

                invalid => return Err(LunifyError::InvalidConstantType(invalid)),
            }
        }

        let function_count = byte_stream.integer()?;
        #[cfg(feature = "debug")]
        println!("function_count: {}", function_count);

        for _index in 0..function_count as usize {

            let function = Function::from_byte_stream(byte_stream)?;
            functions.push(function);
        }

        let line_info_count = byte_stream.integer()?;
        #[cfg(feature = "debug")]
        println!("line_info_count: {}", line_info_count);

        for _index in 0..line_info_count as usize {

            let line = byte_stream.integer()?;

            #[cfg(feature = "debug")]
            println!("intruction {}: line {}", _index, line);

            line_info.push(line);
        }

        let local_variable_count = byte_stream.integer()?;
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

        let upvalue_count = byte_stream.integer()?;
        #[cfg(feature = "debug")]
        println!("upvalue_count: {}", upvalue_count);

        for _index in 0..upvalue_count as usize {

            let upvalue = byte_stream.string()?;

            #[cfg(feature = "debug")]
            println!("upvalue[{}]: {:?}", _index, upvalue);

            upvalues.push(upvalue);
        }

        Ok(Self {
            source_file,
            line_defined,
            last_line_defined,
            nups,
            parameter_count,
            is_variardic,
            maxstacksize,
            instructions,
            constants,
            functions,
            local_variables,
            line_info,
            upvalues,
        })
    }

    pub(crate) fn write(self, byte_writer: &mut ByteWriter) {

        // function
        byte_writer.string(&self.source_file);
        byte_writer.integer(self.line_defined);
        byte_writer.integer(self.last_line_defined);
        byte_writer.byte(self.nups);
        byte_writer.byte(self.parameter_count);
        byte_writer.byte(self.is_variardic);
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
                    byte_writer.number(number);
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
            function.write(byte_writer);
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

        // ??
        byte_writer.integer(self.upvalues.len() as i64);
        for upvalue in self.upvalues {
            byte_writer.string(&upvalue);
        }
    }
}
