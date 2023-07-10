use crate::{lua50, lua51, LunifyError, Settings};

mod layout;
mod mode;

pub(crate) use self::layout::OperandLayout;
pub use self::layout::{InstructionLayout, OperandType};
pub(crate) use self::mode::{ConstantRegister, Generic, Register, Unused};
use self::mode::{ModeGet, ModeOffset, ModePut};

pub(crate) trait OperandGet<T> {
    fn get(value: u64, settings: &Settings, layout: &InstructionLayout) -> Self;
}

pub(crate) trait OperandPut {
    fn put(self, settings: &Settings) -> Result<u64, LunifyError>;
}

pub(crate) trait OperandOffset {
    fn offset(&mut self, _stack_start: u64, _offset: i64) {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Opcode(pub u64);

impl<T> OperandGet<T> for Opcode {
    fn get(value: u64, _settings: &Settings, layout: &InstructionLayout) -> Self {
        Self(layout.opcode.get(value))
    }
}

impl OperandPut for Opcode {
    fn put(self, settings: &Settings) -> Result<u64, LunifyError> {
        settings.output.layout.opcode.put(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct A(pub u64);

impl<T> OperandGet<T> for A {
    fn get(value: u64, _settings: &Settings, layout: &InstructionLayout) -> Self {
        Self(layout.a.get(value))
    }
}

impl OperandPut for A {
    fn put(self, settings: &Settings) -> Result<u64, LunifyError> {
        settings.output.layout.a.put(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BC<B, C>(pub B, pub C);

impl<B, C> OperandGet<lua50::Instruction> for BC<B, C>
where
    B: ModeGet<lua50::Instruction>,
    C: ModeGet<lua50::Instruction>,
{
    fn get(value: u64, settings: &Settings, layout: &InstructionLayout) -> Self {
        let b = B::get(value, settings, &layout.b);
        let c = C::get(value, settings, &layout.c);
        Self(b, c)
    }
}

impl<B, C> OperandGet<lua51::Instruction> for BC<B, C>
where
    B: ModeGet<lua51::Instruction>,
    C: ModeGet<lua51::Instruction>,
{
    fn get(value: u64, settings: &Settings, layout: &InstructionLayout) -> Self {
        let b = B::get(value, settings, &layout.b);
        let c = C::get(value, settings, &layout.c);
        Self(b, c)
    }
}

impl<B, C> OperandPut for BC<B, C>
where
    B: ModePut,
    C: ModePut,
{
    fn put(self, settings: &Settings) -> Result<u64, LunifyError> {
        let b = self.0.put(settings, &settings.output.layout.b)?;
        let c = self.1.put(settings, &settings.output.layout.c)?;
        Ok(b | c)
    }
}

impl<B, C> OperandOffset for BC<B, C>
where
    B: ModeOffset,
    C: ModeOffset,
{
    fn offset(&mut self, stack_start: u64, offset: i64) {
        self.0.offset(stack_start, offset);
        self.1.offset(stack_start, offset);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Bx(pub u64);

impl<T> OperandGet<T> for Bx {
    fn get(value: u64, _settings: &Settings, layout: &InstructionLayout) -> Self {
        Self(layout.bx.get(value))
    }
}

impl OperandPut for Bx {
    fn put(self, settings: &Settings) -> Result<u64, LunifyError> {
        settings.output.layout.bx.put(self.0)
    }
}

impl OperandOffset for Bx {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SignedBx(pub i64);

impl<T> OperandGet<T> for SignedBx {
    fn get(value: u64, _settings: &Settings, layout: &InstructionLayout) -> Self {
        Self(layout.bx.get(value) as i64 - layout.signed_offset)
    }
}

impl OperandPut for SignedBx {
    fn put(self, settings: &Settings) -> Result<u64, LunifyError> {
        settings
            .output
            .layout
            .bx
            .put((self.0 + settings.output.layout.signed_offset) as u64)
    }
}

impl OperandOffset for SignedBx {}

#[cfg(test)]
mod tests {
    use super::{Generic, Opcode, OperandGet, OperandOffset, OperandPut, Register, A};
    use crate::function::instruction::{Bx, ConstantRegister, SignedBx, BC};
    use crate::{lua50, lua51, InstructionLayout, Settings};

    fn operand_test<T>(operand: T, value: u64)
    where
        T: OperandGet<lua51::Instruction> + OperandPut + Eq + std::fmt::Debug,
    {
        let settings = Settings::default();
        assert_eq!(T::get(value, &settings, &settings.lua51.layout), operand);
        assert_eq!(operand.put(&settings), Ok(value));
    }

    fn asymmetric_operand_test<T, L>(settings: &Settings, layout: &InstructionLayout, operand: T, input_value: u64, output_value: u64)
    where
        T: OperandGet<L> + OperandPut + Eq + std::fmt::Debug,
    {
        assert_eq!(T::get(input_value, settings, layout), operand);
        assert_eq!(operand.put(settings), Ok(output_value));
    }

    fn operand_test_offset<T>(mut operand: T, offset: i64, expected: T)
    where
        T: OperandOffset + Eq + std::fmt::Debug,
    {
        operand.offset(0, offset);
        assert_eq!(operand, expected);
    }

    #[test]
    fn opcode() {
        operand_test(Opcode(1), 1);
    }

    #[test]
    fn a() {
        operand_test(A(1), 1 << 6);
    }

    #[test]
    fn b() {
        operand_test(BC(Generic(1), Generic(0)), 1 << 23);
    }

    #[test]
    fn b_offset() {
        operand_test_offset(BC(Register(4), Register(4)), 5, BC(Register(9), Register(9)));
    }

    #[test]
    fn c() {
        operand_test(BC(Generic(0), Generic(1)), 1 << 14);
    }

    #[test]
    fn c_const_lua50() {
        let settings = Settings::default();

        asymmetric_operand_test::<_, lua50::Instruction>(
            &settings,
            &settings.lua50.layout,
            BC(ConstantRegister(0, false), ConstantRegister(1, true)),
            (1 + settings.lua50.stack_limit) << 6,
            (1 | settings.output.get_constant_bit()) << 14,
        );
    }

    #[test]
    fn c_const_lua51() {
        let mut settings = Settings::default();
        // Set the size of B for Lua 5.1 to something smaller than the size of B for
        // output, so we can test that the constant bit is cleared and set
        // correctly.
        settings.lua51.layout.b.size = 5;

        asymmetric_operand_test::<_, lua51::Instruction>(
            &settings,
            &settings.lua51.layout,
            BC(ConstantRegister(0, false), ConstantRegister(1, true)),
            (1 | settings.lua51.get_constant_bit()) << 14,
            (1 | settings.output.get_constant_bit()) << 14,
        );
    }

    #[test]
    fn bx() {
        operand_test(Bx(1), 1 << 14);
    }

    #[test]
    fn bx_offset() {
        operand_test_offset(Bx(10), 5, Bx(10));
    }

    #[test]
    fn signed_bx() {
        operand_test(SignedBx(1), 131072 << 14);
    }

    #[test]
    fn signed_bx_offset() {
        operand_test_offset(SignedBx(10), 5, SignedBx(10));
    }
}
