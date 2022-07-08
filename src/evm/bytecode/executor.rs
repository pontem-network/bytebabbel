use crate::evm::abi::FunHash;
use crate::evm::bytecode::block::InstructionBlock;
use crate::evm::bytecode::executor::block::{BlockId, Chain, ExecutedBlock, Execution};
use crate::evm::bytecode::executor::stack::{ExecutionStack, MemCell, FRAME_SIZE};
use crate::evm::bytecode::executor::statement::Statement;
use crate::evm::bytecode::loc::Loc;
use std::cell::RefCell;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

pub mod block;
pub mod stack;
pub mod statement;

pub fn exec(
    blocks: &BTreeMap<BlockId, InstructionBlock>,
    hash: FunHash,
    params: usize,
) -> BTreeMap<BlockId, Loc<ExecutedBlock>> {
    let mut exec_blocks = BTreeMap::new();
    let executor = Executor::new(hash, params);
    mark(0.into(), blocks, &mut exec_blocks, executor);
    exec_blocks
}

fn mark(
    block_id: BlockId,
    blocks: &BTreeMap<BlockId, InstructionBlock>,
    exec_blocks: &mut BTreeMap<BlockId, Loc<ExecutedBlock>>,
    mut executor: Executor,
) {
    let parent = executor.parent().clone();
    if let Some(block) = blocks.get(&block_id) {
        if let Entry::Vacant(e) = exec_blocks.entry(block_id) {
            let new_block = executor.exec(block);
            let jmp = new_block.last_jump(&parent);
            e.insert(new_block);
            if let Some(jmp) = jmp {
                for jmp in jmp.jumps() {
                    mark(jmp, blocks, exec_blocks, executor.clone());
                }
            }
        } else {
            let exec_block = exec_blocks.get_mut(&block_id).unwrap();
            if !exec_block.has_parent(&parent) {
                exec_block.merge(executor.exec(block).inner());
                if let Some(jmp) = exec_block.last_jump(&parent) {
                    for jmp in jmp.jumps() {
                        mark(jmp, blocks, exec_blocks, executor.clone());
                    }
                }
            }
        }
    }
}

#[derive(Default, Clone)]
pub struct Executor {
    stack: ExecutionStack,
    parent: Chain,
    hash: FunHash,
    params: usize,
    mem: Rc<RefCell<HashMap<MemCell, MemCell>>>,
}

impl Executor {
    pub fn new(hash: FunHash, params: usize) -> Executor {
        Executor {
            stack: Default::default(),
            parent: Chain::default(),
            hash,
            params,
            mem: Default::default(),
        }
    }

    pub fn with_parent(parent: BlockId) -> Executor {
        Executor {
            stack: Default::default(),
            parent: Chain::new(parent),
            hash: FunHash::default(),
            params: 0,
            mem: Default::default(),
        }
    }

    pub fn exec(&mut self, block: &InstructionBlock) -> Loc<ExecutedBlock> {
        let mut executed_block = block.wrap(ExecutedBlock::new(block.start.into()));
        let mut execution = Execution::default();

        let input_stack = self.stack.stack.clone();
        for inst in block.iter() {
            let pops = inst.pops();
            let pushes = inst.pushes();
            let mut st = Statement::new(self.stack.pop(pops));
            let to_push = st.perform(self, inst);
            assert_eq!(to_push.len(), pushes);
            self.stack.push(to_push);
            executed_block.instructions.push(inst.clone());
            execution.state.push(st);
        }
        execution.in_stack_items = self.stack.negative_stack.iter().cloned().collect();
        execution.in_stack_items.extend(input_stack);
        execution.out_stack_items = self.stack.stack.clone();

        executed_block
            .executions
            .insert(self.parent.clone(), execution);
        self.parent.join(block.start.into());
        executed_block
    }

    pub fn parent(&self) -> &Chain {
        &self.parent
    }

    pub fn call_data_size(&self) -> usize {
        self.hash.as_ref().len() + (self.params * FRAME_SIZE)
    }
}
