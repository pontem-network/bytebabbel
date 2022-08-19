/// High-Level Intermediate Representation
pub mod context;
pub mod executor;
pub mod ir;
pub mod mem;
pub mod optimization;
pub mod stack;

use crate::bytecode::block::InstructionBlock;
use crate::bytecode::flow_graph::{Flow, IfFlow, LoopFlow};
use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::instruction::Instruction;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::hir::ir::Hir;
use crate::bytecode::hir::optimization::IrOptimizer;
use crate::bytecode::hir::stack::FRAME_SIZE;
use crate::bytecode::types::{Function, U256};
use crate::BlockId;
use anyhow::{anyhow, bail, ensure, Error};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

pub struct HirTranslator<'a> {
    contract: &'a HashMap<BlockId, InstructionBlock>,
    contact_flow: Flow,
}

impl<'a> HirTranslator<'a> {
    pub fn new(
        contract: &'a HashMap<BlockId, InstructionBlock>,
        contact_flow: Flow,
    ) -> HirTranslator {
        HirTranslator {
            contract,
            contact_flow,
        }
    }

    pub fn translate(&self, fun: Function, contract_address: U256) -> Result<Hir, Error> {
        let mut ctx = Context::new(fun, contract_address);
        let mut ir = Hir::default();
        self.exec_flow(&self.contact_flow, &mut ir, &mut ctx)?;
        let ir = IrOptimizer::optimize(ir)?;
        ir.print();
        Ok(ir)
    }

    pub fn find_entry_points(&self) -> Result<Option<BlockId>, Error> {
        let mut ctx = Context::new(Function::default(), U256::zero());
        let mut ir = Hir::default();
        let result = self.exec_flow(&self.contact_flow, &mut ir, &mut ctx);
        match result {
            Ok(_) => Ok(None),
            Err(err) => {
                if let Some(SpecialError::CodeCopy(block)) = err.downcast_ref::<SpecialError>() {
                    Ok(Some(*block))
                } else {
                    Err(err)
                }
            }
        }
    }

    fn get_block(&self, block_id: &BlockId) -> Result<&InstructionBlock, Error> {
        self.contract
            .get(block_id)
            .ok_or_else(|| anyhow!("block not found"))
    }

    fn exec_flow(&self, flow: &Flow, ir: &mut Hir, ctx: &mut Context) -> Result<StopFlag, Error> {
        match flow {
            Flow::Block(id) => self.exec_flow_block(id, ir, ctx),
            Flow::Loop(loop_) => self.exec_flow_loop(loop_, ir, ctx),
            Flow::IF(if_) => self.exec_flow_if(if_, ir, ctx),
            Flow::Sequence(seq) => self.exec_flow_seq(seq, ir, ctx),
            Flow::Continue(block) => self.exec_flow_continue(block, ir, ctx),
        }
    }

    fn exec_flow_continue(
        &self,
        block: &BlockId,
        ir: &mut Hir,
        ctx: &mut Context,
    ) -> Result<StopFlag, Error> {
        let stack = ctx
            .get_loop(block)
            .ok_or_else(|| anyhow!("loop not found"))?;
        let mapping = ctx.map_stack(stack);
        let context = mapping
            .into_iter()
            .map(|map| Instruction::MapVar {
                id: map.origin,
                val: map.new,
            })
            .collect();

        ir.push_continue(*block, context);
        Ok(StopFlag::Continue)
    }

    fn exec_flow_loop(
        &self,
        loop_: &LoopFlow,
        ir: &mut Hir,
        ctx: &mut Context,
    ) -> Result<StopFlag, Error> {
        ctx.create_loop(loop_.jmp.block);
        let before_inst = ir.swap_instruction(vec![]);
        ctx.enter_loop();
        let res = self.exec_block(&loop_.jmp.block, ir, ctx)?;
        ctx.exit_loop();
        let cnd_block = ir.swap_instruction(before_inst);
        match res {
            BlockResult::Jmp(jmp) => {
                if loop_.br.is_true_br() {
                    if loop_.jmp.true_br == jmp {
                        bail!("infinite loop detected");
                    } else {
                        self.exec_flow(loop_.br.flow(), ir, ctx)
                    }
                } else if loop_.jmp.false_br == jmp {
                    bail!("infinite loop detected");
                } else {
                    self.exec_flow(loop_.br.flow(), ir, ctx)
                }
            }
            BlockResult::CndJmp {
                cnd,
                true_br,
                false_br,
            } => {
                ensure!(true_br == loop_.jmp.true_br, "invalid true_br");
                ensure!(false_br == loop_.jmp.false_br, "invalid false_br");
                ctx.enter_loop();
                let instructions = ir.swap_instruction(vec![]);
                self.exec_flow(loop_.br.flow(), ir, ctx)?;
                let loop_inst = ir.swap_instruction(instructions);
                ctx.exit_loop();
                ir.push_loop(
                    loop_.jmp.block,
                    cnd_block,
                    cnd,
                    loop_inst,
                    loop_.br.is_true_br(),
                );
                Ok(StopFlag::Continue)
            }
            _ => bail!("loop condition must be a conditional jump"),
        }
    }

    fn exec_flow_seq(
        &self,
        seq: &[Flow],
        ir: &mut Hir,
        ctx: &mut Context,
    ) -> Result<StopFlag, Error> {
        for flow in seq {
            if let StopFlag::Stop = self.exec_flow(flow, ir, ctx)? {
                return Ok(StopFlag::Stop);
            }
        }
        Ok(StopFlag::Continue)
    }

    fn exec_flow_block(
        &self,
        id: &BlockId,
        ir: &mut Hir,
        ctx: &mut Context,
    ) -> Result<StopFlag, Error> {
        let res = self.exec_block(id, ir, ctx)?;
        match res {
            BlockResult::Jmp(_) => {
                //no op
                Ok(StopFlag::Continue)
            }
            BlockResult::CndJmp { .. } => {
                bail!("conditional jump not supported");
            }
            BlockResult::Stop => {
                ir.stop();
                Ok(StopFlag::Stop)
            }
            BlockResult::Result { offset, len } => {
                let offset = ir.resolve_var(offset);
                let len = ir.resolve_var(len);

                if let (Some(offset), Some(len)) = (offset, len) {
                    let outputs = len / FRAME_SIZE;
                    let mut res = vec![];
                    for i in 0..outputs.as_usize() {
                        let i = U256::from(i);
                        let id = ctx
                            .mem_load(offset + i * U256::from(FRAME_SIZE))
                            .expect("mem load error");
                        res.push(id);
                    }
                    ir.result(res);
                } else {
                    bail!("result var not found");
                }
                Ok(StopFlag::Stop)
            }
            BlockResult::Abort(code) => {
                ir.abort(code);
                Ok(StopFlag::Stop)
            }
        }
    }

    fn exec_flow_if(
        &self,
        if_: &IfFlow,
        ir: &mut Hir,
        ctx: &mut Context,
    ) -> Result<StopFlag, Error> {
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
                Ok(StopFlag::Continue)
            }
            _ => Err(anyhow!("unexpected block result")),
        }
    }

    fn exec_block(
        &self,
        id: &BlockId,
        ir: &mut Hir,
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
                ExecutionResult::Result { offset, len } => {
                    return Ok(BlockResult::Result { offset, len });
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

#[derive(Debug)]
pub enum StopFlag {
    Continue,
    Stop,
}

#[derive(Debug, Default)]
pub struct BlockIO {
    pub inputs: Vec<VarId>,
    pub outputs: Vec<VarId>,
}

#[derive(Debug)]
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