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
    pub integer_size: u8,
    pub size_t_size: u8,
    pub instruction_size: u8,
    pub number_size: u8,
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
            integer_size: 4,
            size_t_size,
            instruction_size: 4,
            number_size: 8,
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
        let integer_size = byte_stream.byte()?;
        let size_t_size = byte_stream.byte()?;
        let instruction_size = byte_stream.byte()?;

        if version == 0x50 {

            let format = byte_stream.slice(4)?;
            if format[0] != 6 || format[1] != 8 || format[2] != 9 || format[3] != 9 {
                return Err(LunifyError::UnsupportedInstructionFormat(format.try_into().unwrap()));
            }
        }

        let number_size = byte_stream.byte()?;
        byte_stream.set_number_format(endianness, number_size);

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
        println!("integer_size: {}", integer_size);
        #[cfg(feature = "debug")]
        println!("size_t_size: {}", size_t_size);
        #[cfg(feature = "debug")]
        println!("instruction_size: {}", instruction_size);
        #[cfg(feature = "debug")]
        println!("number_size: {}", number_size);
        #[cfg(feature = "debug")]
        println!("is_number_integral: {}", is_number_integral);

        Ok(Self {
            format,
            endianness,
            integer_size,
            size_t_size,
            instruction_size,
            number_size,
            is_number_integral,
        })
    }

    pub(crate) fn write(&self, byte_writer: &mut ByteWriter) {

        byte_writer.byte(self.format);
        byte_writer.byte(self.endianness);
        byte_writer.byte(self.integer_size);
        byte_writer.byte(self.size_t_size);
        byte_writer.byte(self.instruction_size);
        byte_writer.byte(self.number_size);
        byte_writer.byte(self.is_number_integral);
    }

    pub fn assert_supported(&self) -> Result<(), LunifyError> {

        if self.integer_size != 4 && self.integer_size != 8 {
            return Err(LunifyError::UnsupportedIntegerSize(self.integer_size));
        }

        if self.size_t_size != 4 && self.size_t_size != 8 {
            return Err(LunifyError::UnsupportedSizeTSize(self.size_t_size));
        }

        if self.instruction_size != 4 && self.instruction_size != 8 {
            return Err(LunifyError::UnsupportedInstructionSize(self.instruction_size));
        }

        if self.number_size != 4 && self.number_size != 8 {
            return Err(LunifyError::UnsupportedNumberSize(self.number_size));
        }

        Ok(())
    }
}
