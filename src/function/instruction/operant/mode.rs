use super::OperantLayout;
use crate::{lua50, lua51, LunifyError, Settings};

pub(crate) trait ModeGet<T> {
    fn get(value: u64, settings: &Settings, layout: &OperantLayout) -> Self;
}

pub(crate) trait ModePut {
    fn put(&self, settings: &Settings, layout: &OperantLayout) -> Result<u64, LunifyError>;
}

pub(crate) trait ModeOffset {
    fn offset(&mut self, _stack_start: u64, _offset: i64) {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Unused;

impl<T> ModeGet<T> for Unused {
    fn get(_value: u64, _settings: &Settings, _layout: &OperantLayout) -> Self {
        Unused
    }
}

impl ModePut for Unused {
    fn put(&self, _settings: &Settings, _layout: &OperantLayout) -> Result<u64, LunifyError> {
        Ok(0)
    }
}

impl ModeOffset for Unused {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Generic(pub u64);

impl<T> ModeGet<T> for Generic {
    fn get(value: u64, _settings: &Settings, layout: &OperantLayout) -> Self {
        Generic(layout.get(value))
    }
}

impl ModePut for Generic {
    fn put(&self, _settings: &Settings, layout: &OperantLayout) -> Result<u64, LunifyError> {
        layout.put(self.0)
    }
}

impl ModeOffset for Generic {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Register(pub u64);

impl<T> ModeGet<T> for Register {
    fn get(value: u64, _settings: &Settings, layout: &OperantLayout) -> Self {
        Register(layout.get(value))
    }
}

impl ModePut for Register {
    fn put(&self, _settings: &Settings, layout: &OperantLayout) -> Result<u64, LunifyError> {
        layout.put(self.0)
    }
}

impl ModeOffset for Register {
    fn offset(&mut self, stack_start: u64, offset: i64) {
        if self.0 >= stack_start {
            self.0 = (self.0 as i64 + offset) as u64;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ConstantRegister(pub u64, pub bool);

impl ModeGet<lua50::Instruction> for ConstantRegister {
    fn get(value: u64, settings: &Settings, layout: &OperantLayout) -> Self {
        let mut value = layout.get(value);
        let is_constant = value >= settings.lua50.stack_limit;

        if is_constant {
            value -= settings.lua50.stack_limit;
        }

        ConstantRegister(value, is_constant)
    }
}

impl ModeGet<lua51::Instruction> for ConstantRegister {
    fn get(value: u64, settings: &Settings, layout: &OperantLayout) -> Self {
        let mut value = layout.get(value);
        let constant_bit = settings.lua51.get_constant_bit();
        let is_constant = value & constant_bit != 0;

        if is_constant {
            value ^= constant_bit;
        }

        ConstantRegister(value, is_constant)
    }
}

impl ModePut for ConstantRegister {
    fn put(&self, settings: &Settings, layout: &OperantLayout) -> Result<u64, LunifyError> {
        if self.0 > settings.output.get_maximum_constant_index() {
            return Err(LunifyError::ValueTooTooBigForOperant);
        }

        let value = match self.1 {
            true => self.0 | settings.output.get_constant_bit(),
            false => self.0,
        };

        layout.put(value)
    }
}

impl ModeOffset for ConstantRegister {
    fn offset(&mut self, stack_start: u64, offset: i64) {
        if self.0 >= stack_start && !self.1 {
            self.0 = (self.0 as i64 + offset) as u64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConstantRegister, Generic, ModeGet, ModeOffset, Register, Unused};
    use crate::function::instruction::operant::mode::ModePut;
    use crate::{lua50, lua51, LunifyError, Settings};

    fn mode_test_get<T, L>(value: u64, expected: T)
    where
        T: ModeGet<L> + Eq + std::fmt::Debug,
    {
        let settings = Settings::default();
        let result: T = ModeGet::<L>::get(value << 6, &settings, &settings.lua50.layout.c);
        assert_eq!(result, expected);
    }

    fn mode_test_put<T>(value: T, expected: Result<u64, LunifyError>)
    where
        T: ModePut + Eq + std::fmt::Debug,
    {
        let settings = Settings::default();
        let result = value.put(&settings, &settings.lua50.layout.c);
        assert_eq!(result.map(|value| value >> 6), expected);
    }

    fn mode_test_offset<T>(mut value: T, offset: i64, expected: T)
    where
        T: ModeOffset + Eq + std::fmt::Debug,
    {
        value.offset(0, offset);
        assert_eq!(value, expected);
    }

    #[test]
    fn unused_get() {
        mode_test_get::<_, ()>(0, Unused);
    }

    #[test]
    fn unused_put() {
        mode_test_put(Unused, Ok(0));
    }

    #[test]
    fn generic_get() {
        mode_test_get::<_, ()>(1, Generic(1));
    }

    #[test]
    fn generic_put() {
        mode_test_put(Generic(1), Ok(1));
    }

    #[test]
    fn generic_offset() {
        mode_test_offset(Generic(4), 5, Generic(4));
    }

    #[test]
    fn register_get() {
        mode_test_get::<_, ()>(1, Register(1));
    }

    #[test]
    fn register_put() {
        mode_test_put(Register(1), Ok(1));
    }

    #[test]
    fn register_offset() {
        mode_test_offset(Register(4), 5, Register(9));
    }

    #[test]
    fn constant_register_get() {
        mode_test_get::<_, lua50::Instruction>(1, ConstantRegister(1, false));
    }

    #[test]
    fn constant_register_get_constant_lua_50() {
        let settings = Settings::default();
        mode_test_get::<_, lua50::Instruction>(1 + settings.lua50.stack_limit, ConstantRegister(1, true));
    }

    #[test]
    fn constant_register_get_constant_lua_51() {
        let settings = Settings::default();
        mode_test_get::<_, lua51::Instruction>(1 | settings.lua51.get_constant_bit(), ConstantRegister(1, true));
    }

    #[test]
    fn constant_register_put() {
        mode_test_put(ConstantRegister(1, false), Ok(1));
    }

    #[test]
    fn constant_register_put_value_too_big() {
        let settings = Settings::default();
        mode_test_put(
            ConstantRegister(1 + settings.output.get_maximum_constant_index(), false),
            Err(LunifyError::ValueTooTooBigForOperant),
        );
    }

    #[test]
    fn constant_register_offset() {
        mode_test_offset(ConstantRegister(4, false), 5, ConstantRegister(9, false));
    }

    #[test]
    fn constant_register_offset_constant() {
        mode_test_offset(ConstantRegister(4, true), 5, ConstantRegister(4, true));
    }
}
