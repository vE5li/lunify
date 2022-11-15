use std::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Settings {
    pub fields_per_flush: u64,
    pub stack_limit: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fields_per_flush: 50,
            stack_limit: 250,
        }
    }
}

opcodes! {
    Move,
    LoadK,
    LoadBool,
    LoadNil,
    GetUpValue,
    GetGlobal,
    GetTable,
    SetGlobal,
    SetUpValue,
    SetTable,
    NewTable,
    _Self,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Unary,
    Not,
    Length,
    Concatinate,
    Jump,
    Equals,
    LessThan,
    LessEquals,
    Test,
    TestSet,
    Call,
    TailCall,
    Return,
    ForLoop,
    ForPrep,
    TForLoop,
    SetList,
    Close,
    Closure,
    VarArg,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(super) struct Instruction {
    pub(super) opcode: Opcode,
    pub(super) a: u64,
    pub(super) b: u64,
    pub(super) c: u64,
    pub(super) bx: u64,
    pub(super) is_bx: bool,
}

impl Instruction {
    pub(super) fn new(opcode: Opcode, a: u64, b: u64, c: u64) -> Self {
        #[cfg(feature = "debug")]
        {
            println!("=== generated ===");
            println!("Opcode {:?}", opcode);
            println!("A {}", a);
            println!("B {}", b);
            println!("C {}", c);
        }

        Self {
            opcode,
            a,
            b,
            c,
            bx: 0,
            is_bx: false,
        }
    }

    pub(super) fn new_bx(opcode: Opcode, a: u64, bx: u64) -> Self {
        Self {
            opcode,
            a,
            b: 0,
            c: 0,
            bx,
            is_bx: true,
        }
    }

    pub(super) fn stack_destination(&self) -> Option<Range<u64>> {
        match self.opcode {
            Opcode::SetGlobal => None,
            Opcode::SetUpValue => None,
            Opcode::_Self => Some(self.a..self.a + 1),
            Opcode::Jump => None,
            Opcode::Equals => None,
            Opcode::LessThan => None,
            Opcode::LessEquals => None,
            Opcode::Test => None,
            Opcode::Call => Some(self.a..self.a + self.c - 1),
            Opcode::TailCall => None,
            Opcode::Return => None,
            Opcode::ForLoop => Some(self.a..self.a + 3),
            Opcode::ForPrep => Some(self.a..self.a + 2),
            Opcode::TForLoop => Some(self.a..self.a + 2 + self.c),
            Opcode::Close => None,
            Opcode::VarArg => Some(self.a..self.a.max((self.a + self.b).saturating_sub(1))),
            _ => Some(self.a..self.a),
        }
    }

    pub(super) fn move_stack_accesses(&mut self, offset: i64) {
        let offset = |position| (position as i64 + offset) as u64;

        // FIX: correct this list
        match self.opcode {
            Opcode::Jump => {}
            Opcode::Equals => {}
            Opcode::LessThan => {}
            Opcode::LessEquals => {}
            _ => self.a = offset(self.a),
        }
    }

    pub(super) fn to_u64(mut self) -> u64 {
        if self.is_bx {
            let mut instruction = self.opcode as u64;

            instruction |= self.a << 6;
            instruction |= self.bx << 14;

            //#[cfg(feature = "debug")]
            //println!("{:0>32b}\n", instruction);

            instruction
        } else {
            let mut instruction = self.opcode as u64;
            instruction |= self.a << 6;

            // HACK: Implement correct logic
            if self.b > 200 {
                self.b = (self.b + 6) & 0b111111111;
            }

            // HACK: Implement correct logic
            if self.c > 200 {
                self.c = (self.c + 6) & 0b111111111;
            }

            instruction |= self.c << 14;
            instruction |= self.b << 23;

            //#[cfg(feature = "debug")]
            //println!("{:0>32b}\n", instruction);

            instruction
        }
    }
}
