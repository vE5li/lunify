#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::operant::{Bx, SignedBx, BC};

/// Lua 5.0 compile constants. The Lua interpreter is compiled with certain
/// predefined constants that affect how the bytecode is generated. This
/// structure represents a small subset of the constants that are relevant for
/// Lunify. If the bytecode you are trying to modify was complied with
/// non-standard constants, you can use these settings to make it compatible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Settings<'a> {
    /// Number of elements to put on the stack before inserting a SETLIST
    /// instruction (LFIELDS_PER_FLUSH).
    pub fields_per_flush: u64,
    /// Lua binary file signature (LUA_SIGNATURE).
    pub binary_signature: &'a str,
}

impl<'a> Default for Settings<'a> {
    fn default() -> Self {
        Self {
            fields_per_flush: 32,
            binary_signature: "\x1bLua",
        }
    }
}

lua_instructions! {
    Move(BC),
    LoadK(Bx),
    LoadBool(BC),
    LoadNil(BC),
    GetUpValue(BC),
    GetGlobal(Bx),
    GetTable(BC),
    SetGlobal(Bx),
    SetUpValue(BC),
    SetTable(BC),
    NewTable(BC),
    _Self(BC),
    Add(BC),
    Subtract(BC),
    Multiply(BC),
    Divide(BC),
    Power(BC),
    Unary(BC),
    Not(BC),
    Concatinate(BC),
    Jump(SignedBx),
    Equals(BC),
    LessThan(BC),
    LessEquals(BC),
    Test(BC),
    Call(BC),
    TailCall(BC),
    Return(BC),
    ForLoop(SignedBx),
    TForLoop(BC),
    TForPrep(SignedBx),
    SetList(Bx),
    SetListO(Bx),
    Close(BC),
    Closure(Bx),
}
