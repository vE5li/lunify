#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::stream::ByteStream;
use crate::writer::ByteWriter;
use crate::LunifyError;

/// Lua bytecode format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Format {
    pub format: u8,
    pub endianness: u8,
    pub integer_width: u8,
    pub size_t_width: u8,
    pub instruction_width: u8,
    pub number_width: u8,
    pub is_number_integral: u8,
}

impl Default for Format {
    fn default() -> Self {
        let size_t_size = match cfg!(target_pointer_width = "64") {
            true => 8,
            false => 4,
        };

        Self {
            format: 0,
            endianness: 1,
            integer_width: 4,
            size_t_width: size_t_size,
            instruction_width: 4,
            number_width: 8,
            is_number_integral: 0,
        }
    }
}

impl Format {
    pub(crate) fn from_byte_stream(byte_stream: &mut ByteStream, version: u8) -> Result<Self, LunifyError> {
        let format = match version {
            0x51 => byte_stream.byte()?,
            0x50 => 0,
            _ => unreachable!(),
        };

        let endianness = byte_stream.byte()?;
        let integer_width = byte_stream.byte()?;
        let size_t_width = byte_stream.byte()?;
        let instruction_width = byte_stream.byte()?;

        if version == 0x50 {
            let format = byte_stream.slice(4)?;
            if format[0] != 6 || format[1] != 8 || format[2] != 9 || format[3] != 9 {
                return Err(LunifyError::UnsupportedInstructionFormat(format.try_into().unwrap()));
            }
        }

        let number_width = byte_stream.byte()?;
        byte_stream.set_number_format(endianness, number_width);

        let is_number_integral = match version {
            0x51 => byte_stream.byte()?,
            0x50 => byte_stream.number()? as u8, // TODO: some check
            _ => unreachable!(),
        };

        #[cfg(feature = "debug")]
        println!("format: {}", format);
        #[cfg(feature = "debug")]
        println!("endianness: {}", endianness);
        #[cfg(feature = "debug")]
        println!("integer_size: {}", integer_width);
        #[cfg(feature = "debug")]
        println!("size_t_size: {}", size_t_width);
        #[cfg(feature = "debug")]
        println!("instruction_size: {}", instruction_width);
        #[cfg(feature = "debug")]
        println!("number_size: {}", number_width);
        #[cfg(feature = "debug")]
        println!("is_number_integral: {}", is_number_integral);

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
        byte_writer.byte(self.endianness);
        byte_writer.byte(self.integer_width);
        byte_writer.byte(self.size_t_width);
        byte_writer.byte(self.instruction_width);
        byte_writer.byte(self.number_width);
        byte_writer.byte(self.is_number_integral);
    }

    pub fn assert_supported(&self) -> Result<(), LunifyError> {
        if self.integer_width != 4 && self.integer_width != 8 {
            return Err(LunifyError::UnsupportedIntegerSize(self.integer_width));
        }

        if self.size_t_width != 4 && self.size_t_width != 8 {
            return Err(LunifyError::UnsupportedSizeTSize(self.size_t_width));
        }

        if self.instruction_width != 4 && self.instruction_width != 8 {
            return Err(LunifyError::UnsupportedInstructionSize(self.instruction_width));
        }

        if self.number_width != 4 && self.number_width != 8 {
            return Err(LunifyError::UnsupportedNumberSize(self.number_width));
        }

        Ok(())
    }
}
