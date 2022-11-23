use crate::bytecode::hir::stack::Stack;
use crate::bytecode::hir::vars::Vars;
use crate::bytecode::loc::Loc;
use crate::{Flags, Function, Offset};
use primitive_types::U256;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Context<'a, 'b> {
    address: U256,
    contract: &'b [u8],
    fun: &'a Function,
    loops: HashMap<Offset, (Offset, Offset)>,
    loop_stack_size: usize,
    static_analysis: bool,
    flags: Flags,
    pub loc: Loc<()>,
    pub stack: Stack,
    pub vars: Vars,
    jmp_id: usize,
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn new(
        fun: &'a Function,
        contract_address: U256,
        flags: Flags,
        contract: &'b [u8],
    ) -> Context<'a, 'b> {
        Context {
            address: contract_address,
            stack: Stack::default(),
            fun,
            loops: Default::default(),
            loop_stack_size: 0,
            static_analysis: true,
            flags,
            vars: Default::default(),
            loc: Loc::new(0u128, 0u128, ()),
            jmp_id: 0,
            contract,
        }
    }

    pub fn next_jmp_id(&mut self) -> Offset {
        self.jmp_id += 1;
        Offset::from(self.jmp_id as u128)
    }

    pub fn disable_static_analysis(&mut self) {
        self.static_analysis = false;
    }

    pub fn is_static_analysis_enable(&self) -> bool {
        self.static_analysis
    }

    pub fn fun(&self) -> &Function {
        self.fun
    }

    pub fn address(&self) -> U256 {
        self.address
    }

    pub fn code_size(&self) -> u128 {
        self.contract.len() as u128
    }

    pub fn contract_slice(&self, offset: u128, len: u128) -> &[u8] {
        &self.contract[offset as usize..(offset + len) as usize]
    }

    pub fn is_in_loop(&self) -> bool {
        self.loop_stack_size != 0
    }

    fn enter_loop(&mut self) {
        self.loop_stack_size += 1;
    }

    pub fn exit_loop(&mut self) {
        self.loop_stack_size -= 1;
    }

    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    pub fn has_loop(&self, lp: Offset) -> bool {
        self.loops.contains_key(&lp)
    }

    pub fn create_loop(&mut self, lp: Offset, from: Offset) -> (Offset, bool) {
        self.enter_loop();
        if let Some((from_lp, idx)) = self.loops.get(&lp).cloned() {
            if from == lp {
                let id = (from, self.next_jmp_id());
                self.loops.insert(lp, id);
                (id.0 + id.1, true)
            } else {
                (from_lp + idx, false)
            }
        } else {
            let id = (from, self.next_jmp_id());
            self.loops.insert(lp, id);
            (id.0 + id.1, true)
        }
    }
}
