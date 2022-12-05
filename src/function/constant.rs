use crate::number::Number;

#[derive(Debug, PartialEq)]
pub(crate) enum Constant {
    Nil,
    Boolean(bool),
    Number(Number),
    String(String),
}

pub(super) struct ConstantManager<'a> {
    pub(super) constants: &'a mut Vec<Constant>,
}

impl<'a> ConstantManager<'a> {
    pub(super) fn create_unique(&mut self, program_counter: usize) -> u64 {
        let constant_index = self.constants.len() as u64;
        let mut index = 0;

        let constant = loop {
            let constant_name = format!("__%lunify%__temp{program_counter}_{index}\0");
            let constant = Constant::String(constant_name);

            if !self.constants.contains(&constant) {
                break constant;
            }

            index += 1;
        };

        self.constants.push(constant);
        constant_index
    }

    pub(super) fn constant_for_str(&mut self, constant_str: &'static str) -> u64 {
        let zero_terminated = format!("{constant_str}\0");

        // If the constant already exists we don't need to add it again.
        let matches = |constant: &_| matches!(constant, Constant::String(string) if string == zero_terminated.as_str());
        if let Some(index) = self.constants.iter().position(matches) {
            return index as u64;
        }

        let constant_index = self.constants.len() as u64;
        self.constants.push(Constant::String(zero_terminated));
        constant_index
    }

    pub(super) fn constant_nil(&mut self) -> u64 {
        // If the constant already exists we don't need to add it again.
        let matches = |constant: &_| matches!(constant, Constant::Nil);
        if let Some(index) = self.constants.iter().position(matches) {
            return index as u64;
        }

        let constant_index = self.constants.len() as u64;
        self.constants.push(Constant::Nil);
        constant_index
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
        assert_eq!(&constants[0], &Constant::String("__%lunify%__temp9_0\0".to_owned()));
    }

    #[test]
    fn create_unique_twice() {
        let mut constants = vec![Constant::String("__%lunify%__temp9_0\0".to_owned())];
        let mut constant_manager = ConstantManager { constants: &mut constants };

        assert_eq!(constant_manager.create_unique(9), 1);
        assert_eq!(&constants[1], &Constant::String("__%lunify%__temp9_1\0".to_owned()));
    }

    #[test]
    fn constant_for_str() {
        let mut constants = vec![Constant::String("constant".to_owned())];
        let mut constant_manager = ConstantManager { constants: &mut constants };

        assert_eq!(constant_manager.constant_for_str("test"), 1);
        assert_eq!(&constants[1], &Constant::String("test\0".to_owned()));
    }

    #[test]
    fn constant_for_str_duplicate() {
        let mut constants = vec![Constant::String("test\0".to_owned()), Constant::String("constant".to_owned())];
        let mut constant_manager = ConstantManager { constants: &mut constants };

        assert_eq!(constant_manager.constant_for_str("test"), 0);
    }

    #[test]
    fn constant_nil() {
        let mut constants = vec![Constant::String("constant".to_owned())];
        let mut constant_manager = ConstantManager { constants: &mut constants };

        assert_eq!(constant_manager.constant_nil(), 1);
        assert_eq!(&constants[1], &Constant::Nil);
    }

    #[test]
    fn constant_nil_duplicate() {
        let mut constants = vec![Constant::Nil, Constant::String("constant".to_owned())];
        let mut constant_manager = ConstantManager { constants: &mut constants };

        assert_eq!(constant_manager.constant_nil(), 0);
    }
}
