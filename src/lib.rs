#![doc = include_str!("../README.md")]
//#![deny(missing_docs)]

mod error;
mod format;
mod function;
mod instruction;
mod number;
mod stream;
mod version;
mod writer;

pub use error::LunifyError;
pub use format::{BitWidth, Endianness, Format};
use function::Function;
use instruction::Settings;
pub use instruction::{lua50, lua51};
use stream::ByteStream;
use version::LuaVersion;
use writer::ByteWriter;

const LUA_SIGNATURE: &[u8; 4] = b"\x1bLua";

/// Takes Lua bytecode in a supported format and converts it to bytecode in the
/// specified output [Format]. Returns [LunifyError] on error.
pub fn unify(input_bytes: &[u8], output_format: Format, settings: Settings) -> Result<Vec<u8>, LunifyError> {
    let mut byte_stream = ByteStream::new(input_bytes);

    let signature = byte_stream.slice(LUA_SIGNATURE.len())?;

    if signature != LUA_SIGNATURE {
        return Err(LunifyError::IncorrectSignature);
    }

    let version = byte_stream.byte()?.try_into()?;

    #[cfg(feature = "debug")]
    println!("version: {}", version);

    let input_format = Format::from_byte_stream(&mut byte_stream, version)?;

    // if the input is already in the correct format, return it as is
    if input_format == output_format && !cfg!(test) {
        return Ok(input_bytes.to_vec());
    }

    byte_stream.set_format(input_format);

    let root_function = Function::from_byte_stream(&mut byte_stream, version, settings)?;

    if !byte_stream.is_empty() {
        return Err(LunifyError::InputTooLong);
    }

    let mut byte_writer = ByteWriter::new(output_format);

    byte_writer.slice(LUA_SIGNATURE);
    byte_writer.byte(LuaVersion::Lua51.into());

    output_format.write(&mut byte_writer);
    root_function.write(&mut byte_writer)?;

    Ok(byte_writer.finalize())
}

#[cfg(test)]
mod tests {
    use super::{unify, Format, LunifyError};
    use crate::{BitWidth, Endianness};

    #[cfg(feature = "integration")]
    fn test_output(bytecode: &[u8]) {
        use mlua::prelude::*;

        let lua = Lua::new();
        lua.load(bytecode).exec().unwrap();
        assert_eq!(lua.globals().get::<_, LuaNumber>("result").unwrap(), 9.0);
    }

    #[test]
    fn _32bit_to_64bit() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/32bit.luab");
        let output_format = Format {
            size_t_width: BitWidth::Bit64,
            ..Default::default()
        };
        let _output_bytes = unify(input_bytes, output_format, Default::default())?;

        Ok(())
    }

    #[test]
    fn lua50_to_lua51() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/lua50.luab");
        let output_format = Format::default();
        let output_bytes = unify(input_bytes, output_format, Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&output_bytes);
        Ok(())
    }

    #[test]
    fn matching_format_remains_unchanged() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/32bit.luab");
        let output_format = Format {
            endianness: Endianness::Little,
            size_t_width: BitWidth::Bit32,
            ..Default::default()
        };
        let output_bytes = unify(input_bytes, output_format, Default::default())?;

        assert_eq!(&input_bytes[..], &output_bytes);
        Ok(())
    }

    #[test]
    fn little_endian() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/little_endian.luab");
        let output_format = Format::default();
        let output_bytes = unify(input_bytes, output_format, Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&output_bytes);
        Ok(())
    }

    #[test]
    fn big_endian() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/big_endian.luab");
        let output_format = Format::default();
        let output_bytes = unify(input_bytes, output_format, Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&output_bytes);
        Ok(())
    }

    #[test]
    fn large_table() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/large_table.luab");
        let output_format = Format::default();
        let output_bytes = unify(input_bytes, output_format, Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&output_bytes);
        Ok(())
    }

    #[test]
    fn variadic() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/variadic.luab");
        let output_format = Format::default();
        let output_bytes = unify(input_bytes, output_format, Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&output_bytes);
        Ok(())
    }
}
