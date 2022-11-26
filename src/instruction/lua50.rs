#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::operant::{Bx, SignedBx, BC};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Settings {
    pub fields_per_flush: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self { fields_per_flush: 32 }
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
