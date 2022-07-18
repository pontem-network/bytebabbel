use crate::evm::bytecode::block::{BlockId, InstructionBlock};
use crate::evm::bytecode::executor::env::{Env, Function};
use crate::evm::bytecode::executor::execution::FunctionFlow;
use crate::evm::bytecode::executor::instructions::execute;
use crate::evm::bytecode::executor::mem::Memory;
use crate::evm::bytecode::executor::stack::{Frame, Stack, StackFrame, FRAME_SIZE};
use crate::evm::bytecode::executor::types::U256;
use crate::evm::bytecode::instruction::Instruction;
use anyhow::{anyhow, ensure, Error};
use std::collections::BTreeMap;

pub mod debug;
pub mod env;
pub mod execution;
pub mod instructions;
pub mod mem;
pub mod ops;
pub mod stack;
pub mod types;

pub struct StaticExecutor<'a> {
    mem: Memory,
    stack: Stack,
    contract: &'a BTreeMap<BlockId, InstructionBlock>,
    new_code_offset: Option<BlockId>,
}

impl<'a> StaticExecutor<'a> {
    pub fn new(contract: &'a BTreeMap<BlockId, InstructionBlock>) -> StaticExecutor {
        StaticExecutor {
            mem: Memory::default(),
            stack: Stack::default(),
            contract,
            new_code_offset: None,
        }
    }

    fn inherit(&self) -> StaticExecutor {
        StaticExecutor {
            mem: self.mem.clone(),
            stack: self.stack.clone(),
            contract: self.contract,
            new_code_offset: None,
        }
    }

    pub fn exec(&mut self, fun: Function) -> Result<FunctionFlow, Error> {
        let env = Env::new(fun);
        let mut flow = FunctionFlow::default();
        self._exec_block(BlockId::default(), &env, &mut flow)?;
        self.mem.clean();
        self.stack.clean();
        self.new_code_offset = None;
        Ok(flow)
    }

    pub fn find_next_entry_point(&mut self) -> Result<Option<BlockId>, Error> {
        let mut block_id = BlockId::default();
        loop {
            let block = self
                .contract
                .get(&block_id)
                .ok_or_else(|| anyhow!("Block not found: {block_id}"))?;
            let res = self.exec_block(block, &Env::default())?;
            if self.new_code_offset.is_some() {
                return Ok(self.new_code_offset.take());
            }

            match res {
                Some(ExecResult::Jmp(Jump::UnCnd(block))) => {
                    block_id = block;
                }
                Some(ExecResult::Jmp(Jump::Cnd { .. })) => {
                    todo!()
                }
                Some(ExecResult::Abort(_)) => {
                    return Ok(None);
                }
                Some(ExecResult::Return { .. }) => {
                    return Ok(None);
                }
                None => {
                    return Ok(None);
                }
            }
        }
    }

    fn _exec_block(
        &mut self,
        block: BlockId,
        env: &Env,
        flow: &mut FunctionFlow,
    ) -> Result<(), Error> {
        let block = self
            .contract
            .get(&block)
            .ok_or_else(|| anyhow!("Block not found: {block}"))?;
        match self.exec_block(block, env)? {
            Some(ExecResult::Jmp(Jump::Cnd {
                cnd,
                true_br,
                false_br,
            })) => self.handle_cnd_jmp(cnd, true_br, false_br, env, flow),
            Some(ExecResult::Abort(code)) => {
                flow.abort(code);
                Ok(())
            }
            Some(ExecResult::Return { offset, len }) => self.handle_result(env, offset, len, flow),
            Some(ExecResult::Jmp(Jump::UnCnd(block))) => self._exec_block(block, env, flow),
            None => {
                let next_block = BlockId::from(block.end + 1);
                self._exec_block(next_block, env, flow)
            }
        }
    }

    fn handle_cnd_jmp(
        &mut self,
        cnd: StackFrame,
        true_br: BlockId,
        false_br: BlockId,
        env: &Env,
        flow: &mut FunctionFlow,
    ) -> Result<(), Error> {
        flow.calc_stack(cnd);

        let mut true_br_executor = self.inherit();
        let mut true_br_flow = FunctionFlow::default();
        true_br_flow.var_seq = flow.var_seq;
        true_br_executor._exec_block(true_br, env, &mut true_br_flow)?;

        let mut false_br_executor = self.inherit();
        let mut false_br_flow = FunctionFlow::default();
        false_br_flow.var_seq = true_br_flow.var_seq;
        false_br_executor._exec_block(false_br, env, &mut false_br_flow)?;
        flow.var_seq = false_br_flow.var_seq;
        flow.brunch(true_br_flow, false_br_flow);
        Ok(())
    }

    fn handle_result(
        &mut self,
        _env: &Env,
        offset: StackFrame,
        len: StackFrame,
        flow: &mut FunctionFlow,
    ) -> Result<(), Error> {
        let len = len
            .as_u256()
            .ok_or_else(|| anyhow!("unsupported dynamic result len"))?;
        let offset = offset
            .as_u256()
            .ok_or_else(|| anyhow!("unsupported dynamic result len"))?;

        let outputs = len.as_usize() / FRAME_SIZE;
        for i in 0..outputs {
            let calculation = self.mem.load(&StackFrame::new(Frame::Val(
                offset + U256::from(i * FRAME_SIZE),
            )));
            calculation.mark_as_used();

            let var = flow.calc_var(calculation);
            flow.set_result(var);
        }

        Ok(())
    }

    fn exec_block(
        &mut self,
        block: &InstructionBlock,
        env: &Env,
    ) -> Result<Option<ExecResult>, Error> {
        let next_block = BlockId::from(block.end + 1);
        for inst in block.iter() {
            let res = self.exec_instruction(inst, env, next_block)?;
            if let Some(res) = res {
                return Ok(Some(res));
            }
        }
        Ok(None)
    }

    fn exec_instruction(
        &mut self,
        inst: &Instruction,
        env: &Env,
        next_block: BlockId,
    ) -> Result<Option<ExecResult>, Error> {
        log::trace!("{}", inst);
        let pops = inst.pops();
        let mut input = self.stack.pop(pops);
        ensure!(pops == input.len(), "Invalid stake state.");
        let mut ctx = Context {
            executor: self,
            input: &mut input,
            env,
            next_block,
            result: None,
        };
        let pushes = execute(inst, &mut ctx)?;
        let result = ctx.result.take();
        ensure!(pushes.len() == inst.pushes(), "Invalid stake state.");
        self.print_stack(&input, &pushes);
        self.stack.push(pushes);
        Ok(result)
    }

    fn print_stack(&self, input: &[StackFrame], output: &[StackFrame]) {
        log::trace!("      stack diff: {input:?} => {output:?}");
    }
}

pub struct Context<'a, 'b> {
    executor: &'b mut StaticExecutor<'a>,
    pub input: &'b mut [StackFrame],
    env: &'b Env,
    pub next_block: BlockId,
    result: Option<ExecResult>,
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn mem_store(&mut self, rf: StackFrame, val: StackFrame) {
        log::trace!("      var {:?} = {:?}", rf, val);
        self.executor.mem.store(rf, val);
    }

    pub fn mem_load(&mut self, rf: &StackFrame) -> StackFrame {
        let val = self.executor.mem.load(rf);
        log::trace!("      var {:?} = {:?}", rf, val);
        val
    }

    pub fn env(&self) -> &Env {
        self.env
    }

    pub fn set_jump(&mut self, jmp: Jump) {
        self.result = Some(ExecResult::Jmp(jmp));
    }

    pub fn set_abort(&mut self, code: u8) {
        self.result = Some(ExecResult::Abort(code));
    }

    pub fn set_result(&mut self, offset: StackFrame, len: StackFrame) {
        self.result = Some(ExecResult::Return { offset, len })
    }

    pub fn set_code_offset(&mut self, offset: BlockId) {
        self.executor.new_code_offset = Some(offset);
    }
}

#[derive(Debug)]
pub enum Jump {
    Cnd {
        cnd: StackFrame,
        true_br: BlockId,
        false_br: BlockId,
    },
    UnCnd(BlockId),
}

#[derive(Debug)]
pub enum ExecResult {
    Jmp(Jump),
    Abort(u8),
    Return { offset: StackFrame, len: StackFrame },
}
