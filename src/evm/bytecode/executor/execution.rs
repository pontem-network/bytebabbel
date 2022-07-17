use crate::evm::bytecode::executor::stack::StackFrame;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Var(u16);

impl Var {
    pub fn index(self) -> u8 {
        self.0 as u8
    }

    pub fn unit() -> Var {
        Var(u16::MAX)
    }

    pub fn is_unit(&self) -> bool {
        self.0 == u16::MAX
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

#[derive(Default, Debug)]
pub struct FunctionFlow {
    pub var_seq: Var,
    flow: Vec<Execution>,
    result: Vec<Var>,
}

impl FunctionFlow {
    pub fn calc_var(&mut self, frame: StackFrame) -> Var {
        let idx = self.var_seq;
        self.var_seq.0 += 1;
        self.flow.push(Execution::SetVar(idx, frame));
        idx
    }

    pub fn brunch(&mut self, cnd: StackFrame, true_br: FunctionFlow, false_br: FunctionFlow) {
        self.flow.push(Execution::Branch {
            cnd,
            true_br,
            false_br,
        })
    }

    pub fn set_result(&mut self, var: Var) {
        self.result.push(var);
    }

    pub fn execution_tree(&self) -> &[Execution] {
        &self.flow
    }

    pub fn result(&self) -> &[Var] {
        &self.result
    }

    pub fn abort(&mut self, code: u8) {
        self.flow.push(Execution::Abort(code));
    }
}

#[derive(Debug)]
pub enum Execution {
    SetVar(Var, StackFrame),
    Calc(StackFrame),
    Branch {
        cnd: StackFrame,
        true_br: FunctionFlow,
        false_br: FunctionFlow,
    },
    Abort(u8),
}
