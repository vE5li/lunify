use crate::number::Number;
use crate::{BitWidth, Endianness, Format, LunifyError};

pub(crate) struct ByteWriter<'a> {
    data: Vec<u8>,
    format: &'a Format,
}

impl<'a> ByteWriter<'a> {
    pub fn new(format: &'a Format) -> Self {
        let data = Vec::new();
        Self { data, format }
    }

    pub fn byte(&mut self, byte: u8) {
        self.data.push(byte);
    }

    pub fn slice(&mut self, slice: &[u8]) {
        self.data.extend_from_slice(slice);
    }

    pub fn integer(&mut self, value: i64) {
        to_slice!(self, value, integer_width, i32)
    }

    pub fn size_t(&mut self, value: i64) {
        to_slice!(self, value, size_t_width, i32)
    }

    pub fn instruction(&mut self, instruction: u64) {
        to_slice!(self, instruction, instruction_width, u32)
    }

    pub fn number(&mut self, value: Number) -> Result<(), LunifyError> {
        match self.format.is_number_integral {
            true => to_slice!(self, value.as_integer()?, number_width, i32),
            false => to_slice!(self, value.as_float()?, number_width, f32),
        }
        Ok(())
    }

    pub fn string(&mut self, value: &str) {
        self.size_t(value.len() as i64);
        self.slice(value.as_bytes());
    }

    pub fn finalize(self) -> Vec<u8> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::ByteWriter;
    use crate::number::Number;
    use crate::{BitWidth, Endianness, Format, LunifyError};

    const TEST_FORMAT: Format = Format {
        format: 0,
        endianness: Endianness::Little,
        integer_width: BitWidth::Bit32,
        size_t_width: BitWidth::Bit32,
        instruction_width: BitWidth::Bit32,
        number_width: BitWidth::Bit32,
        is_number_integral: false,
    };

    macro_rules! configuration {
        ($endianness:ident, $width:expr, $value:expr, $expected:expr) => {
            Configuration {
                expected: $expected.as_slice(),
                endianness: Endianness::$endianness,
                width: $width,
                is_number_integral: false,
                value: $value,
            }
        };

        ($endianness:ident, $width:expr, $is_number_integral:expr, $value:expr, $expected:expr) => {
            Configuration {
                expected: $expected.as_slice(),
                endianness: Endianness::$endianness,
                width: $width,
                is_number_integral: $is_number_integral,
                value: $value,
            }
        };
    }

    struct Configuration<'a, T> {
        expected: &'a [u8],
        endianness: Endianness,
        width: BitWidth,
        is_number_integral: bool,
        value: T,
    }

    impl<T> Configuration<'_, T> {
        fn format(&self) -> Format {
            Format {
                endianness: self.endianness,
                integer_width: self.width,
                size_t_width: self.width,
                instruction_width: self.width,
                number_width: self.width,
                is_number_integral: self.is_number_integral,
                ..Default::default()
            }
        }
    }

    #[test]
    fn byte() {
        let mut writer = ByteWriter::new(&TEST_FORMAT);
        writer.byte(9);
        assert_eq!(writer.data, &[9]);
    }

    #[test]
    fn slice() {
        let mut writer = ByteWriter::new(&TEST_FORMAT);
        writer.slice(&[7, 8, 9]);
        assert_eq!(writer.data, &[7, 8, 9]);
    }

    #[test]
    fn integer() {
        let configurations = [
            configuration!(Little, BitWidth::Bit32, 9, [9, 0, 0, 0]),
            configuration!(Big, BitWidth::Bit32, 9, [0, 0, 0, 9]),
            configuration!(Little, BitWidth::Bit64, 9, [9, 0, 0, 0, 0, 0, 0, 0]),
            configuration!(Big, BitWidth::Bit64, 9, [0, 0, 0, 0, 0, 0, 0, 9]),
        ];

        for configuration in configurations {
            let format = configuration.format();
            let mut writer = ByteWriter::new(&format);
            writer.integer(configuration.value);
            assert_eq!(writer.data, configuration.expected);
        }
    }

    #[test]
    fn size_t() {
        let configurations = [
            configuration!(Little, BitWidth::Bit32, 9, [9, 0, 0, 0]),
            configuration!(Big, BitWidth::Bit32, 9, [0, 0, 0, 9]),
            configuration!(Little, BitWidth::Bit64, 9, [9, 0, 0, 0, 0, 0, 0, 0]),
            configuration!(Big, BitWidth::Bit64, 9, [0, 0, 0, 0, 0, 0, 0, 9]),
        ];

        for configuration in configurations {
            let format = configuration.format();
            let mut writer = ByteWriter::new(&format);
            writer.size_t(configuration.value);
            assert_eq!(writer.data, configuration.expected);
        }
    }

    #[test]
    fn instruction() {
        let configurations = [
            configuration!(Little, BitWidth::Bit32, 9, [9, 0, 0, 0]),
            configuration!(Big, BitWidth::Bit32, 9, [0, 0, 0, 9]),
            configuration!(Little, BitWidth::Bit64, 9, [9, 0, 0, 0, 0, 0, 0, 0]),
            configuration!(Big, BitWidth::Bit64, 9, [0, 0, 0, 0, 0, 0, 0, 9]),
        ];

        for configuration in configurations {
            let format = configuration.format();
            let mut writer = ByteWriter::new(&format);
            writer.instruction(configuration.value);
            assert_eq!(writer.data, configuration.expected);
        }
    }

    #[test]
    fn number() -> Result<(), LunifyError> {
        let configurations = [
            // Integer
            configuration!(Little, BitWidth::Bit32, true, Number::Integer(9), [9, 0, 0, 0]),
            configuration!(Big, BitWidth::Bit32, true, Number::Integer(9), [0, 0, 0, 9]),
            configuration!(Little, BitWidth::Bit64, true, Number::Integer(9), [9, 0, 0, 0, 0, 0, 0, 0]),
            configuration!(Big, BitWidth::Bit64, true, Number::Integer(9), [0, 0, 0, 0, 0, 0, 0, 9]),
            // Float
            configuration!(Little, BitWidth::Bit32, false, Number::Float(9.0), [0, 0, 16, 65]),
            configuration!(Big, BitWidth::Bit32, false, Number::Float(9.0), [65, 16, 0, 0]),
            configuration!(Little, BitWidth::Bit64, false, Number::Float(9.0), [0, 0, 0, 0, 0, 0, 34, 64]),
            configuration!(Big, BitWidth::Bit64, false, Number::Float(9.0), [64, 34, 0, 0, 0, 0, 0, 0]),
        ];

        for configuration in configurations {
            let format = configuration.format();
            let mut writer = ByteWriter::new(&format);
            writer.number(configuration.value)?;
            assert_eq!(writer.data, configuration.expected);
        }

        Ok(())
    }

    #[test]
    fn string() {
        let mut writer = ByteWriter::new(&TEST_FORMAT);
        writer.string("LUA");
        assert_eq!(writer.data, &[3, 0, 0, 0, b'L', b'U', b'A']);
    }

    #[test]
    fn finalize() {
        let writer = ByteWriter {
            data: vec![7, 8, 9],
            format: &TEST_FORMAT,
        };
        assert_eq!(writer.finalize(), &[7, 8, 9]);
    }
}
