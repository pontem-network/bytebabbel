use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::stack::Stack;
use crate::bytecode::hir2::vars::Vars;
use crate::bytecode::types::Env;
use primitive_types::U256;
use std::rc::Rc;

#[derive(Debug)]
pub struct Context {
    address: U256,
    stack: Stack,
    env: Rc<Env>,
    // loop_input: HashMap<BlockId, (Stack, BlockId)>,
    loop_stack_size: usize,
    static_analysis: bool,
    code_size: u128,
    vars: Vars,
}

impl Context {
    pub fn new(env: Env, contract_address: U256, code_size: u128) -> Context {
        Context {
            address: contract_address,
            stack: Stack::default(),
            env: Rc::new(env),
            // loop_input: Default::default(),
            loop_stack_size: 0,
            static_analysis: true,
            code_size,
            vars: Default::default(),
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

    pub fn pop_stack(&mut self, pops: usize) -> Vec<Rc<Expr>> {
        self.stack.pop(pops)
    }

    pub fn push_stack(&mut self, to_push: Vec<Rc<Expr>>) {
        self.stack.push(to_push)
    }

    pub fn env(&self) -> &Env {
        self.env.as_ref()
    }

    pub fn address(&self) -> U256 {
        self.address
    }

    pub fn code_size(&self) -> u128 {
        self.code_size
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
            env: self.env.clone(),
            loop_stack_size: self.loop_stack_size,
            static_analysis: self.static_analysis,
            code_size: self.code_size,
            vars: self.vars.clone(),
        }
    }
}
