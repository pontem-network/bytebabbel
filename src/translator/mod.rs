use crate::Instruction;
pub use code::CodeUnit;

mod code;

pub trait Translate {
    fn translate(op: Instruction) -> CodeUnit;
}
