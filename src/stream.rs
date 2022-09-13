use std::convert::TryInto;

use crate::{Format, LunifyError};

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

    pub fn set_format(&mut self, format: Format) -> Result<(), LunifyError> {

        format.assert_supported()?;
        self.format = format;
        Ok(())
    }

    pub fn byte(&mut self) -> Result<u8, LunifyError> {

        let offset = self.offset;
        self.offset += 1;
        self.data.get(offset).cloned().ok_or(LunifyError::TooShort)
    }

    fn number_from_slice(slice: &[u8]) -> i64 {
        match slice.len() {
            4 => i32::from_le_bytes(slice.try_into().unwrap()) as i64,
            8 => i64::from_le_bytes(slice.try_into().unwrap()),
            _ => unreachable!(),
        }
    }

    pub fn integer(&mut self) -> Result<i64, LunifyError> {

        let slice = self.slice(self.format.integer_size as usize)?;
        Ok(Self::number_from_slice(slice))
    }

    pub fn size_t(&mut self) -> Result<i64, LunifyError> {

        let slice = self.slice(self.format.size_t_size as usize)?;
        Ok(Self::number_from_slice(slice))
    }

    pub fn instruction(&mut self) -> Result<u64, LunifyError> {

        let slice = self.slice(self.format.instruction_size as usize)?;
        match slice.len() {
            4 => Ok(u32::from_le_bytes(slice.try_into().unwrap()) as u64),
            8 => Ok(u64::from_le_bytes(slice.try_into().unwrap())),
            _ => unreachable!(),
        }
    }

    pub fn number(&mut self) -> Result<i64, LunifyError> {

        let slice = self.slice(self.format.number_size as usize)?;
        let integer = i64::from_le_bytes(slice.try_into().unwrap());
        Ok(integer)
    }

    pub fn slice(&mut self, length: usize) -> Result<&[u8], LunifyError> {

        let start = self.offset;
        self.offset += length;

        if self.offset > self.data.len() {
            return Err(LunifyError::TooShort);
        }

        Ok(&self.data[start..self.offset])
    }

    pub fn string(&mut self) -> Result<String, LunifyError> {

        let length = self.size_t()? as usize;
        let start = self.offset;

        self.offset += length;

        Ok(self.data[start..self.offset].iter().map(|&byte| byte as char).collect())
    }

    pub fn is_empty(&self) -> bool {
        self.offset >= self.data.len()
    }

    pub fn set_number_format(&mut self, endianness: u8, number_size: u8) {

        self.format.endianness = endianness;
        self.format.number_size = number_size;
    }
}
