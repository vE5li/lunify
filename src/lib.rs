#![doc = include_str!("../README.md")]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]

mod error;
mod number;
#[macro_use]
mod serialization;
mod format;
mod function;

pub use error::LunifyError;
pub use format::{BitWidth, Endianness, Format};
use function::Function;
pub use function::{lua50, lua51, InstructionLayout, OperandType, Settings};

use crate::format::LuaVersion;
use crate::serialization::{ByteStream, ByteWriter};

/// Takes Lua byte code in a supported format and converts it to byte code in
/// the specified output [`Format`]. Returns [`LunifyError`] on error.
pub fn unify(input_bytes: &[u8], output_format: &Format, settings: &Settings) -> Result<Vec<u8>, LunifyError> {
    let mut byte_stream = ByteStream::new(input_bytes);

    if !byte_stream.remove_signature(settings.lua50.binary_signature) && !byte_stream.remove_signature(settings.lua51.binary_signature) {
        return Err(LunifyError::IncorrectSignature);
    }

    let version = byte_stream.byte()?.try_into()?;

    #[cfg(feature = "debug")]
    {
        println!("\n======== Header ========");
        println!("version: {version}");
    }

    let input_format = Format::from_byte_stream(&mut byte_stream, version, settings)?;

    // If the input is already in the correct format, return it as is.
    if input_format == *output_format && !cfg!(test) {
        #[cfg(feature = "debug")]
        println!("\n======== Done ========\n");

        return Ok(input_bytes.to_vec());
    }

    byte_stream.set_format(input_format);

    let root_function = Function::from_byte_stream(&mut byte_stream, version, settings)?;

    if !byte_stream.is_empty() {
        return Err(LunifyError::InputTooLong);
    }

    let mut byte_writer = ByteWriter::new(output_format);

    byte_writer.slice(settings.output.binary_signature.as_bytes());
    byte_writer.byte(LuaVersion::Lua51.into());
    output_format.write(&mut byte_writer);
    root_function.write(&mut byte_writer)?;

    #[cfg(feature = "debug")]
    println!("======== Done ========\n");

    Ok(byte_writer.finalize())
}

#[cfg(test)]
mod tests {
    use super::{unify, Format, LunifyError};
    use crate::{lua51, BitWidth, Endianness, Settings};

    #[cfg(feature = "integration")]
    fn test_output(byte_code: &[u8]) {
        use mlua::prelude::*;

        let lua = Lua::new();
        lua.load(byte_code).exec().unwrap();
        assert_eq!(lua.globals().get::<_, LuaNumber>("result").unwrap(), 9.0);
    }

    #[test]
    fn _32bit_to_64bit() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/32bit.luab");
        let output_format = Format {
            size_t_width: BitWidth::Bit64,
            ..Default::default()
        };
        unify(input_bytes, &output_format, &Default::default())?;

        Ok(())
    }

    #[test]
    fn lua50_to_lua51() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/lua50.luab");
        let output_format = Format::default();
        let _output_bytes = unify(input_bytes, &output_format, &Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&_output_bytes);
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
        let output_bytes = unify(input_bytes, &output_format, &Default::default())?;

        assert_eq!(&input_bytes[..], &output_bytes);
        Ok(())
    }

    #[test]
    fn little_endian() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/little_endian.luab");
        let output_format = Format::default();
        let _output_bytes = unify(input_bytes, &output_format, &Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&_output_bytes);
        Ok(())
    }

    #[test]
    fn big_endian() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/big_endian.luab");
        let output_format = Format::default();
        let _output_bytes = unify(input_bytes, &output_format, &Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&_output_bytes);
        Ok(())
    }

    #[test]
    fn large_table() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/large_table.luab");
        let output_format = Format::default();
        let _output_bytes = unify(input_bytes, &output_format, &Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&_output_bytes);
        Ok(())
    }

    #[test]
    fn dynamic_table() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/dynamic_table.luab");
        let output_format = Format::default();
        let _output_bytes = unify(input_bytes, &output_format, &Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&_output_bytes);
        Ok(())
    }

    #[test]
    fn variadic() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/variadic.luab");
        let output_format = Format::default();
        let _output_bytes = unify(input_bytes, &output_format, &Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&_output_bytes);
        Ok(())
    }

    #[test]
    fn for_loop() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/for_loop.luab").to_vec();
        let output_format = Format::default();
        let _output_bytes = unify(&input_bytes, &output_format, &Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&_output_bytes);
        Ok(())
    }

    #[test]
    fn constants() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/constants.luab").to_vec();
        let output_format = Format::default();
        let _output_bytes = unify(&input_bytes, &output_format, &Default::default())?;

        #[cfg(feature = "integration")]
        test_output(&_output_bytes);
        Ok(())
    }

    #[test]
    fn custom_signature() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/custom_signature.luab").to_vec();
        let output_format = Format::default();

        let settings = Settings {
            lua51: lua51::Settings {
                binary_signature: "\x1bLul",
                ..Default::default()
            },
            ..Default::default()
        };

        let _output_bytes = unify(&input_bytes, &output_format, &settings)?;

        #[cfg(feature = "integration")]
        test_output(&_output_bytes);
        Ok(())
    }

    #[test]
    fn empty() -> Result<(), LunifyError> {
        let input_bytes = include_bytes!("../test_files/empty.luab");
        let output_format = Format::default();
        unify(input_bytes, &output_format, &Default::default())?;
        Ok(())
    }

    #[test]
    fn incorrect_signature() {
        let output_format = Format::default();
        let result = unify(b"\x1bLuo", &output_format, &Default::default());

        assert_eq!(result, Err(LunifyError::IncorrectSignature));
    }

    #[test]
    fn input_too_long() {
        let mut input_bytes = include_bytes!("../test_files/empty.luab").to_vec();
        input_bytes.extend_from_slice(b"extra bytes");

        let output_format = Format::default();
        let result = unify(&input_bytes, &output_format, &Default::default());

        assert_eq!(result, Err(LunifyError::InputTooLong));
    }
}
