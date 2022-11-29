use std::convert::TryInto;

use crate::number::Number;
use crate::{Endianness, Format, LunifyError};

pub(crate) struct ByteStream<'a> {
    data: &'a [u8],
    offset: usize,
    format: Format,
}

impl<'a> ByteStream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let offset = 0;
        let format = Format::default();

        Self { data, offset, format }
    }

    pub fn set_format(&mut self, format: Format) {
        self.format = format;
    }

    pub fn byte(&mut self) -> Result<u8, LunifyError> {
        let offset = self.offset;
        self.offset += 1;
        self.data.get(offset).cloned().ok_or(LunifyError::InputTooShort)
    }

    pub fn integer(&mut self) -> Result<i64, LunifyError> {
        Ok(from_slice!(self, self.format.integer_width, self.format.endianness, i32, i64))
    }

    pub fn size_t(&mut self) -> Result<i64, LunifyError> {
        Ok(from_slice!(self, self.format.size_t_width, self.format.endianness, i32, i64))
    }

    pub fn instruction(&mut self) -> Result<u64, LunifyError> {
        Ok(from_slice!(
            self,
            self.format.instruction_width,
            self.format.endianness,
            u32,
            u64
        ))
    }

    pub fn number(&mut self) -> Result<Number, LunifyError> {
        match self.format.is_number_integral {
            true => Ok(Number::Integer(from_slice!(
                self,
                self.format.number_width,
                self.format.endianness,
                i32,
                i64
            ))),
            false => Ok(Number::Float(from_slice!(
                self,
                self.format.number_width,
                self.format.endianness,
                f32,
                f64
            ))),
        }
    }

    pub fn slice(&mut self, length: usize) -> Result<&[u8], LunifyError> {
        let start = self.offset;
        self.offset += length;

        if self.offset > self.data.len() {
            return Err(LunifyError::InputTooShort);
        }

        Ok(&self.data[start..self.offset])
    }

    pub fn string(&mut self) -> Result<String, LunifyError> {
        let length = self.size_t()? as usize;
        let start = self.offset;

        self.offset += length;

        if self.offset > self.data.len() {
            return Err(LunifyError::InputTooShort);
        }

        Ok(self.data[start..self.offset].iter().map(|&byte| byte as char).collect())
    }

    pub fn is_empty(&self) -> bool {
        self.offset >= self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::ByteStream;
    use crate::{BitWidth, Endianness, Format, LunifyError, number::Number};

    const TEST_FORMAT: Format = Format {
        format: 80,
        endianness: Endianness::Little,
        integer_width: BitWidth::Bit32,
        size_t_width: BitWidth::Bit64,
        instruction_width: BitWidth::Bit32,
        number_width: BitWidth::Bit64,
        is_number_integral: false,
    };

    macro_rules! configuration {
        ($endianness:ident, $width:expr, $expected:expr, $bytes:expr) => {
            Configuration {
                bytes: $bytes.as_slice(),
                endianness: Endianness::$endianness,
                width: $width,
                is_number_integral: false,
                expected: $expected,
            }
        };

        ($endianness:ident, $width:expr, $is_number_integral:expr, $expected:expr, $bytes:expr) => {
            Configuration {
                bytes: $bytes.as_slice(),
                endianness: Endianness::$endianness,
                width: $width,
                is_number_integral: $is_number_integral,
                expected: $expected,
            }
        };
    }

    struct Configuration<'a, T> {
        bytes: &'a [u8],
        endianness: Endianness,
        width: BitWidth,
        is_number_integral: bool,
        expected: T,
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
    fn set_format() {
        let mut stream = ByteStream::new(&[]);
        stream.set_format(TEST_FORMAT);
        assert_eq!(stream.format, TEST_FORMAT);
    }

    #[test]
    fn byte() {
        let mut stream = ByteStream::new(&[9]);
        assert_eq!(stream.byte(), Ok(9));
        assert!(stream.is_empty());
    }

    #[test]
    fn byte_too_short() {
        let mut stream = ByteStream::new(&[]);
        assert_eq!(stream.byte(), Err(LunifyError::InputTooShort));
        assert!(stream.is_empty());
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
            let mut stream = ByteStream::new(configuration.bytes);
            stream.set_format(configuration.format());

            assert_eq!(stream.integer(), Ok(configuration.expected));
            assert!(stream.is_empty());
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
            let mut stream = ByteStream::new(configuration.bytes);
            stream.set_format(configuration.format());

            assert_eq!(stream.size_t(), Ok(configuration.expected));
            assert!(stream.is_empty());
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
            let mut stream = ByteStream::new(configuration.bytes);
            stream.set_format(configuration.format());

            assert_eq!(stream.instruction(), Ok(configuration.expected));
            assert!(stream.is_empty());
        }
    }

    #[test]
    fn number() {
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
            let mut stream = ByteStream::new(configuration.bytes);
            stream.set_format(configuration.format());

            assert_eq!(stream.number(), Ok(configuration.expected));
            assert!(stream.is_empty());
        }
    }

    #[test]
    fn slice() {
        let mut stream = ByteStream::new(&[9, 9, 9]);
        stream.set_format(TEST_FORMAT);
        assert_eq!(stream.slice(3), Ok([9, 9, 9].as_slice()));
        assert!(stream.is_empty());
    }

    #[test]
    fn empty_slice() {
        let mut stream = ByteStream::new(&[]);
        stream.set_format(TEST_FORMAT);
        assert_eq!(stream.slice(0), Ok([].as_slice()));
        assert!(stream.is_empty());
    }

    #[test]
    fn slice_too_short() {
        let mut stream = ByteStream::new(&[9, 9]);
        stream.set_format(TEST_FORMAT);
        assert_eq!(stream.slice(3), Err(LunifyError::InputTooShort));
        assert!(stream.is_empty());
    }

    #[test]
    fn string() {
        let mut stream = ByteStream::new(&[3, 0, 0, 0, 0, 0, 0, 0, b'L', b'U', b'A']);
        stream.set_format(TEST_FORMAT);
        assert_eq!(stream.string(), Ok("LUA".to_owned()));
        assert!(stream.is_empty());
    }

    #[test]
    fn string_too_short() {
        let mut stream = ByteStream::new(&[3, 0, 0, 0, 0, 0, 0, 0, b'L', b'U']);
        stream.set_format(TEST_FORMAT);
        assert_eq!(stream.string(), Err(LunifyError::InputTooShort));
        assert!(stream.is_empty());
    }

    #[test]
    fn is_empty() {
        let stream = ByteStream::new(&[]);
        assert!(stream.is_empty());
    }

    #[test]
    fn is_not_empty() {
        let stream = ByteStream::new(&[0]);
        assert!(!stream.is_empty());
    }
}
