use crate::evm::bytecode::executor::stack::{StackFrame, Used, FRAME_SIZE};
use crate::evm::bytecode::executor::Executor;
use crate::evm::bytecode::instruction::Instruction;
use crate::evm::OpCode;

#[derive(Debug, Clone)]
pub struct Statement {
    pub in_items: Vec<StackFrame>,
    pub out_items: Vec<StackFrame>,
}

impl Statement {
    pub fn new(in_items: Vec<StackFrame>) -> Statement {
        Statement {
            in_items,
            out_items: vec![],
        }
    }

    pub fn perform(&mut self, exec: &mut Executor, inst: &Instruction) -> Vec<StackFrame> {
        let out = match &inst.1 {
            OpCode::Pop => vec![],
            OpCode::Addr => vec![StackFrame::self_address()],
            OpCode::CallDataSize => vec![StackFrame::from(exec.call_data_size())],
            OpCode::CallDataLoad => self.call_data_load(exec),
            OpCode::Lt => self.lt(),
            OpCode::EQ => self.eq(),
            OpCode::Shr => self.shr(),
            OpCode::MStore => self.mem_store(),
            OpCode::Stop
            | OpCode::CallDataCopy
            | OpCode::CodeCopy
            | OpCode::ExtCodeCopy
            | OpCode::MStore
            | OpCode::MStore8
            | OpCode::SStore
            | OpCode::Jump
            | OpCode::JumpIf
            | OpCode::JumpDest
            | OpCode::Log(..)
            | OpCode::Return
            | OpCode::Invalid(_)
            | OpCode::SelfDestruct
            | OpCode::ReturnDataCopy
            | OpCode::Revert => {
                self.mark_as_used();
                vec![]
            }
            OpCode::Push(val) => vec![StackFrame::from(val.as_slice())],
            OpCode::Dup(_) => {
                let mut out = self.in_items.clone();
                let new_item = out[out.len() - 1].clone();
                out.insert(0, new_item);
                out
            }
            OpCode::Swap(_) => {
                let mut out = self.in_items.clone();
                let last_index = out.len() - 1;
                out.swap(0, last_index);
                out
            }
            _ => vec![StackFrame::unknown()],
        };
        self.out_items = out.clone();
        out
    }

    fn mem_store(&mut self, exec: &mut Executor) -> Vec<StackFrame> {
       let mut mem = exec.mem.borrow_mut();
        mem.insert(self.in_items[0].clone(), self)
        vec![]
    }

    fn mark_as_used(&self) {
        for item in &self.in_items {
            item.mark_as_used();
        }
    }

    fn lt(&mut self) -> Vec<StackFrame> {
        let used = Used::default();
        let a = &mut self.in_items[0];
        a.set_used_flag(used.clone());
        let a = a.as_u256();

        let b = &mut self.in_items[1];
        b.set_used_flag(used.clone());
        let b = b.as_u256();

        if let Some(a) = a {
            if let Some(b) = b {
                let mut new = StackFrame::from(a < b);
                new.set_used_flag(used);
                return vec![new];
            }
        }

        let mut new = StackFrame::calc2(
            OpCode::Lt,
            self.in_items[0].clone(),
            self.in_items[1].clone(),
        );
        new.set_used_flag(used);
        vec![new]
    }

    fn eq(&mut self) -> Vec<StackFrame> {
        let used = Used::default();
        let a = &mut self.in_items[0];
        a.set_used_flag(used.clone());
        let a = a.as_u256();

        let b = &mut self.in_items[1];
        b.set_used_flag(used.clone());
        let b = b.as_u256();

        if let Some(a) = a {
            if let Some(b) = b {
                let mut new = StackFrame::from(a == b);
                new.set_used_flag(used);
                return vec![new];
            }
        }

        let mut new = StackFrame::calc2(
            OpCode::EQ,
            self.in_items[0].clone(),
            self.in_items[1].clone(),
        );
        new.set_used_flag(used);
        vec![new]
    }

    fn shr(&mut self) -> Vec<StackFrame> {
        let used = Used::default();
        let a = &mut self.in_items[0];
        a.set_used_flag(used.clone());
        let a = a.as_u256();

        let b = &mut self.in_items[1];
        b.set_used_flag(used.clone());
        let b = b.as_u256();

        if let Some(a) = a {
            if let Some(b) = b {
                let mut new = StackFrame::from(b >> a.as_u64() as usize);
                new.set_used_flag(used);
                return vec![new];
            }
        }

        let mut new = StackFrame::calc2(
            OpCode::EQ,
            self.in_items[0].clone(),
            self.in_items[1].clone(),
        );
        new.set_used_flag(used);
        vec![new]
    }

    fn call_data_load(&mut self, ctx: &Executor) -> Vec<StackFrame> {
        if let Some(offset) = self.in_items[0].as_u256() {
            let index = offset.as_u64() / FRAME_SIZE as u64;
            if index == 0 {
                let mut buf = [0u8; 32];
                buf[0..4].copy_from_slice(ctx.hash.as_ref().as_slice());
                let new_frame = StackFrame::from(buf.as_slice());
                self.in_items[0].set_used_flag(new_frame.get_used_flag());
                vec![new_frame]
            } else {
                let new_frame = StackFrame::param(index as u16);
                self.in_items[0].set_used_flag(new_frame.get_used_flag());
                vec![new_frame]
            }
        } else {
            let new_frame = StackFrame::unknown();
            self.in_items[0].set_used_flag(new_frame.get_used_flag());
            vec![new_frame]
        }
    }
}
