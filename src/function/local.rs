pub(crate) struct LocalVariable {
    pub(crate) name: Vec<u8>,
    pub(crate) start_program_counter: i64,
    pub(crate) end_program_counter: i64,
}
