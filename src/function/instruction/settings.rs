#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{lua50, lua51};

/// Lua 5.0 and Lua 5.1 compile constants. The Lua interpreter is compiled with
/// certain predefined constants that affect how the byte code is generated.
/// This structure represents a small subset of the constants that are relevant
/// for Lunify. If the byte code you are trying to modify was complied with
/// non-standard constants, you can use these settings to make it compatible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Settings<'a> {
    /// Lua 5.0 input compile constants.
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub lua50: lua50::Settings<'a>,
    /// Lua 5.1 input compile constants.
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub lua51: lua51::Settings<'a>,
    /// Emitted Lua 5.1 compile constants.
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub output: lua51::Settings<'a>,
}
