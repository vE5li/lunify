use crate::number::Number;
use crate::{LunifyError, Settings};

#[derive(Debug, PartialEq)]
pub(crate) enum Constant {
    Nil,
    Boolean(bool),
    Number(Number),
    String(String),
}

pub(super) struct ConstantManager<'a> {
    pub(super) settings: &'a Settings<'a>,
    pub(super) constants: &'a mut Vec<Constant>,
}

impl<'a> ConstantManager<'a> {
    fn allocate_new_constant(&mut self) -> Result<u64, LunifyError> {
        let constant_index = self.constants.len() as u64;
        match constant_index <= self.settings.output.get_maximum_constant_index() {
            true => Ok(constant_index),
            false => Err(LunifyError::TooManyConstants(constant_index)),
        }
    }

    pub(super) fn create_unique(&mut self, program_counter: usize) -> Result<u64, LunifyError> {
        let unique_constant = self.allocate_new_constant()?;
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
        Ok(unique_constant)
    }

    pub(super) fn constant_for_str(&mut self, constant_str: &'static str) -> Result<u64, LunifyError> {
        let zero_terminated = format!("{constant_str}\0");

        // If the constant already exists we don't need to add it again.
        let matches = |constant: &_| matches!(constant, Constant::String(string) if string == zero_terminated.as_str());
        if let Some(index) = self.constants.iter().position(matches) {
            return Ok(index as u64);
        }

        let constant = self.allocate_new_constant()?;
        self.constants.push(Constant::String(zero_terminated));
        Ok(constant)
    }

    pub(super) fn constant_nil(&mut self) -> Result<u64, LunifyError> {
        // If the constant already exists we don't need to add it again.
        let matches = |constant: &_| matches!(constant, Constant::Nil);
        if let Some(index) = self.constants.iter().position(matches) {
            return Ok(index as u64);
        }

        let constant = self.allocate_new_constant()?;
        self.constants.push(Constant::Nil);
        Ok(constant)
    }
}

#[cfg(test)]
mod tests {
    use super::{Constant, ConstantManager};
    use crate::{LunifyError, Settings};

    #[test]
    fn constant_allocate_new() {
        let settings = Settings::default();
        let mut constants = vec![Constant::Nil];
        let mut constant_manager = ConstantManager {
            settings: &settings,
            constants: &mut constants,
        };

        let result = constant_manager.constant_for_str("willsucceed");
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn constant_allocate_new_too_many() {
        let mut settings = Settings::default();
        // With this BITRK will be 1, meaning we can only index a single constant.
        settings.output.layout.b.size = 1;

        let mut constants = vec![Constant::Nil];
        let mut constant_manager = ConstantManager {
            settings: &settings,
            constants: &mut constants,
        };

        let result = constant_manager.constant_for_str("willerror");
        assert_eq!(result, Err(LunifyError::TooManyConstants(1)));
    }

    #[test]
    fn create_unique() -> Result<(), LunifyError> {
        let settings = Settings::default();
        let mut constants = Vec::new();
        let mut constant_manager = ConstantManager {
            settings: &settings,
            constants: &mut constants,
        };

        assert_eq!(constant_manager.create_unique(9)?, 0);
        Ok(())
    }

    #[test]
    fn create_unique_twice() -> Result<(), LunifyError> {
        let settings = Settings::default();
        let mut constants = vec![Constant::String("__%lunify%__temp9_0\0".to_owned())];
        let mut constant_manager = ConstantManager {
            settings: &settings,
            constants: &mut constants,
        };

        assert_eq!(constant_manager.create_unique(9)?, 1);
        assert_eq!(&constants[1], &Constant::String("__%lunify%__temp9_1\0".to_owned()));
        Ok(())
    }

    #[test]
    fn constant_for_str() -> Result<(), LunifyError> {
        let settings = Settings::default();
        let mut constants = vec![Constant::String("constant".to_owned())];
        let mut constant_manager = ConstantManager {
            settings: &settings,
            constants: &mut constants,
        };

        assert_eq!(constant_manager.constant_for_str("test")?, 1);
        Ok(())
    }

    #[test]
    fn constant_for_str_duplicate() -> Result<(), LunifyError> {
        let settings = Settings::default();
        let mut constants = vec![Constant::String("test\0".to_owned()), Constant::String("constant".to_owned())];
        let mut constant_manager = ConstantManager {
            settings: &settings,
            constants: &mut constants,
        };

        assert_eq!(constant_manager.constant_for_str("test")?, 0);
        Ok(())
    }

    #[test]
    fn constant_nil() -> Result<(), LunifyError> {
        let settings = Settings::default();
        let mut constants = vec![Constant::String("constant".to_owned())];
        let mut constant_manager = ConstantManager {
            settings: &settings,
            constants: &mut constants,
        };

        assert_eq!(constant_manager.constant_nil()?, 1);
        Ok(())
    }

    #[test]
    fn constant_nil_duplicate() -> Result<(), LunifyError> {
        let settings = Settings::default();
        let mut constants = vec![Constant::Nil, Constant::String("constant".to_owned())];
        let mut constant_manager = ConstantManager {
            settings: &settings,
            constants: &mut constants,
        };

        assert_eq!(constant_manager.constant_nil()?, 0);
        Ok(())
    }
}
