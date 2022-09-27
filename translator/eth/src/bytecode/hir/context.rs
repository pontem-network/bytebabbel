use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::hir::stack::Stack;
use crate::bytecode::tracing::exec::StackItem;
use crate::{BlockId, Flags, Function};
use primitive_types::U256;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Context<'a> {
    address: U256,
    stack: Stack,
    fun: &'a Function,
    loop_input: HashMap<BlockId, (Stack, BlockId)>,
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
        }
    }

    pub fn disable_static_analysis(&mut self) {
        self.static_analysis = false;
    }

    pub fn is_static_analysis_enable(&self) -> bool {
        self.static_analysis
    }

    pub fn pop_stack(&mut self, pops: usize) -> Vec<VarId> {
        self.stack.pop(pops)
    }

    pub fn push_stack(&mut self, to_push: Vec<VarId>) {
        self.stack.push(to_push)
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

    pub fn map_stack(&self, origin: &Stack, input: &HashSet<StackItem>) -> Vec<MapStackItem> {
        // let mut mapping = Vec::with_capacity(input.len());

        input
            .iter()
            .enumerate()
            .map(|(idx, _item)| {
                let idx = idx + 1;
                MapStackItem {
                    origin: origin.stack[origin.stack.len() - idx],
                    new: self.stack.stack[self.stack.stack.len() - idx],
                }
            })
            .collect()

        // mapping.push(MapStackItem {
        //     origin: origin.stack[origin.stack.len() - 1],
        //     new: self.stack.stack[self.stack.stack.len() - 1],
        // });
        // mapping.push(MapStackItem {
        //     origin: origin.stack[origin.stack.len() - 2],
        //     new: self.stack.stack[self.stack.stack.len() - 2],
        // });
        // mapping.push(MapStackItem {
        //     origin: origin.stack[origin.stack.len() - 3],
        //     new: self.stack.stack[self.stack.stack.len() - 3],
        // });
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

        // mapping
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

#[derive(Debug, Clone)]
pub struct MapStackItem {
    pub origin: VarId,
    pub new: VarId,
}
