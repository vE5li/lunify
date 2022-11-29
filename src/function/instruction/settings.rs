#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{lua50, lua51};

/// Lua 5.0 and Lua 5.1 compile constants. The Lua interpreter is compiled with
/// certain predefined constants that affect how the bytecode is generated. This
/// structure represents a small subset of the constants that are relevant for
/// Lunify. If the bytecode you are trying to modify was complied with
/// non-standard constants, you can use these settings to make it compatible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Settings {
    /// Lua 5.0 compile constants.
    pub lua50: lua50::Settings,
    /// Lua 5.1 compile constants.
    pub lua51: lua51::Settings,
}
