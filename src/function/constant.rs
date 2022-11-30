use crate::number::Number;

pub(crate) enum Constant {
    Nil,
    Boolean(u8),
    Number(Number),
    String(String),
}

pub(super) struct ConstantManager<'a> {
    pub(super) constants: &'a mut Vec<Constant>,
}

impl<'a> ConstantManager<'a> {
    pub(super) fn create_unique(&mut self, program_counter: usize) -> u64 {
        let unique_constant = self.constants.len() as u64;
        let constant_name = format!("__%lunify%__temp{}\0", program_counter);
        // TODO: make sure this is actually unique.
        self.constants.push(Constant::String(constant_name));
        unique_constant
    }

    pub(super) fn constant_for_str(&mut self, constant_str: &'static str) -> u64 {
        let zero_terminated = format!("{}\0", constant_str);

        // If the constant already exists we don't need to add it again.
        let matches = |constant: &_| matches!(constant, Constant::String(string) if string == zero_terminated.as_str());
        if let Some(index) = self.constants.iter().position(matches) {
            return index as u64;
        }

        let constant = self.constants.len() as u64;
        self.constants.push(Constant::String(zero_terminated));
        constant
    }

    pub(super) fn constant_nil(&mut self) -> u64 {
        // If the constant already exists we don't need to add it again.
        let matches = |constant: &_| matches!(constant, Constant::Nil);
        if let Some(index) = self.constants.iter().position(matches) {
            return index as u64;
        }

        let constant = self.constants.len() as u64;
        self.constants.push(Constant::Nil);
        constant
    }
}

#[cfg(test)]
mod tests {
    use super::{Constant, ConstantManager};

    #[test]
    fn create_unique() {
        let mut constants = Vec::new();
        let mut constant_manager = ConstantManager { constants: &mut constants };
        assert_eq!(constant_manager.create_unique(9), 0);
    }

    #[test]
    fn create_unique_twice() {
        let mut constants = vec![Constant::String("__%lunify%__temp9\0".to_owned())];
        let mut constant_manager = ConstantManager { constants: &mut constants };
        assert_eq!(constant_manager.create_unique(9), 1);
    }

    #[test]
    fn constant_for_str() {
        let mut constants = Vec::new();
        let mut constant_manager = ConstantManager { constants: &mut constants };
        assert_eq!(constant_manager.constant_for_str("test"), 0);
    }

    #[test]
    fn constant_for_str_duplicate() {
        let mut constants = vec![Constant::String("test\0".to_owned())];
        let mut constant_manager = ConstantManager { constants: &mut constants };
        assert_eq!(constant_manager.constant_for_str("test"), 0);
    }

    #[test]
    fn constant_nil() {
        let mut constants = Vec::new();
        let mut constant_manager = ConstantManager { constants: &mut constants };
        assert_eq!(constant_manager.constant_nil(), 0);
    }

    #[test]
    fn constant_nil_duplicate() {
        let mut constants = vec![Constant::Nil];
        let mut constant_manager = ConstantManager { constants: &mut constants };
        assert_eq!(constant_manager.constant_nil(), 0);
    }
}
