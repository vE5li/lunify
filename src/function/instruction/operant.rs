#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Settings;

/// Different possible operants of an instruction.
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
pub(crate) struct Layout {
    pub(crate) size: u64,
    pub(crate) position: u64,
    pub(crate) bit_mask: u64,
}

impl Layout {
    pub(crate) fn new(size: u64, position: u64) -> Self {
        let bit_mask = !0 >> (64 - size);
        Self { size, position, bit_mask }
    }

    pub(crate) fn get(&self, value: u64) -> u64 {
        (value >> self.position) & self.bit_mask
    }

    pub(crate) fn put(&self, value: u64) -> u64 {
        (value & self.bit_mask) << self.position
    }
}

/// Memory layout of instructions inside the Lua bytecode (`SIZE_*`, `POS_*`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InstructionLayout {
    pub(crate) opcode: Layout,
    pub(crate) a: Layout,
    pub(crate) b: Layout,
    pub(crate) c: Layout,
    pub(crate) bx: Layout,
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
    pub fn from_specification(specification: [OperantType; 4]) -> Self {
        let mut opcode = Layout::default();
        let mut a = Layout::default();
        let mut b = Layout::default();
        let mut c = Layout::default();
        let mut offset = 0;

        for operant in specification {
            match operant {
                OperantType::Opcode(size) => {
                    assert!((6..32).contains(&size)); // TODO: doulbe check
                    assert!(opcode == Layout::default());
                    opcode = Layout::new(size, offset);
                    offset += size;
                }
                OperantType::A(size) => {
                    assert!((8..32).contains(&size));
                    assert!(a == Layout::default());
                    a = Layout::new(size, offset);
                    offset += size;
                }
                OperantType::B(size) => {
                    assert!((8..32).contains(&size));
                    assert!(b == Layout::default());
                    b = Layout::new(size, offset);
                    offset += size;
                }
                OperantType::C(size) => {
                    assert!((8..32).contains(&size));
                    assert!(c == Layout::default());
                    c = Layout::new(size, offset);
                    offset += size;
                }
            }
        }

        let bx_size = b.size + c.size;
        let bx_position = u64::min(b.position, c.position);
        let bx = Layout::new(bx_size, bx_position);
        // TODO: assert that b and c are next to each other.

        let signed_offset = (!0u64 >> (64 - bx_size + 1)) as i64;

        Self {
            opcode,
            a,
            b,
            c,
            bx,
            signed_offset,
        }
    }
}

pub(crate) trait Operant {
    fn parse(value: u64, settings: &Settings, layout: &InstructionLayout) -> Self;
    fn write(self, settings: &Settings, layout: &InstructionLayout) -> u64;
}

// A
impl Operant for u64 {
    fn parse(value: u64, _settings: &Settings, layout: &InstructionLayout) -> Self {
        layout.a.get(value)
    }

    fn write(self, _settings: &Settings, layout: &InstructionLayout) -> u64 {
        layout.a.put(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BC(pub u64, pub u64);

impl BC {
    pub fn const_c(b: u64, c: u64, settings: &Settings) -> Self {
        Self(b, c | settings.lua51.get_constant_bit())
    }
}

impl Operant for BC {
    fn parse(value: u64, settings: &Settings, layout: &InstructionLayout) -> Self {
        let process = |value: u64| {
            if value >= settings.lua50.stack_limit {
                let constant_index = value - settings.lua50.stack_limit;
                return constant_index | settings.lua51.get_constant_bit();
            }
            value
        };

        let b = process(layout.b.get(value));
        let c = process(layout.c.get(value));
        Self(b, c)
    }

    fn write(self, _settings: &Settings, layout: &InstructionLayout) -> u64 {
        layout.b.put(self.0) | layout.c.put(self.1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Bx(pub u64);

impl Operant for Bx {
    fn parse(value: u64, _settings: &Settings, layout: &InstructionLayout) -> Self {
        Self(layout.bx.get(value))
    }

    fn write(self, _settings: &Settings, layout: &InstructionLayout) -> u64 {
        layout.bx.put(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SignedBx(pub i64);

impl Operant for SignedBx {
    fn parse(value: u64, _settings: &Settings, layout: &InstructionLayout) -> Self {
        Self(layout.bx.get(value) as i64 - layout.signed_offset)
    }

    fn write(self, _settings: &Settings, layout: &InstructionLayout) -> u64 {
        layout.bx.put((self.0 + layout.signed_offset) as u64)
    }
}
