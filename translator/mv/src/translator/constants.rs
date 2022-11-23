use move_binary_format::file_format::{Constant, ConstantPoolIndex, SignatureToken};

pub struct ConstantPool {
    constants: Vec<Constant>,
}

impl ConstantPool {
    pub fn new(constants: Vec<Constant>) -> ConstantPool {
        ConstantPool { constants }
    }

    pub fn make_vec_constant(&mut self, constant: &[u8]) -> ConstantPoolIndex {
        let constant = bcs::to_bytes(constant).expect("Failed to serialize constant");
        let constant = Constant {
            type_: SignatureToken::Vector(Box::new(SignatureToken::U8)),
            data: constant,
        };

        let idx = self
            .constants
            .iter()
            .enumerate()
            .find(|(_, c)| c == &&constant)
            .map(|(i, _)| i);

        if let Some(idx) = idx {
            ConstantPoolIndex(idx as u16)
        } else {
            self.constants.push(constant);
            ConstantPoolIndex((self.constants.len() - 1) as u16)
        }
    }

    pub fn freeze(self) -> Vec<Constant> {
        self.constants
    }
}
