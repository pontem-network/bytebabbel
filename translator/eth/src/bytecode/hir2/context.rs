use crate::bytecode::hir2::const_pool::ConstPool;
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::stack::Stack;
use crate::bytecode::hir2::vars::{VarId, Vars};
use crate::bytecode::tracing::exec::StackItem;
use crate::{Flags, Function};
use primitive_types::U256;
use std::rc::Rc;

#[derive(Debug)]
pub struct Context<'a> {
    address: U256,
    stack: Stack,
    vars: Vars,
    const_pool: ConstPool,
    fun: &'a Function,
    loop_stack_size: usize,
    static_analysis: bool,
    code_size: u128,
    flags: Flags,
}

impl<'a> Context<'a> {
    pub fn new(
        fun: &'a Function,
        contract_address: U256,
        code_size: u128,
        flags: Flags,
    ) -> Context<'a> {
        Context {
            address: contract_address,
            stack: Stack::default(),
            loop_stack_size: 0,
            static_analysis: true,
            code_size,
            flags,
            vars: Default::default(),
            const_pool: Default::default(),
            fun,
        }
    }

    pub fn disable_static_analysis(&mut self) {
        self.static_analysis = false;
    }

    pub fn is_static_analysis_enable(&self) -> bool {
        self.static_analysis
    }

    pub fn vars(&self) -> &Vars {
        &self.vars
    }

    pub fn push_var(&mut self, expr: Rc<Expr>, stack_item: StackItem) -> VarId {
        self.vars.insert(expr, stack_item)
    }

    pub fn pop_stack(&mut self, pops: usize) -> Vec<Rc<Expr>> {
        self.stack.pop(pops)
    }

    pub fn push_stack(&mut self, to_push: Vec<Rc<Expr>>) {
        self.stack.push(to_push)
    }

    pub fn address(&self) -> U256 {
        self.address
    }

    pub fn code_size(&self) -> u128 {
        self.code_size
    }

    pub fn flags(&self) -> &Flags {
        &self.flags
    }

    pub fn fun(&self) -> &Function {
        self.fun
    }

    pub fn const_pool(&mut self) -> &mut ConstPool {
        &mut self.const_pool
    }

    // pub fn create_loop(&mut self, block_id: BlockId, break_br: BlockId) {
    //     self.loop_input
    //         .insert(block_id, (self.stack.clone(), break_br));
    // }
    //
    // pub fn get_loop(&self, block_id: &BlockId) -> Option<&Stack> {
    //     self.loop_input.get(block_id).map(|(stack, _)| stack)
    // }
    //
    // pub fn map_stack(&self, origin: &Stack) -> Vec<MapStackItem> {
    //     let mut mapping = Vec::with_capacity(origin.stack.len());
    //
    //     mapping.push(MapStackItem {
    //         origin: origin.stack[origin.stack.len() - 1],
    //         new: self.stack.stack[self.stack.stack.len() - 1],
    //     });
    //     mapping.push(MapStackItem {
    //         origin: origin.stack[origin.stack.len() - 2],
    //         new: self.stack.stack[self.stack.stack.len() - 2],
    //     });
    //     mapping
    // }
    //
    pub fn is_in_loop(&self) -> bool {
        self.loop_stack_size != 0
    }

    pub fn enter_loop(&mut self) {
        self.loop_stack_size += 1;
    }

    pub fn exit_loop(&mut self) {
        self.loop_stack_size -= 1;
    }

    pub fn inherit(&self) -> Context {
        Context {
            address: self.address,
            stack: self.stack.clone(),
            vars: self.vars.clone(),
            const_pool: self.const_pool.clone(),
            fun: self.fun,
            loop_stack_size: self.loop_stack_size,
            static_analysis: self.static_analysis,
            code_size: self.code_size,
            flags: self.flags,
        }
    }
}
