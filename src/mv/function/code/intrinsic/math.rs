use crate::mv::function::code::writer::CodeWriter;

pub mod u256_math;

pub trait Cast {
    fn cast_from_u128(&self, bytecode: &mut CodeWriter);
    fn cast_to_u128(&self, bytecode: &mut CodeWriter);
}
