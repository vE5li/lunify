use crate::function::Constant;

pub(super) struct ConstantManager<'a> {
    pub(super) constants: &'a mut Vec<Constant>,
}

impl<'a> ConstantManager<'a> {
    pub(super) fn create_unique(&mut self, program_counter: usize) -> u64 {
        let unique_constant = self.constants.len() as u64;
        let constant_name = format!("__%lunify%__temp{}", program_counter);
        // TODO: make sure this is actually unique.
        self.constants.push(Constant::String(constant_name));
        unique_constant
    }

    pub(super) fn constant_for_str(&mut self, constant_str: &'static str) -> u64 {
        // If the constant already exists we don't need to add it again.
        let matches = |constant: &_| matches!(constant, Constant::String(string) if string == constant_str);
        if let Some(index) = self.constants.iter().position(matches) {
            return index as u64;
        }

        let constant = self.constants.len() as u64;
        self.constants.push(Constant::String(constant_str.to_owned()));
        constant
    }
}
