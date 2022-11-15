#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod endianness;
mod width;

pub use endianness::Endianness;
pub use width::BitWidth;

use crate::stream::{from_slice, ByteStream};
use crate::version::LuaVersion;
use crate::writer::ByteWriter;
use crate::LunifyError;

/// Lua bytecode format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Format {
    /// The format of the compiler. The standard is 0.
    pub format: u8,
    /// The endianness of the target system.
    pub endianness: Endianness,
    /// The width of an integer on the target system.
    pub integer_width: BitWidth,
    /// The size of a C type_t on the target system. Also the pointer width.
    pub size_t_width: BitWidth,
    /// The size of a Lua instruction inside the binary.
    pub instruction_width: BitWidth,
    /// The size of the Lua number type.
    pub number_width: BitWidth,
    /// If a Lua number is stored as an integer or a flaot.
    pub is_number_integral: bool,
}

impl Default for Format {
    fn default() -> Self {
        // By default we get the pointer width of the target system.
        let size_t_width = match cfg!(target_pointer_width = "64") {
            true => BitWidth::Bit64,
            false => BitWidth::Bit32,
        };

        // By default we get the endianness of the target system.
        let endianness = match unsafe { *(&1u32 as *const u32 as *const u8) } {
            0 => Endianness::Big,
            1 => Endianness::Little,
            _ => unreachable!(),
        };

        Self {
            format: 0,
            endianness,
            integer_width: BitWidth::Bit32,
            size_t_width,
            instruction_width: BitWidth::Bit32,
            number_width: BitWidth::Bit64,
            is_number_integral: false,
        }
    }
}

impl Format {
    pub(crate) fn from_byte_stream(byte_stream: &mut ByteStream, version: LuaVersion) -> Result<Self, LunifyError> {
        let format = match version {
            LuaVersion::Lua51 => byte_stream.byte()?,
            LuaVersion::Lua50 => 0,
        };

        let endianness = byte_stream.byte()?.try_into()?;
        let integer_width = byte_stream.byte()?.try_into().map_err(LunifyError::UnsupportedIntegerWidth)?;
        let size_t_width = byte_stream.byte()?.try_into().map_err(LunifyError::UnsupportedSizeTWidth)?;
        let instruction_width = byte_stream.byte()?.try_into().map_err(LunifyError::UnsupportedInstructionWidth)?;

        if version == LuaVersion::Lua50 {
            let instruction_format = byte_stream.slice(4)?;

            if instruction_format != [6, 8, 9, 9] {
                return Err(LunifyError::UnsupportedInstructionFormat(
                    instruction_format.try_into().unwrap(),
                ));
            }
        }

        let number_width = byte_stream.byte()?.try_into().map_err(LunifyError::UnsupportedNumberWidth)?;

        let is_number_integral = match version {
            LuaVersion::Lua51 => byte_stream.byte()? == 1,
            LuaVersion::Lua50 => {
                // TODO: double check this
                let value = from_slice!(byte_stream, number_width, endianness, f32, f64);
                value != 31415926.535897933
            }
        };

        #[cfg(feature = "debug")]
        {
            println!("format: {}", format);
            println!("endianness: {:?}", endianness);
            println!("integer_width: {}", integer_width);
            println!("size_t_width: {}", size_t_width);
            println!("instruction_width: {}", instruction_width);
            println!("number_width: {}", number_width);
            println!("is_number_integral: {}", is_number_integral);
        }

        Ok(Self {
            format,
            endianness,
            integer_width,
            size_t_width,
            instruction_width,
            number_width,
            is_number_integral,
        })
    }

    pub(crate) fn write(&self, byte_writer: &mut ByteWriter) {
        byte_writer.byte(self.format);
        byte_writer.byte(self.endianness.into());
        byte_writer.byte(self.integer_width.into());
        byte_writer.byte(self.size_t_width.into());
        byte_writer.byte(self.instruction_width.into());
        byte_writer.byte(self.number_width.into());
        byte_writer.byte(self.is_number_integral as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::LuaVersion;
    use crate::{BitWidth, ByteStream, ByteWriter, Endianness, Format, LunifyError};

    const EXPECTED_FORMAT: Format = Format {
        format: 0,
        endianness: Endianness::Little,
        integer_width: BitWidth::Bit32,
        size_t_width: BitWidth::Bit64,
        instruction_width: BitWidth::Bit32,
        number_width: BitWidth::Bit64,
        is_number_integral: false,
    };

    fn from_test_data(version: LuaVersion, bytes: &[u8]) -> Result<Format, LunifyError> {
        let mut byte_stream = ByteStream::new(bytes);
        let result = Format::from_byte_stream(&mut byte_stream, version);

        assert!(byte_stream.is_empty());
        result
    }

    #[test]
    fn lua_51() -> Result<(), LunifyError> {
        let result = from_test_data(LuaVersion::Lua51, &[0, 1, 4, 8, 4, 8, 0])?;
        assert_eq!(result, EXPECTED_FORMAT);
        Ok(())
    }

    #[test]
    fn lua_50() -> Result<(), LunifyError> {
        let result = from_test_data(LuaVersion::Lua50, &[
            1, 4, 8, 4, 6, 8, 9, 9, 8, 0xB6, 0x09, 0x93, 0x68, 0xE7, 0xF5, 0x7D, 0x41,
        ])?;
        assert_eq!(result, EXPECTED_FORMAT);
        Ok(())
    }

    #[test]
    fn write() {
        let mut byter_writer = ByteWriter::new(EXPECTED_FORMAT);
        EXPECTED_FORMAT.write(&mut byter_writer);
        assert_eq!(byter_writer.finalize(), [0, 1, 4, 8, 4, 8, 0]);
    }

    #[test]
    fn lua_50_unsupported_instruction_format() {
        let result = from_test_data(LuaVersion::Lua50, &[1, 4, 8, 4, 6, 9, 8, 9]);
        assert_eq!(result, Err(LunifyError::UnsupportedInstructionFormat([6, 9, 8, 9])));
    }

    #[test]
    fn unsupported_integer_width() {
        let result = from_test_data(LuaVersion::Lua51, &[0, 1, 6]);
        assert_eq!(result, Err(LunifyError::UnsupportedIntegerWidth(6)));
    }

    #[test]
    fn unsupported_size_t_width() {
        let result = from_test_data(LuaVersion::Lua51, &[0, 1, 4, 6]);
        assert_eq!(result, Err(LunifyError::UnsupportedSizeTWidth(6)));
    }

    #[test]
    fn unsupported_instruction_width() {
        let result = from_test_data(LuaVersion::Lua51, &[0, 1, 4, 8, 6]);
        assert_eq!(result, Err(LunifyError::UnsupportedInstructionWidth(6)));
    }

    #[test]
    fn unsupported_number_width() {
        let result = from_test_data(LuaVersion::Lua51, &[0, 1, 4, 8, 4, 6]);
        assert_eq!(result, Err(LunifyError::UnsupportedNumberWidth(6)));
    }
}
