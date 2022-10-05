use crate::bytecode::hir2::ir::{VarId, _Expr};
use crate::bytecode::loc::Loc;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct Vars {
    vars: HashMap<VarId, Loc<_Expr>>,
    var_seq: Rc<Cell<u32>>,
}

impl Vars {
    pub fn gen_tmp(&self) -> VarId {
        let idx = self.var_seq.get();
        let var = VarId::new_tmp(idx);
        self.var_seq.set(idx + 1);
        var
    }

    pub fn get(&self, var: &VarId) -> Option<&Loc<_Expr>> {
        self.vars.get(var)
    }

    pub fn set(&mut self, var: VarId, expr: Loc<_Expr>) {
        if let _Expr::Var(id) = expr.as_ref() {
            let expr = self.vars.get(id).expect("var not found").clone();
            self.vars.insert(var, expr);
        } else {
            self.vars.insert(var, expr);
        }
    }
}

impl Debug for Vars {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Vars {{")?;
        for (var, expr) in &self.vars {
            writeln!(f, " {} => {:?},", var, expr.as_ref())?;
        }
        write!(f, " }}")
    }
}
