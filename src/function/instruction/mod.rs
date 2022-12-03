#[macro_use]
mod macros;
mod interface;
/// Lua 5.0 settings.
pub mod lua50;
/// Lua 5.1 settings.
pub mod lua51;
mod operant;
mod settings;

pub(crate) use self::interface::LuaInstruction;
pub(crate) use self::operant::{Bx, ConstantRegister, Generic, Register, SignedBx, Unused, BC};
pub use self::operant::{InstructionLayout, OperantType};
pub use self::settings::Settings;
