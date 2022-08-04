use crate::bytecode::block::{BlockId, InstructionBlock};
use crate::bytecode::executor::env::{Env, Function};
use crate::bytecode::executor::execution::{FunctionFlow, Var};
use crate::bytecode::executor::instructions::execute;
use crate::bytecode::executor::mem::Memory;
use crate::bytecode::executor::stack::{Frame, Stack, StackFrame, FRAME_SIZE};
use crate::bytecode::executor::types::U256;
use crate::bytecode::flow_graph;
use crate::bytecode::instruction::Instruction;
use crate::bytecode::llir::Translator;
use anyhow::{anyhow, ensure, Error};
use log::log_enabled;
use log::Level;
use std::collections::HashMap;

pub mod debug;
pub mod env;
pub mod execution;
pub mod history;
pub mod instructions;
pub mod mem;
pub mod ops;
pub mod stack;
pub mod types;

pub struct StaticExecutor<'a> {
    mem: Memory,
    stack: Stack,
    contract: &'a HashMap<BlockId, InstructionBlock>,
    new_code_offset: Option<BlockId>,
}

impl<'a> StaticExecutor<'a> {
    pub fn new(contract: &'a HashMap<BlockId, InstructionBlock>) -> StaticExecutor {
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
        let next_block = BlockId::default();
        self.exec_with_ctx(&env, &mut flow, next_block)?;
        Ok(flow)
    }

    fn exec_with_ctx(
        &mut self,
        env: &Env,
        flow: &mut FunctionFlow,
        mut next_block: BlockId,
    ) -> Result<(), Error> {
        loop {
            let res = self.exec_block(next_block, env, flow)?;
            match res {
                Jump::None => {
                    break;
                }
                Jump::UnCnd(next) => {
                    next_block = next;
                }
                Jump::Cnd {
                    cnd,
                    true_br,
                    false_br,
                } => {
                    self.handle_cnd_jmp(cnd, true_br, false_br, env, flow)?;
                    break;
                }
            }
        }

        self.mem.clean();
        self.stack.clean();
        self.new_code_offset = None;
        Ok(())
    }

    pub fn find_next_entry_point(&mut self) -> Result<Option<BlockId>, Error> {
        let mut block_id = BlockId::default();
        loop {
            let block = self
                .contract
                .get(&block_id)
                .ok_or_else(|| anyhow!("Block not found: {block_id}"))?;
            let res = self.perform_instructions(block, &Env::default())?;
            if self.new_code_offset.is_some() {
                return Ok(self.new_code_offset.take());
            }

            match res {
                Some(ExecResult::Jmp(Jump::UnCnd(block))) => {
                    block_id = block;
                }
                _ => {
                    return Ok(None);
                }
            }
        }
    }

    fn exec_block(
        &mut self,
        block: BlockId,
        env: &Env,
        flow: &mut FunctionFlow,
    ) -> Result<Jump, Error> {
        let block = self
            .contract
            .get(&block)
            .ok_or_else(|| anyhow!("Block not found: {block}"))?;

        match self.perform_instructions(block, env)? {
            Some(ExecResult::Abort(code)) => {
                flow.abort(code);
                Ok(Jump::None)
            }
            Some(ExecResult::Return { offset, len }) => {
                self.handle_result(env, offset, len, flow)?;
                Ok(Jump::None)
            }
            None => {
                let next = block.end + block.last().map(|ins| ins.size()).unwrap_or(1);
                Ok(Jump::UnCnd(BlockId::from(next)))
            }
            Some(ExecResult::Stop) => {
                flow.set_result(Var::unit());
                Ok(Jump::None)
            }
            Some(ExecResult::Revert { offset, len }) => {
                self.handle_revert(env, offset, len, flow)?;
                Ok(Jump::None)
            }
            Some(ExecResult::Jmp(jmp)) => Ok(jmp),
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
        let mut true_br_executor = self.inherit();
        let mut true_br_flow = FunctionFlow::default();
        true_br_flow.var_seq = flow.var_seq;

        true_br_executor.exec_with_ctx(env, &mut true_br_flow, true_br)?;

        let mut false_br_executor = self.inherit();
        let mut false_br_flow = FunctionFlow::default();
        false_br_flow.var_seq = true_br_flow.var_seq;

        false_br_executor.exec_with_ctx(env, &mut false_br_flow, false_br)?;

        flow.var_seq = false_br_flow.var_seq;
        flow.brunch(cnd, true_br_flow, false_br_flow);
        Ok(())
    }

    fn handle_result(
        &mut self,
        _env: &Env,
        offset: StackFrame,
        len: StackFrame,
        flow: &mut FunctionFlow,
    ) -> Result<(), Error> {
        if log_enabled!(Level::Trace) {
            log::trace!("mem:\n{}", self.mem);
        }

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

    fn handle_revert(
        &mut self,
        _env: &Env,
        _offset: StackFrame,
        _len: StackFrame,
        flow: &mut FunctionFlow,
    ) -> Result<(), Error> {
        if log_enabled!(Level::Trace) {
            log::trace!("mem:\n{}", self.mem);
        }
        flow.abort(0);
        Ok(())
    }

    #[inline]
    fn perform_instructions(
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

    pub fn set_revert(&mut self, offset: StackFrame, len: StackFrame) {
        self.result = Some(ExecResult::Revert { offset, len })
    }

    pub fn set_stop(&mut self) {
        self.result = Some(ExecResult::Stop)
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
    None,
}

#[derive(Debug)]
pub enum ExecResult {
    Jmp(Jump),
    Abort(u8),
    Return { offset: StackFrame, len: StackFrame },
    Revert { offset: StackFrame, len: StackFrame },
    Stop,
}

pub struct Loop {}
