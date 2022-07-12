use move_binary_format::file_format::{Bytecode, LocalIndex};

pub mod u256_math;

pub trait Math {
    fn cast_from_u128(&self, bytecode: &mut Vec<Bytecode>, local: Option<LocalIndex>);
    // fn add(&self, bytecode: &mut Vec<Bytecode>);
    // fn sub(&self, bytecode: &mut Vec<Bytecode>);
    // fn mul(&self, bytecode: &mut Vec<Bytecode>);
    // fn div(&self, bytecode: &mut Vec<Bytecode>);
    // fn eq(&self, bytecode: &mut Vec<Bytecode>);
    fn cast_to_u128(&self, bytecode: &mut Vec<Bytecode>);
}
