use crate::bytecode::llir::stack::{Frame, StackFrame};
use crate::bytecode::types::U256;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Clone)]
pub struct Memory {
    mem: HashMap<StackFrame, StackFrame>,
}

impl Memory {
    pub fn store(&mut self, rf: StackFrame, val: StackFrame) {
        self.mem.insert(rf, val);
    }

    pub fn load(&mut self, rf: &StackFrame) -> StackFrame {
        // let val = self
        //     .mem
        //     .entry(rf.clone())
        //     .or_insert_with(|| StackFrame::new(Frame::Var(U256::from(0))));
        //val.clone()
        todo!()
    }

    pub fn clean(&mut self) {
        self.mem.clear();
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (rf, val) in &self.mem {
            writeln!(f, "{:?} = {:?}", rf, val)?;
        }
        Ok(())
    }
}
