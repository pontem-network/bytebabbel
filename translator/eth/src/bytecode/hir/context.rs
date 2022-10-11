use crate::bytecode::hir::stack::Stack;
use crate::bytecode::hir::vars::Vars;
use crate::bytecode::loc::Loc;
use crate::{BlockId, Flags, Function};
use primitive_types::U256;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Context<'a> {
    address: U256,
    fun: &'a Function,
    loop_input: HashMap<BlockId, (Stack, BlockId)>,
    loop_stack_size: usize,
    static_analysis: bool,
    code_size: u128,
    flags: Flags,
    pub loc: Loc<()>,
    pub stack: Stack,
    pub vars: Vars,
    jmp_id: usize,
}

impl<'a> Context<'a> {
    pub fn new(
        fun: &'a Function,
        contract_address: U256,
        code_size: u128,
        flags: Flags,
    ) -> Context {
        Context {
            address: contract_address,
            stack: Stack::default(),
            fun,
            loop_input: Default::default(),
            loop_stack_size: 0,
            static_analysis: true,
            code_size,
            flags,
            vars: Default::default(),
            loc: Loc::new(0, 0, ()),
            jmp_id: 0,
        }
    }

    pub fn next_jmp_id(&mut self) -> BlockId {
        self.jmp_id += 1;
        BlockId::from(self.jmp_id as u128)
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
        self.code_size
    }

    pub fn create_loop(&mut self, block_id: BlockId, break_br: BlockId) {
        self.loop_input
            .insert(block_id, (self.stack.clone(), break_br));
    }

    pub fn get_loop(&self, block_id: &BlockId) -> Option<&Stack> {
        self.loop_input.get(block_id).map(|(stack, _)| stack)
    }

    pub fn is_in_loop(&self) -> bool {
        self.loop_stack_size != 0
    }

    pub fn enter_loop(&mut self) {
        self.loop_stack_size += 1;
    }

    pub fn exit_loop(&mut self) {
        self.loop_stack_size -= 1;
    }

    pub fn flags(&self) -> &Flags {
        &self.flags
    }
}
