use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::hir::stack::Stack;
use crate::bytecode::types::Env;
use crate::BlockId;
use primitive_types::U256;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Context {
    address: U256,
    stack: Stack,
    env: Rc<Env>,
    loop_input: HashMap<BlockId, (Stack, BlockId)>,
    loop_stack_size: usize,
    code_size: u128,
}

impl Context {
    pub fn new(env: Env, contract_address: U256, code_size: u128) -> Context {
        Context {
            address: contract_address,
            stack: Stack::default(),
            env: Rc::new(env),
            loop_input: Default::default(),
            loop_stack_size: 0,
            code_size,
        }
    }

    pub fn pop_stack(&mut self, pops: usize) -> Vec<VarId> {
        self.stack.pop(pops)
    }

    pub fn push_stack(&mut self, to_push: Vec<VarId>) {
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

    pub fn create_loop(&mut self, block_id: BlockId, break_br: BlockId) {
        self.loop_input
            .insert(block_id, (self.stack.clone(), break_br));
    }

    pub fn get_loop(&self, block_id: &BlockId) -> Option<&Stack> {
        self.loop_input.get(block_id).map(|(stack, _)| stack)
    }

    pub fn map_stack(&self, origin: &Stack) -> Vec<MapStackItem> {
        let mut mapping = Vec::with_capacity(origin.stack.len());

        mapping.push(MapStackItem {
            origin: origin.stack[origin.stack.len() - 1],
            new: self.stack.stack[self.stack.stack.len() - 1],
        });
        mapping.push(MapStackItem {
            origin: origin.stack[origin.stack.len() - 2],
            new: self.stack.stack[self.stack.stack.len() - 2],
        });
        // let mut last_origin_index = origin.stack.len() - 1;
        // let mut last_new_index = self.stack.stack.len() - 1;
        // loop {
        //     let new = self.stack.stack.get(last_new_index);
        //     let origin = origin.stack.get(last_origin_index);
        //
        //     if let (Some(new), Some(origin)) = (new, origin) {
        //         if new != origin {
        //             mapping.push(MapStackItem {
        //                 origin: *origin,
        //                 new: *new,
        //             });
        //         }
        //     } else {
        //         break;
        //     }
        //     if last_origin_index == 0 || last_new_index == 0 {
        //         break;
        //     }
        //     last_origin_index -= 1;
        //     last_new_index -= 1;
        // }

        mapping
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
}

#[derive(Debug, Clone)]
pub struct MapStackItem {
    pub origin: VarId,
    pub new: VarId,
}
