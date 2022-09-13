//! A crate for converting Lua bytecode to different versions and formats.
//!
//! # Usage
//!
//! This crate only exposes one function and two types. The function is called `unify` and it is used to
//! convert Lua bytecode from a supported format to a specified output format.
//!
//! Currently only Lua 5.0 and Lua 5.1 are supported inputs

mod error;
mod format;
mod function;
mod stream;
mod writer;

pub use error::LunifyError;
pub use format::Format;
use function::Function;
use stream::ByteStream;
use writer::ByteWriter;

const LUA_SIGNATURE: &[u8; 4] = b"\x1bLua";

/// Takes Lua bytecode in a supported format as a `Vec` and converts it to bytecode in the specified output
/// [Format]. Returns [LunifyError] on error.
pub fn unify(input_bytes: Vec<u8>, output_format: Format) -> Result<Vec<u8>, LunifyError> {

    let mut byte_stream = ByteStream::new(&input_bytes);

    let signature = byte_stream.slice(LUA_SIGNATURE.len())?;

    if signature != LUA_SIGNATURE {
        return Err(LunifyError::IncorrectSignature);
    }

    let version = byte_stream.byte()?;

    #[cfg(feature = "debug")]
    println!("version: {:x}", version);

    if !(0x50..=0x51).contains(&version) {
        return Err(LunifyError::UnsupportedVersion(version));
    }

    let input_format = Format::from_byte_stream(&mut byte_stream, version)?;

    // if the input is already in the correct format, return it as is
    if input_format == output_format && !cfg!(test) {
        return Ok(input_bytes);
    }

    byte_stream.set_format(input_format)?;

    let root_function = Function::from_byte_stream(&mut byte_stream, version)?;

    if !byte_stream.is_empty() {
        return Err(LunifyError::TooLong);
    }

    let mut byte_writer = ByteWriter::new(output_format)?;

    byte_writer.slice(LUA_SIGNATURE);
    byte_writer.byte(0x51); // version

    output_format.write(&mut byte_writer);
    root_function.write(&mut byte_writer);

    Ok(byte_writer.finalize())
}

#[cfg(test)]
mod tests {

    use mlua::prelude::*;

    use super::{unify, Format, LunifyError};

    #[test]
    fn _32bit_to_64bit() -> Result<(), LunifyError> {

        let input_bytes = include_bytes!("../test_files/32bit.luab");
        let output_format = Format {
            size_t_size: 8,
            ..Default::default()
        };
        let _output_bytes = unify(input_bytes.to_vec(), output_format)?;

        Ok(())
    }

    #[test]
    fn lua50_to_lua51() -> Result<(), LunifyError> {

        let input_bytes = include_bytes!("../test_files/lua50.luab");
        let output_format = Format::default();
        let output_bytes = unify(input_bytes.to_vec(), output_format)?;

        let lua = Lua::new();
        lua.load(&output_bytes).exec().unwrap();
        assert_eq!(lua.globals().get::<_, LuaNumber>("bar").unwrap(), 14.0);

        Ok(())
    }

    #[test]
    fn matching_format_remains_unchanged() -> Result<(), LunifyError> {

        let input_bytes = include_bytes!("../test_files/32bit.luab");
        let output_format = Format {
            size_t_size: 4,
            ..Default::default()
        };
        let output_bytes = unify(input_bytes.to_vec(), output_format)?;

        assert_eq!(&input_bytes[..], &output_bytes[..]);

        Ok(())
    }
}
