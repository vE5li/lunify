use crate::{Format, LunifyError};

pub(crate) struct ByteWriter {
    data: Vec<u8>,
    format: Format,
}

impl ByteWriter {

    pub fn new(format: Format) -> Result<Self, LunifyError> {

        format.assert_supported()?;

        let data = Vec::new();

        Ok(Self { data, format })
    }

    pub fn byte(&mut self, byte: u8) {
        self.data.push(byte);
    }

    pub fn slice(&mut self, slice: &[u8]) {
        self.data.extend_from_slice(slice);
    }

    pub fn integer(&mut self, value: i64) {
        match self.format.integer_size {
            4 => self.slice(&(value as i32).to_le_bytes()),
            8 => self.slice(&value.to_le_bytes()),
            _ => unreachable!(),
        }
    }

    pub fn size_t(&mut self, value: i64) {
        match self.format.size_t_size {
            4 => self.slice(&(value as i32).to_le_bytes()),
            8 => self.slice(&value.to_le_bytes()),
            _ => unreachable!(),
        }
    }

    pub fn instruction(&mut self, instruction: u64) {
        match self.format.instruction_size {
            4 => self.slice(&(instruction as u32).to_le_bytes()),
            _ => unreachable!(),
        }
    }

    pub fn number(&mut self, value: i64) {
        match self.format.number_size {
            4 => self.slice(&(value as i32).to_le_bytes()),
            8 => self.slice(&value.to_le_bytes()),
            _ => unreachable!(),
        }
    }

    pub fn string(&mut self, value: &str) {

        self.size_t(value.len() as i64);
        self.slice(value.as_bytes());
    }

    pub fn finalize(self) -> Vec<u8> {
        self.data
    }
}
