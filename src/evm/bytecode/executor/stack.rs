use crate::evm::bytecode::executor::block::BlockId;
use crate::evm::OpCode;
use bigint::U256;
use std::cell::Cell;
use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub const FRAME_SIZE: usize = 32;

#[derive(Default, Clone)]
pub struct ExecutionStack {
    pub negative_stack: VecDeque<StackFrame>,
    pub stack: Vec<StackFrame>,
}

impl ExecutionStack {
    pub fn pop(&mut self, count: usize) -> Vec<StackFrame> {
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(item) = self.stack.pop() {
                res.push(item);
            } else {
                let item = StackFrame::negative();
                self.negative_stack.push_front(item.clone());
                res.push(item);
            }
        }
        res
    }

    pub fn push(&mut self, to_push: Vec<StackFrame>) {
        self.stack.extend(to_push.into_iter().rev());
    }
}

#[derive(Default, Clone)]
pub struct Used(Rc<Cell<bool>>);

impl Used {
    pub fn mark_as_used(&self) {
        self.0.set(true);
    }
}

impl Debug for Used {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.get() {
            write!(f, "used")
        } else {
            write!(f, "unused")
        }
    }
}

impl Display for Used {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
pub struct StackFrame {
    pub cell: Rc<MemCell>,
    used: Used,
}

pub enum MemCell {
    Val(U256),
    Param(u16),
    NegativeStack,
    Unknown,
    SelfAddress,
    Mem(Box<StackFrame>, Box<StackFrame>),
    Calc2(OpCode, StackFrame, StackFrame),
}

impl StackFrame {
    pub fn calc2(op: OpCode, a: StackFrame, b: StackFrame) -> StackFrame {
        StackFrame {
            cell: Rc::new(MemCell::Calc2(op, a, b)),
            used: Default::default(),
        }
    }

    pub fn negative() -> StackFrame {
        StackFrame {
            cell: Rc::new(MemCell::NegativeStack),
            used: Default::default(),
        }
    }

    pub fn param(val: u16) -> StackFrame {
        StackFrame {
            cell: Rc::new(MemCell::Param(val)),
            used: Default::default(),
        }
    }

    pub fn unknown() -> StackFrame {
        StackFrame {
            cell: Rc::new(MemCell::Unknown),
            used: Default::default(),
        }
    }

    pub fn self_address() -> StackFrame {
        StackFrame {
            cell: Rc::new(MemCell::SelfAddress),
            used: Default::default(),
        }
    }

    pub fn mark_as_used(&self) {
        self.used.mark_as_used();
    }

    pub fn is_used(&self) -> bool {
        self.used.0.get()
    }

    pub fn get_used_flag(&self) -> Used {
        self.used.clone()
    }

    pub fn set_used_flag(&mut self, used: Used) {
        self.used = used;
    }

    pub fn as_usize(&self) -> Option<usize> {
        if let MemCell::Val(val) = self.cell.as_ref() {
            Some(val.as_u64() as usize)
        } else {
            None
        }
    }

    pub fn as_u256(&self) -> Option<U256> {
        if let MemCell::Val(val) = self.cell.as_ref() {
            Some(val.clone())
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let MemCell::Val(val) = self.cell.as_ref() {
            Some(val.as_u64() == 1)
        } else {
            None
        }
    }

    pub fn as_block_id(&self) -> Option<BlockId> {
        self.as_usize().map(|i| i.into())
    }
}

impl Debug for StackFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.cell.as_ref() {
            MemCell::Val(val) => write!(f, "({}-{})", val.to_hex(), self.used),
            MemCell::Param(idx) => write!(f, "(param: {idx} - {})", self.used),
            MemCell::NegativeStack => write!(f, "(negative stack - {})", self.used),
            MemCell::Unknown => write!(f, "(unknown - {})", self.used),
            MemCell::SelfAddress => write!(f, "(addr - {})", self.used),
            MemCell::Calc2(op, a, b) => write!(f, "(({op:?}({a}, {b}))-{})", self.used),
            MemCell::Mem(rf, mem) => write!(f, "(mem {:?} => {:?} - {})", rf, mem, self.used),
        }
    }
}

impl Display for StackFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<&[u8]> for StackFrame {
    fn from(buf: &[u8]) -> Self {
        let mut val = [0u8; 32];
        val[(32 - buf.len())..32].copy_from_slice(buf);
        StackFrame {
            cell: Rc::new(MemCell::Val(U256::from(val))),
            used: Default::default(),
        }
    }
}

impl From<usize> for StackFrame {
    fn from(buf: usize) -> Self {
        StackFrame {
            cell: Rc::new(MemCell::Val(U256::from(buf))),
            used: Default::default(),
        }
    }
}

impl From<U256> for StackFrame {
    fn from(val: U256) -> Self {
        StackFrame {
            cell: Rc::new(MemCell::Val(val)),
            used: Default::default(),
        }
    }
}

impl From<bool> for StackFrame {
    fn from(val: bool) -> Self {
        let val = if val { 1 } else { 0 };
        StackFrame {
            cell: Rc::new(MemCell::Val(U256::from(val))),
            used: Default::default(),
        }
    }
}
