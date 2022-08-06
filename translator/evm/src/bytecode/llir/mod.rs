pub mod context;
pub mod executor;
pub mod ir;
pub mod mem;
pub mod stack;

use crate::bytecode::block::InstructionBlock;
use crate::bytecode::flow_graph::Flow;
use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::ir::var::VarId;
use crate::bytecode::llir::ir::Ir;
use crate::bytecode::types::{Function, U256};
use crate::BlockId;
use anyhow::{anyhow, ensure, Error};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

pub struct Translator<'a> {
    contract: &'a HashMap<BlockId, InstructionBlock>,
    contact_flow: Flow,
}

impl<'a> Translator<'a> {
    pub fn new(contract: &'a HashMap<BlockId, InstructionBlock>, contact_flow: Flow) -> Translator {
        Translator {
            contract,
            contact_flow,
        }
    }

    pub fn translate(&self, fun: Function, contract_address: U256) -> Result<Ir, Error> {
        let mut ctx = Context::new(fun, contract_address);
        let mut ir = Ir::default();
        self.exec_flow(&self.contact_flow, &mut ir, &mut ctx)?;
        ir.print();
        Ok(ir)
    }

    fn get_block(&self, block_id: &BlockId) -> Result<&InstructionBlock, Error> {
        self.contract
            .get(block_id)
            .ok_or_else(|| anyhow!("block not found"))
    }

    fn exec_flow(&self, flow: &Flow, ir: &mut Ir, ctx: &mut Context) -> Result<(), Error> {
        match flow {
            Flow::Block(id) => {
                self.exec_block(id, ir, ctx)?;
                Ok(())
            }
            Flow::Loop(_loop_) => {
                todo!()
            }
            Flow::IF(if_) => {
                let cnd_block = if_.jmp.block;
                let res = self.exec_block(&cnd_block, ir, ctx)?;
                match res {
                    BlockResult::Jmp(jmp) => {
                        if jmp == if_.jmp.true_br {
                            self.exec_flow(&if_.true_br, ir, ctx)
                        } else if jmp == if_.jmp.false_br {
                            self.exec_flow(&if_.false_br, ir, ctx)
                        } else {
                            Err(anyhow!("invalid jmp"))
                        }
                    }
                    BlockResult::CndJmp {
                        cnd,
                        true_br,
                        false_br,
                    } => {
                        ensure!(true_br == if_.jmp.true_br, "invalid true_br");
                        ensure!(false_br == if_.jmp.false_br, "invalid false_br");
                        let instructions = ir.swap_instruction(vec![]);
                        self.exec_flow(&if_.true_br, ir, &mut ctx.clone())?;
                        let true_ir = ir.swap_instruction(vec![]);
                        self.exec_flow(&if_.false_br, ir, &mut ctx.clone())?;
                        let false_ir = ir.swap_instruction(instructions);
                        ir.push_if(cnd, true_ir, false_ir);
                        Ok(())
                    }
                    _ => Err(anyhow!("unexpected block result")),
                }
            }
            Flow::Sequence(seq) => {
                for flow in seq {
                    self.exec_flow(flow, ir, ctx)?;
                }
                Ok(())
            }
        }
    }

    fn exec_block(
        &self,
        id: &BlockId,
        ir: &mut Ir,
        ctx: &mut Context,
    ) -> Result<BlockResult, Error> {
        let block = self.get_block(id)?;
        for inst in block.iter() {
            let pops = inst.pops();
            let params = ctx.pop_stack(pops);
            ensure!(pops == params.len(), "Invalid stake state.");
            let res = inst.handle(params, ir, ctx);
            match res {
                ExecutionResult::CodeCopy(offset) => {
                    return Err(SpecialError::CodeCopy(offset).into());
                }
                ExecutionResult::Abort(code) => {
                    return Ok(BlockResult::Abort(code));
                }
                ExecutionResult::None => {}
                ExecutionResult::Output(stack) => {
                    ensure!(stack.len() == inst.pushes(), "Invalid stake state.");
                    ctx.push_stack(stack);
                }
                ExecutionResult::Result {
                    offset,
                    len,
                    revert,
                } => {
                    return Ok(BlockResult::Result {
                        offset,
                        len,
                        revert,
                    });
                }
                ExecutionResult::Stop => {
                    return Ok(BlockResult::Stop);
                }
                ExecutionResult::Jmp(block) => {
                    return Ok(BlockResult::Jmp(block));
                }
                ExecutionResult::CndJmp {
                    cnd,
                    true_br,
                    false_br,
                } => {
                    return Ok(BlockResult::CndJmp {
                        cnd,
                        true_br,
                        false_br,
                    });
                }
            }
        }
        Ok(BlockResult::Jmp(
            block.last().map(|i| BlockId(i.next())).unwrap_or_default(),
        ))
    }
}

pub enum BlockResult {
    Jmp(BlockId),
    CndJmp {
        cnd: VarId,
        true_br: BlockId,
        false_br: BlockId,
    },
    Stop,
    Result {
        offset: VarId,
        len: VarId,
        revert: bool,
    },
    Abort(u8),
}

#[derive(Debug)]
enum SpecialError {
    CodeCopy(BlockId),
}

impl Display for SpecialError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SpecialError {}
