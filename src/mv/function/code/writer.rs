use move_binary_format::file_format::{
    Bytecode, CodeOffset, LocalIndex, SignatureToken, StructFieldInformation,
};
use std::collections::{HashMap, VecDeque};

pub struct CodeWriter {
    code: Vec<Bytecode>,
    locals: HashMap<SignatureToken, Locals>,
    local_seq: LocalIndex,
    params_count: LocalIndex,
    trace: bool,
    stack: Vec<SignatureToken>,
}

impl CodeWriter {
    pub fn new(params_count: usize, trace: bool) -> CodeWriter {
        CodeWriter {
            code: vec![],
            locals: Default::default(),
            local_seq: params_count as LocalIndex,
            params_count: params_count as LocalIndex,
            trace,
            stack: Default::default(),
        }
    }

    pub fn borrow_local(&mut self, tp: SignatureToken) -> LocalIndex {
        let locals = self.locals.entry(tp).or_default();
        if let Some(idx) = locals.borrow() {
            idx
        } else {
            let idx = self.local_seq;
            self.local_seq += 1;
            locals.new_borrowed(idx);
            idx
        }
    }

    pub fn release_local(&mut self, idx: LocalIndex) -> Option<SignatureToken> {
        for (tp, locals) in self.locals.iter_mut() {
            if locals.release(idx) {
                return Some(tp.clone());
            }
        }
        None
    }

    pub fn set_var(&mut self, tp: SignatureToken) -> LocalIndex {
        let idx = self.borrow_local(tp);
        self.stack.pop();
        self.push(Bytecode::StLoc(idx));
        idx
    }

    pub fn move_local(&mut self, idx: LocalIndex) {
        if let Some(tp) = self.release_local(idx) {
            self.stack.push(tp);
            self.push(Bytecode::MoveLoc(idx));
        }
    }

    pub fn push(&mut self, code: Bytecode) {
        code.self.code.push(code);
    }

    pub fn set_op(&mut self, idx: CodeOffset, op_code: Bytecode) {
        self.code[idx as usize] = op_code;
    }

    pub fn extend<I: IntoIterator<Item = Bytecode>>(&mut self, code: I) {
        for bytecode in code {
            self.push(bytecode);
        }
    }

    pub fn pc(&self) -> CodeOffset {
        self.code.len() as CodeOffset
    }

    pub fn freeze(self) -> FunctionCode {
        let locals = (self.params_count..self.local_seq)
            .map(|id| {
                self.locals
                    .iter()
                    .find(|(_, locals)| locals.contains(id))
                    .map(|(tkn, _)| tkn.clone())
                    .unwrap()
            })
            .collect::<Vec<_>>();

        if self.trace {
            for (local, tkn) in locals.iter().enumerate() {
                println!("loc_{:?} = {:?}", local, tkn);
            }

            for (idx, code) in self.code.iter().enumerate() {
                println!("{idx}: {code:?}");
            }
        }

        FunctionCode {
            code: self.code,
            locals,
        }
    }
}

#[derive(Default)]
pub struct Locals {
    free: Vec<LocalIndex>,
    borrowed: Vec<LocalIndex>,
}

impl Locals {
    pub fn borrow(&mut self) -> Option<LocalIndex> {
        if let Some(free) = self.free.pop() {
            self.borrowed.push(free);
            Some(free)
        } else {
            None
        }
    }

    pub fn new_borrowed(&mut self, id: LocalIndex) {
        self.borrowed.push(id);
    }

    pub fn release(&mut self, id: LocalIndex) -> bool {
        let borrowed_idx = self
            .borrowed
            .iter()
            .enumerate()
            .find(|(_, b)| **b == id)
            .map(|(i, _)| i);

        if let Some(borrowed_idx) = borrowed_idx {
            self.borrowed.remove(borrowed_idx);
            self.free.push(id);
            return true;
        } else {
            return false;
        }
    }

    pub fn contains(&self, idx: LocalIndex) -> bool {
        self.free.contains(&idx) || self.borrowed.contains(&idx)
    }
}

pub struct FunctionCode {
    pub code: Vec<Bytecode>,
    pub locals: Vec<SignatureToken>,
}

/// The effect of an instruction is a tuple where the first element
/// is the number of pops it does, and the second element is the number
/// of pushes it does
fn instruction_effect(&self, instruction: &Bytecode) -> (u64, u64) {
    match instruction {
        Bytecode::Pop
        | Bytecode::BrTrue(_)
        | Bytecode::BrFalse(_)
        | Bytecode::StLoc(_)
        | Bytecode::Abort => (1, 0),

        Bytecode::LdU8(_)
        | Bytecode::LdU64(_)
        | Bytecode::LdU128(_)
        | Bytecode::LdTrue
        | Bytecode::LdFalse
        | Bytecode::LdConst(_)
        | Bytecode::CopyLoc(_)
        | Bytecode::MoveLoc(_)
        | Bytecode::MutBorrowLoc(_)
        | Bytecode::ImmBorrowLoc(_) => (0, 1),

        Bytecode::Not
        | Bytecode::FreezeRef
        | Bytecode::ReadRef
        | Bytecode::Exists(_)
        | Bytecode::ExistsGeneric(_)
        | Bytecode::MutBorrowGlobal(_)
        | Bytecode::MutBorrowGlobalGeneric(_)
        | Bytecode::ImmBorrowGlobal(_)
        | Bytecode::ImmBorrowGlobalGeneric(_)
        | Bytecode::MutBorrowField(_)
        | Bytecode::MutBorrowFieldGeneric(_)
        | Bytecode::ImmBorrowField(_)
        | Bytecode::ImmBorrowFieldGeneric(_)
        | Bytecode::MoveFrom(_)
        | Bytecode::MoveFromGeneric(_)
        | Bytecode::CastU8
        | Bytecode::CastU64
        | Bytecode::CastU128
        | Bytecode::VecLen(_)
        | Bytecode::VecPopBack(_) => (1, 1),

        Bytecode::Add
        | Bytecode::Sub
        | Bytecode::Mul
        | Bytecode::Mod
        | Bytecode::Div
        | Bytecode::BitOr
        | Bytecode::BitAnd
        | Bytecode::Xor
        | Bytecode::Shl
        | Bytecode::Shr
        | Bytecode::Or
        | Bytecode::And
        | Bytecode::Eq
        | Bytecode::Neq
        | Bytecode::Lt
        | Bytecode::Gt
        | Bytecode::Le
        | Bytecode::Ge => (2, 1),

        Bytecode::VecPack(_, num) => (*num, 1),
        Bytecode::VecUnpack(_, num) => (1, *num),

        Bytecode::VecImmBorrow(_) | Bytecode::VecMutBorrow(_) => (2, 1),

        Bytecode::MoveTo(_)
        | Bytecode::MoveToGeneric(_)
        | Bytecode::WriteRef
        | Bytecode::VecPushBack(_) => (2, 0),

        Bytecode::VecSwap(_) => (3, 0),

        Bytecode::Branch(_) | Bytecode::Nop => (0, 0),

        Bytecode::Ret => {
            let return_count = self.return_.len();
            (return_count as u64, 0)
        }

        Bytecode::Call(idx) => {
            let function_handle = self.resolver.function_handle_at(*idx);
            let arg_count = self.resolver.signature_at(function_handle.parameters).len() as u64;
            let return_count = self.resolver.signature_at(function_handle.return_).len() as u64;
            (arg_count, return_count)
        }
        Bytecode::CallGeneric(idx) => {
            let func_inst = self.resolver.function_instantiation_at(*idx);
            let function_handle = self.resolver.function_handle_at(func_inst.handle);
            let arg_count = self.resolver.signature_at(function_handle.parameters).len() as u64;
            let return_count = self.resolver.signature_at(function_handle.return_).len() as u64;
            (arg_count, return_count)
        }

        Bytecode::Pack(idx) => {
            let struct_definition = self.resolver.struct_def_at(*idx)?;
            let field_count = match &struct_definition.field_information {
                StructFieldInformation::Native => 0,
                StructFieldInformation::Declared(fields) => fields.len(),
            };
            (field_count as u64, 1)
        }
        Bytecode::PackGeneric(idx) => {
            let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
            let struct_definition = self.resolver.struct_def_at(struct_inst.def)?;
            let field_count = match &struct_definition.field_information {
                StructFieldInformation::Native => 0,
                StructFieldInformation::Declared(fields) => fields.len(),
            };
            (field_count as u64, 1)
        }

        Bytecode::Unpack(idx) => {
            let struct_definition = self.resolver.struct_def_at(*idx)?;
            let field_count = match &struct_definition.field_information {
                StructFieldInformation::Native => 0,
                StructFieldInformation::Declared(fields) => fields.len(),
            };
            (1, field_count as u64)
        }
        Bytecode::UnpackGeneric(idx) => {
            let struct_inst = self.resolver.struct_instantiation_at(*idx)?;
            let struct_definition = self.resolver.struct_def_at(struct_inst.def)?;
            let field_count = match &struct_definition.field_information {
                StructFieldInformation::Native => 0,
                StructFieldInformation::Declared(fields) => fields.len(),
            };
            (1, field_count as u64)
        }
    }
}
