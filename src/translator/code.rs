use move_binary_format::file_format::Bytecode;

#[derive(Debug)]
pub struct CodeUnit {
    pub code: Vec<Bytecode>,
}

impl From<Vec<Bytecode>> for CodeUnit {
    fn from(bytecode: Vec<Bytecode>) -> Self {
        Self { code: bytecode }
    }
}

impl From<Bytecode> for CodeUnit {
    fn from(bytecode: Bytecode) -> Self {
        Self {
            code: vec![bytecode],
        }
    }
}
