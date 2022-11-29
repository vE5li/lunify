use crate::{lua50, lua51};

pub(crate) trait Operant<T> {
    fn parse(value: u64) -> Self;
    fn write(self) -> u64;
}

// A register
impl Operant<lua50::Instruction> for u64 {
    fn parse(value: u64) -> Self {
        (value >> 24) & 0b11111111
    }

    fn write(self) -> u64 {
        panic!()
    }
}

impl Operant<lua51::Instruction> for u64 {
    fn parse(_value: u64) -> Self {
        panic!();
    }

    fn write(self) -> u64 {
        self << 6
    }
}

pub const LUA_CONST_BIT: u64 = 1 << 8;
const LUA50_MAXSTACK: u64 = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BC(pub u64, pub u64);

impl BC {
    pub fn const_c(b: u64, c: u64) -> Self {
        Self(b, c | LUA_CONST_BIT)
    }
}

impl Operant<lua50::Instruction> for BC {
    fn parse(value: u64) -> Self {
        let process = |value: u64| {
            let value = value & 0b111111111;

            if value >= LUA50_MAXSTACK {
                let constant_index = value - LUA50_MAXSTACK;
                return constant_index | LUA_CONST_BIT;
            }

            value
        };

        let b = process(value >> 15);
        let c = process(value >> 6);
        Self(b, c)
    }

    fn write(self) -> u64 {
        panic!()
    }
}

impl Operant<lua51::Instruction> for BC {
    fn parse(_value: u64) -> Self {
        panic!();
    }

    fn write(self) -> u64 {
        (self.0 << 23) | (self.1 << 14)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Bx(pub u64);

impl Operant<lua50::Instruction> for Bx {
    fn parse(value: u64) -> Self {
        Self((value >> 6) & 0b111111111111111111)
    }

    fn write(self) -> u64 {
        panic!()
    }
}

impl Operant<lua51::Instruction> for Bx {
    fn parse(_value: u64) -> Self {
        panic!();
    }

    fn write(self) -> u64 {
        self.0 << 14
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SignedBx(pub i64);

impl Operant<lua50::Instruction> for SignedBx {
    fn parse(value: u64) -> Self {
        let signed_offset = 0b11111111111111111;
        Self(((value as i64 >> 6) & 0b111111111111111111) - signed_offset)
    }

    fn write(self) -> u64 {
        panic!()
    }
}

impl Operant<lua51::Instruction> for SignedBx {
    fn parse(_value: u64) -> Self {
        panic!();
    }

    fn write(self) -> u64 {
        let signed_offset = 0b11111111111111111;
        ((self.0 + signed_offset) as u64) << 14
    }
}
