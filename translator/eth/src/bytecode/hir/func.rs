use crate::{Hir, Offset};

pub struct PrivateFunc {
    pub name: Offset,
    // pub params: Vec<Variable>,
    // pub ret: Vec<Variable>,
    pub body: Hir,
}
