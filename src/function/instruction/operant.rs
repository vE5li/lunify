#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{lua50, lua51, LunifyError, Settings};

/// All possible operants of an instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OperantType {
    /// Size of the Opcode (`SIZE_O`P).
    Opcode(u64),
    /// Size of the A operant (`SIZE_A`).
    A(u64),
    /// Size of the B operant (`SIZE_B`).
    B(u64),
    /// Size of the C operant (`SIZE_C`).
    C(u64),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct OperantLayout {
    pub(crate) size: u64,
    pub(crate) position: u64,
    pub(crate) bit_mask: u64,
}

impl OperantLayout {
    pub(crate) fn new(size: u64, position: u64) -> Self {
        let bit_mask = !0 >> (64 - size);
        Self { size, position, bit_mask }
    }

    pub(crate) fn get(&self, value: u64) -> u64 {
        (value >> self.position) & self.bit_mask
    }

    pub(crate) fn put(&self, value: u64) -> Result<u64, LunifyError> {
        let maximum_value = (1 << self.size) - 1;

        if value > maximum_value {
            return Err(LunifyError::ValueTooTooBigForOperant);
        }

        Ok((value & self.bit_mask) << self.position)
    }
}

/// Memory layout of instructions inside the Lua bytecode (`SIZE_*`, `POS_*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InstructionLayout {
    pub(crate) opcode: OperantLayout,
    pub(crate) a: OperantLayout,
    pub(crate) b: OperantLayout,
    pub(crate) c: OperantLayout,
    pub(crate) bx: OperantLayout,
    pub(crate) signed_offset: i64,
}

impl InstructionLayout {
    /// Create a memory layout from a list of [OperantType]s.
    ///# Example
    ///
    ///```rust
    /// use lunify::{InstructionLayout, OperantType};
    ///
    /// // This is how the Lua 5.0 InstructionLayout is created.
    /// let layout = InstructionLayout::from_specification([
    ///     OperantType::Opcode(6),
    ///     OperantType::C(9),
    ///     OperantType::B(9),
    ///     OperantType::A(8)
    /// ]);
    /// ```
    pub fn from_specification(specification: [OperantType; 4]) -> Result<Self, LunifyError> {
        let mut opcode = None;
        let mut a = None;
        let mut b = None;
        let mut c = None;
        let mut offset = 0;

        for operant in specification {
            match operant {
                OperantType::Opcode(size) => {
                    // The minimum is 6 because there are 38 instructions in Lua 5.1, so we need a
                    // minimum of ceil(log2(38)) bits to represent them all.
                    if !(6..32).contains(&size) || opcode.is_some() {
                        return Err(LunifyError::InvalidInstructionLayout);
                    }

                    opcode = Some(OperantLayout::new(size, offset));
                    offset += size;
                }
                OperantType::A(size) => {
                    // The minimum is 7 because A needs to be able to hold values up to
                    // Lua 5.1 `MAXSTACK`.
                    if !(7..32).contains(&size) || a.is_some() {
                        return Err(LunifyError::InvalidInstructionLayout);
                    }

                    a = Some(OperantLayout::new(size, offset));
                    offset += size;
                }
                OperantType::B(size) => {
                    // The minimum is 8 because B needs to be able to hold values up to
                    // Lua 5.1 `MAXSTACK`, plus an additional bit for constant values.
                    if !(8..32).contains(&size) || b.is_some() {
                        return Err(LunifyError::InvalidInstructionLayout);
                    }

                    b = Some(OperantLayout::new(size, offset));
                    offset += size;
                }
                OperantType::C(size) => {
                    // The minimum is 8 because C needs to be able to hold values up to
                    // Lua 5.1 `MAXSTACK`, plus an additional bit for constant values.
                    if !(8..32).contains(&size) || c.is_some() {
                        return Err(LunifyError::InvalidInstructionLayout);
                    }

                    c = Some(OperantLayout::new(size, offset));
                    offset += size;
                }
            }
        }

        let opcode = opcode.unwrap();
        let a = a.unwrap();
        let b = b.unwrap();
        let c = c.unwrap();

        // Make sure that B and C are next to each other.
        if c.position + c.size != b.position && b.position + b.size != c.position {
            return Err(LunifyError::InvalidInstructionLayout);
        }

        let bx_size = b.size + c.size;
        let bx_position = u64::min(b.position, c.position);
        let bx = OperantLayout::new(bx_size, bx_position);
        let signed_offset = (!0u64 >> (64 - bx_size + 1)) as i64;

        Ok(Self {
            opcode,
            a,
            b,
            c,
            bx,
            signed_offset,
        })
    }
}

pub(crate) trait OperantGet<T> {
    fn get(value: u64, settings: &Settings, layout: &InstructionLayout) -> Self;
}

pub(crate) trait OperantPut {
    fn put(self, settings: &Settings) -> Result<u64, LunifyError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Opcode(pub u64);

impl<T> OperantGet<T> for Opcode {
    fn get(value: u64, _settings: &Settings, layout: &InstructionLayout) -> Self {
        Self(layout.opcode.get(value))
    }
}

impl OperantPut for Opcode {
    fn put(self, settings: &Settings) -> Result<u64, LunifyError> {
        settings.output.layout.opcode.put(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct A(pub u64);

impl<T> OperantGet<T> for A {
    fn get(value: u64, _settings: &Settings, layout: &InstructionLayout) -> Self {
        Self(layout.a.get(value))
    }
}

impl OperantPut for A {
    fn put(self, settings: &Settings) -> Result<u64, LunifyError> {
        settings.output.layout.a.put(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BC(pub u64, pub u64);

impl BC {
    pub fn const_c(b: u64, c: u64, settings: &Settings) -> Self {
        Self(b, c | settings.output.get_constant_bit())
    }
}

impl OperantGet<lua50::Instruction> for BC {
    fn get(value: u64, settings: &Settings, layout: &InstructionLayout) -> Self {
        let process = |value: u64| {
            if value >= settings.lua50.stack_limit {
                let constant_index = value - settings.lua50.stack_limit;
                return constant_index | settings.output.get_constant_bit();
            }
            value
        };

        let b = process(layout.b.get(value));
        let c = process(layout.c.get(value));
        Self(b, c)
    }
}

impl OperantGet<lua51::Instruction> for BC {
    fn get(value: u64, settings: &Settings, layout: &InstructionLayout) -> Self {
        let lua51_constant_bit = settings.lua51.get_constant_bit();
        let process = |value: u64| {
            if value & lua51_constant_bit != 0 {
                // Clear the Lua 5.1 constant bit.
                let constant_index = value ^ lua51_constant_bit;
                return constant_index | settings.output.get_constant_bit();
            }
            value
        };

        let b = process(layout.b.get(value));
        let c = process(layout.c.get(value));
        Self(b, c)
    }
}

impl OperantPut for BC {
    fn put(self, settings: &Settings) -> Result<u64, LunifyError> {
        Ok(settings.output.layout.b.put(self.0)? | settings.output.layout.c.put(self.1)?)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Bx(pub u64);

impl<T> OperantGet<T> for Bx {
    fn get(value: u64, _settings: &Settings, layout: &InstructionLayout) -> Self {
        Self(layout.bx.get(value))
    }
}

impl OperantPut for Bx {
    fn put(self, settings: &Settings) -> Result<u64, LunifyError> {
        settings.output.layout.bx.put(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SignedBx(pub i64);

impl<T> OperantGet<T> for SignedBx {
    fn get(value: u64, _settings: &Settings, layout: &InstructionLayout) -> Self {
        Self(layout.bx.get(value) as i64 - layout.signed_offset)
    }
}

impl OperantPut for SignedBx {
    fn put(self, settings: &Settings) -> Result<u64, LunifyError> {
        settings
            .output
            .layout
            .bx
            .put((self.0 + settings.output.layout.signed_offset) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::{Opcode, OperantGet, OperantLayout, OperantPut, A};
    use crate::function::instruction::{Bx, SignedBx, BC};
    use crate::{lua50, lua51, InstructionLayout, LunifyError, OperantType, Settings};

    #[test]
    fn layout_new() {
        let layout = OperantLayout::new(8, 6);
        let expected = OperantLayout {
            size: 8,
            position: 6,
            bit_mask: 0b11111111,
        };

        assert_eq!(layout, expected);
    }

    #[test]
    fn layout_get() {
        let layout = OperantLayout::new(2, 2);
        assert_eq!(layout.get(0b11100), 0b11);
    }

    #[test]
    fn layout_put() {
        let layout = OperantLayout::new(2, 2);
        assert_eq!(layout.put(0b11), Ok(0b1100));
    }

    #[test]
    fn layout_put_out_of_bounds() {
        let layout = OperantLayout::new(2, 2);
        assert_eq!(layout.put(0b111), Err(LunifyError::ValueTooTooBigForOperant));
    }

    #[test]
    fn from_specification() -> Result<(), LunifyError> {
        let layout =
            InstructionLayout::from_specification([OperantType::Opcode(6), OperantType::C(9), OperantType::B(9), OperantType::A(8)])?;

        assert_eq!(layout.opcode.position, 0);
        assert_eq!(layout.a.position, 24);
        assert_eq!(layout.b.position, 15);
        assert_eq!(layout.c.position, 6);
        assert_eq!(layout.bx.position, 6);
        assert_eq!(layout.bx.size, layout.b.size + layout.c.size);
        assert_eq!(layout.signed_offset, 131071);
        Ok(())
    }

    #[test]
    fn from_specification_opcode_twice() {
        let result =
            InstructionLayout::from_specification([OperantType::Opcode(6), OperantType::Opcode(6), OperantType::B(9), OperantType::A(8)]);
        assert_eq!(result, Err(LunifyError::InvalidInstructionLayout));
    }

    #[test]
    fn from_specification_a_twice() {
        let result =
            InstructionLayout::from_specification([OperantType::Opcode(6), OperantType::A(8), OperantType::B(9), OperantType::A(8)]);
        assert_eq!(result, Err(LunifyError::InvalidInstructionLayout));
    }

    #[test]
    fn from_specification_b_twice() {
        let result =
            InstructionLayout::from_specification([OperantType::Opcode(6), OperantType::B(9), OperantType::B(9), OperantType::A(8)]);
        assert_eq!(result, Err(LunifyError::InvalidInstructionLayout));
    }

    #[test]
    fn from_specification_c_twice() {
        let result =
            InstructionLayout::from_specification([OperantType::Opcode(6), OperantType::C(9), OperantType::C(9), OperantType::A(8)]);
        assert_eq!(result, Err(LunifyError::InvalidInstructionLayout));
    }

    #[test]
    fn from_specification_b_and_c_seperated() {
        let result =
            InstructionLayout::from_specification([OperantType::Opcode(6), OperantType::C(9), OperantType::A(9), OperantType::B(8)]);
        assert_eq!(result, Err(LunifyError::InvalidInstructionLayout));
    }

    fn operant_test<T>(operant: T, value: u64)
    where
        T: OperantGet<lua51::Instruction> + OperantPut + Eq + std::fmt::Debug,
    {
        let settings = Settings::default();
        assert_eq!(T::get(value, &settings, &settings.lua51.layout), operant);
        assert_eq!(operant.put(&settings), Ok(value));
    }

    #[test]
    fn operant_opcode() {
        operant_test(Opcode(1), 1);
    }

    #[test]
    fn operant_a() {
        operant_test(A(1), 1 << 6);
    }

    #[test]
    fn operant_b() {
        operant_test(BC(1, 0), 1 << 23);
    }

    #[test]
    fn operant_c() {
        operant_test(BC(0, 1), 1 << 14);
    }

    #[test]
    fn operant_c_const_lua50() {
        let settings = Settings::default();
        let value = (1 + settings.lua50.stack_limit) << 6;
        let operant = BC::const_c(0, 1, &settings);

        let bc = <BC as OperantGet<lua50::Instruction>>::get(value, &settings, &settings.lua50.layout);
        assert_eq!(bc, operant);
    }

    #[test]
    fn operant_c_const_lua51() {
        let mut settings = Settings::default();
        // Set the size of B for Lua 5.1 to something smaller than the size of B for
        // output, so we can test that the constant bit is cleared and set
        // correctly.
        settings.lua51.layout.b.size = 5;

        let value = (1 | settings.lua51.get_constant_bit()) << 14;
        let operant = BC::const_c(0, 1, &settings);

        let bc = <BC as OperantGet<lua51::Instruction>>::get(value, &settings, &settings.lua51.layout);
        assert_eq!(bc, operant);
    }

    #[test]
    fn operant_bc_new_const_c() {
        let settings = Settings::default();
        let bc = BC::const_c(0, 1, &settings);
        assert_eq!(bc.1, 1 | settings.output.get_constant_bit());
    }

    #[test]
    fn operant_bx() {
        operant_test(Bx(1), 1 << 14);
    }

    #[test]
    fn operant_signed_bx() {
        operant_test(SignedBx(1), 131072 << 14);
    }
}
