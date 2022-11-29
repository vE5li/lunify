// Workaround untli from_le_bytes is part of a trait.
macro_rules! to_slice {
    ($writer:expr, $value:expr, $width:ident, $type32:ty) => {
        match ($writer.format.$width, $writer.format.endianness) {
            (BitWidth::Bit32, Endianness::Little) => $writer.slice(&($value as $type32).to_le_bytes()),
            (BitWidth::Bit32, Endianness::Big) => $writer.slice(&($value as $type32).to_be_bytes()),
            (BitWidth::Bit64, Endianness::Little) => $writer.slice(&$value.to_le_bytes()),
            (BitWidth::Bit64, Endianness::Big) => $writer.slice(&$value.to_be_bytes()),
        }
    };
}

// Workaround untli from_le_bytes is part of a trait.
macro_rules! from_slice {
    ($stream:expr, $width:expr, $endianness:expr, $type32:ty, $type64:ty) => {{
        let endianness = $endianness;
        let slice = $stream.slice(u8::from($width) as usize)?;

        match (slice.len(), endianness) {
            (4, Endianness::Little) => <$type32>::from_le_bytes(slice.try_into().unwrap()) as $type64,
            (4, Endianness::Big) => <$type32>::from_be_bytes(slice.try_into().unwrap()) as $type64,
            (8, Endianness::Little) => <$type64>::from_le_bytes(slice.try_into().unwrap()),
            (8, Endianness::Big) => <$type64>::from_be_bytes(slice.try_into().unwrap()),
            _ => unreachable!(),
        }
    }};
}

//pub(crate) use {to_slice, from_slice};
