use crate::evm::bytecode::executor::stack::StackFrame;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Var(u8);

impl Var {
    pub fn index(self) -> u8 {
        self.0
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

    pub fn calc_stack(&mut self, frame: StackFrame) {
        self.flow.push(Execution::Calc(frame));
    }

    pub fn brunch(&mut self, true_br: FunctionFlow, false_br: FunctionFlow) {
        self.flow.push(Execution::Branch { true_br, false_br })
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
        true_br: FunctionFlow,
        false_br: FunctionFlow,
    },
    Abort(u8),
}
