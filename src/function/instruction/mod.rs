#[macro_use]
mod macros;
mod interface;
/// Lua 5.0 settings.
pub mod lua50;
/// Lua 5.1 settings.
pub mod lua51;
mod operand;
mod settings;

pub(crate) use self::interface::LuaInstruction;
pub(crate) use self::operand::{Bx, ConstantRegister, Generic, Register, SignedBx, Unused, BC};
pub use self::operand::{InstructionLayout, OperandType};
pub use self::settings::Settings;
