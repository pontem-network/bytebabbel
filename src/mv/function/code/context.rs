use crate::evm::bytecode::executor::execution::Var;
use crate::mv::function::code::stack::Stack;
use crate::mv::function::code::writer::{CodeWriter, FunctionCode};
use anyhow::{anyhow, Error};
use move_binary_format::file_format::{Bytecode, CodeOffset, LocalIndex, SignatureToken};
use std::collections::HashMap;

pub struct Context {
    code: CodeWriter,
    stack: Stack,
    local_mapping: HashMap<Var, LocalIndex>,
    input: Vec<SignatureToken>,
    output: Vec<SignatureToken>,
}

impl Context {
    pub fn new(input: Vec<SignatureToken>, output: Vec<SignatureToken>) -> Context {
        Context {
            code: CodeWriter::new(input.len()),
            stack: Default::default(),
            local_mapping: Default::default(),
            input,
            output,
        }
    }

    pub fn write_code(&mut self, code: Bytecode) {
        self.update_stack(&code);
        log::trace!("{}: {:?} => {}", self.code.pc(), code, self.stack);
        self.code.push(code);
    }

    pub fn update_jmp_pc(&mut self, offset: CodeOffset, pc: CodeOffset) {
        self.code.update_jmp_pc(offset, pc);
    }

    pub fn extend_code<I: IntoIterator<Item = Bytecode>>(&mut self, code: I) {
        for code in code {
            self.write_code(code);
        }
    }

    pub fn push_stack(&mut self, tp: SignatureToken) -> SignatureToken {
        self.stack.push(tp.clone());
        tp
    }

    pub fn set_global_var(&mut self, id: Var, tp: SignatureToken) {
        let idx = self.set_var(tp);
        self.local_mapping.insert(id, idx);
    }

    pub fn freeze(self) -> FunctionCode {
        self.code.freeze()
    }

    pub fn pc(&self) -> CodeOffset {
        self.code.pc()
    }

    pub fn borrow_local(&mut self, tp: SignatureToken) -> LocalIndex {
        self.code.borrow_local(tp)
    }

    pub fn release_local(&mut self, idx: LocalIndex) -> Option<SignatureToken> {
        self.code.release_local(idx)
    }

    pub fn set_var(&mut self, tp: SignatureToken) -> LocalIndex {
        let idx = self.code.borrow_local(tp);
        self.write_code(Bytecode::StLoc(idx));
        idx
    }

    pub fn move_local(&mut self, idx: LocalIndex) {
        self.code.release_local(idx);
        self.write_code(Bytecode::MoveLoc(idx));
    }

    pub fn st_var(&mut self, var: &Var) -> Result<(), Error> {
        let local = self
            .local_mapping
            .get(var)
            .ok_or_else(|| anyhow!("Unknown result variable:{}", var))?;
        self.move_local(*local);
        Ok(())
    }

    fn update_stack(&mut self, code: &Bytecode) {
        match code {
            Bytecode::Pop => {
                self.stack.pop();
            }
            Bytecode::Ret => {
                for _ in 0..self.output.len() {
                    self.stack.pop();
                }
            }
            Bytecode::Abort => {
                self.stack.pop();
            }
            Bytecode::Branch(_) | Bytecode::Nop | Bytecode::FreezeRef => {
                // no-op
            }
            Bytecode::BrTrue(_) => {
                self.stack.pop();
            }
            Bytecode::BrFalse(_) => {
                self.stack.pop();
            }
            Bytecode::LdU8(_) => {
                self.stack.push(SignatureToken::U8);
            }
            Bytecode::LdU64(_) => {
                self.stack.push(SignatureToken::U64);
            }
            Bytecode::LdU128(_) => {
                self.stack.push(SignatureToken::U128);
            }
            Bytecode::CastU8 => {
                self.stack.pop();
                self.stack.push(SignatureToken::U8);
            }
            Bytecode::CastU64 => {
                self.stack.pop();
                self.stack.push(SignatureToken::U64);
            }
            Bytecode::CastU128 => {
                self.stack.pop();
                self.stack.push(SignatureToken::U128);
            }
            Bytecode::LdConst(_) => {
                todo!();
            }
            Bytecode::LdTrue => {
                self.stack.push(SignatureToken::Bool);
            }
            Bytecode::LdFalse => {
                self.stack.push(SignatureToken::Bool);
            }
            Bytecode::CopyLoc(idx) | Bytecode::MoveLoc(idx) => {
                let local_tp = self
                    .input
                    .get(*idx as usize)
                    .cloned()
                    .or_else(|| self.code.local_type(*idx));

                if let Some(tp) = local_tp {
                    self.stack.push(tp);
                } else {
                    panic!("Unknown local variable index:{}", idx);
                }
            }
            Bytecode::StLoc(_) => {
                self.stack.pop();
            }
            Bytecode::Call(_) => {
                todo!()
            }
            Bytecode::CallGeneric(_) => {
                todo!()
            }
            Bytecode::Pack(_) => {
                todo!()
            }
            Bytecode::PackGeneric(_) => {
                todo!()
            }
            Bytecode::Unpack(_) => {
                todo!()
            }
            Bytecode::UnpackGeneric(_) => {
                todo!()
            }
            Bytecode::ReadRef => {
                let tp = self.stack.pop();
                if let SignatureToken::MutableReference(tp) = tp {
                    self.stack.push(*tp);
                } else if let SignatureToken::Reference(tp) = tp {
                    self.stack.push(*tp);
                } else {
                    panic!("Expected mutable reference or reference");
                }
            }
            Bytecode::WriteRef => {
                self.stack.pop2();
            }
            Bytecode::MutBorrowLoc(_) => {
                todo!()
            }
            Bytecode::ImmBorrowLoc(_) => {
                todo!()
            }
            Bytecode::MutBorrowField(_) => {
                todo!()
            }
            Bytecode::MutBorrowFieldGeneric(_) => {
                todo!()
            }
            Bytecode::ImmBorrowField(_) => {
                todo!()
            }
            Bytecode::ImmBorrowFieldGeneric(_) => {
                todo!()
            }
            Bytecode::MutBorrowGlobal(_) => {
                todo!()
            }
            Bytecode::MutBorrowGlobalGeneric(_) => {
                todo!()
            }
            Bytecode::ImmBorrowGlobal(_) => {
                todo!()
            }
            Bytecode::ImmBorrowGlobalGeneric(_) => {
                todo!()
            }
            Bytecode::Add
            | Bytecode::Sub
            | Bytecode::Mul
            | Bytecode::Mod
            | Bytecode::Div
            | Bytecode::BitOr
            | Bytecode::BitAnd
            | Bytecode::Xor
            | Bytecode::Shl
            | Bytecode::Shr => {
                let val = self.stack.pop2();
                self.stack.push(val[0].clone());
            }
            Bytecode::Or
            | Bytecode::And
            | Bytecode::Eq
            | Bytecode::Neq
            | Bytecode::Lt
            | Bytecode::Gt
            | Bytecode::Le
            | Bytecode::Ge => {
                self.stack.pop2();
                self.stack.push(SignatureToken::Bool);
            }
            Bytecode::Not => {
                self.stack.pop();
                self.stack.push(SignatureToken::Bool);
            }
            Bytecode::Exists(_) | Bytecode::ExistsGeneric(_) => {
                self.stack.pop();
                self.stack.push(SignatureToken::Bool);
            }
            Bytecode::MoveFrom(_) => {
                todo!()
            }
            Bytecode::MoveFromGeneric(_) => {
                todo!()
            }
            Bytecode::MoveTo(_) => {
                todo!()
            }
            Bytecode::MoveToGeneric(_) => {
                todo!()
            }
            Bytecode::VecPack(_, num) => {
                let tp = self.stack.pop();
                for _ in 0..(*num - 1) {
                    self.stack.pop();
                }
                self.stack.push(SignatureToken::Vector(Box::new(tp)));
            }
            Bytecode::VecLen(_) => {
                self.stack.pop();
                self.stack.push(SignatureToken::U64);
            }
            Bytecode::VecImmBorrow(_) => {
                let tp = self.stack.pop();
                self.stack.pop();
                if let SignatureToken::Vector(inner) = tp {
                    self.stack.push(SignatureToken::Reference(inner));
                } else {
                    panic!("Expected vector type, got:{:?}", tp);
                }
            }
            Bytecode::VecMutBorrow(_) => {
                let tp = self.stack.pop();
                self.stack.pop();
                if let SignatureToken::Vector(inner) = tp {
                    self.stack.push(SignatureToken::MutableReference(inner));
                } else {
                    panic!("Expected vector type, got:{:?}", tp);
                }
            }
            Bytecode::VecPushBack(_) => {
                self.stack.pop2();
            }
            Bytecode::VecPopBack(_) => {
                let tp = self.stack.pop();
                if let SignatureToken::Vector(inner) = tp {
                    self.stack.push(*inner);
                } else {
                    panic!("Expected vector type, got:{:?}", tp);
                }
            }
            Bytecode::VecUnpack(_, num) => {
                let tp = self.stack.pop();
                if let SignatureToken::Vector(inner) = tp {
                    let inner = *inner;
                    for _ in 0..*num {
                        self.stack.push(inner.clone());
                    }
                } else {
                    panic!("Expected vector type, got:{:?}", tp);
                }
            }
            Bytecode::VecSwap(_) => {
                self.stack.pop2();
                self.stack.pop();
            }
        };
    }

    pub fn store_stack(&mut self) -> StoredStack {
        let mut stack_store = Vec::new();
        let stack = self.stack.clone();
        log::trace!("Stored stack: {:?}", stack);
        for tp in stack.into_inner() {
            stack_store.push(self.set_var(tp));
        }
        StoredStack { stack: stack_store }
    }

    pub fn restore_stack(&mut self, stack: StoredStack) {
        for idx in stack.stack.into_iter().rev() {
            self.move_local(idx);
        }
        log::trace!("Restore stack: {:?}", self.stack);
    }
}

#[derive(Debug, Clone)]
pub struct StoredStack {
    stack: Vec<LocalIndex>,
}

