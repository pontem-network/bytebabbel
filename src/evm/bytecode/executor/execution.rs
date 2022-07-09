use crate::evm::bytecode::executor::stack::StackFrame;

#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Var(u8);

impl Var {
    pub fn index(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Default)]
pub struct FunctionFlow {
    var_seq: Var,
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

    pub fn set_result(&mut self, var: Var) {
        self.result.push(var);
    }

    pub fn execution_tree(&self) -> &[Execution] {
        &self.flow
    }

    pub fn result(&self) -> &[Var] {
        &self.result
    }
}

#[derive(Debug)]
pub enum Execution {
    SetVar(Var, StackFrame),
}
