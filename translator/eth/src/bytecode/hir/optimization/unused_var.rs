use crate::bytecode::hir::ir::statement::Statement;
use crate::bytecode::hir::ir::var::{Expr, VarId, Vars};
use crate::{BlockId, Hir};
use anyhow::{anyhow, Error};
use std::collections::{HashMap, HashSet};

pub struct UnusedVarClipper;

impl UnusedVarClipper {
    pub fn optimize(ir: Hir) -> Result<Hir, Error> {
        let analysis = ReachabilityAnalysis::calculate(&ir);
        let (mut vars, instructions, code_copy) = ir.into_inner();
        let mut ir = Hir::default();
        ir.set_code_copy(code_copy);

        let mut id_mapping = HashMap::new();
        Self::optimize_instructions(instructions, &analysis, &mut id_mapping, &mut vars, &mut ir)?;
        Ok(ir)
    }

    fn optimize_instructions(
        inst: Vec<Statement>,
        analysis: &ReachabilityAnalysis,
        id_mapping: &mut HashMap<VarId, VarId>,
        vars: &mut Vars,
        ir: &mut Hir,
    ) -> Result<(), Error> {
        for inst in inst {
            match inst {
                Statement::SetVar(id) => {
                    if analysis.is_reachable(&id) {
                        let var = vars.take(id)?;
                        let var = Self::map_var(var, id_mapping)?;
                        let new_id = ir.create_var(var);
                        id_mapping.insert(id, new_id);
                    }
                }
                Statement::MemStore { addr, var } => {
                    if analysis.is_reachable(&addr) && analysis.is_reachable(&var) {
                        let addr = Self::map_var_id(addr, id_mapping)?;
                        let var_id = Self::map_var_id(var, id_mapping)?;
                        ir.mstore(addr, var_id);
                    }
                }
                Statement::MemStore8 { addr, var } => {
                    if analysis.is_reachable(&addr) && analysis.is_reachable(&var) {
                        let addr = Self::map_var_id(addr, id_mapping)?;
                        let var_id = Self::map_var_id(var, id_mapping)?;
                        ir.mstore8(addr, var_id);
                    }
                }
                Statement::SStore { addr, var } => {
                    if analysis.is_reachable(&addr) && analysis.is_reachable(&var) {
                        let addr = Self::map_var_id(addr, id_mapping)?;
                        let var_id = Self::map_var_id(var, id_mapping)?;
                        ir.sstore(addr, var_id);
                    }
                }
                Statement::If {
                    condition,
                    true_branch,
                    false_branch,
                } => {
                    let condition = Self::map_var_id(condition, id_mapping)?;
                    let inst_before = ir.swap_instruction(vec![]);
                    Self::optimize_instructions(true_branch, analysis, id_mapping, vars, ir)?;
                    let true_br = ir.swap_instruction(vec![]);
                    Self::optimize_instructions(false_branch, analysis, id_mapping, vars, ir)?;
                    let false_br = ir.swap_instruction(inst_before);
                    ir.push_if(condition, true_br, false_br);
                }
                Statement::Loop {
                    id,
                    condition_block,
                    condition,
                    is_true_br_loop,
                    loop_br,
                } => {
                    let inst_before = ir.swap_instruction(vec![]);
                    Self::optimize_instructions(condition_block, analysis, id_mapping, vars, ir)?;
                    let condition_block = ir.swap_instruction(vec![]);
                    let condition = Self::map_var_id(condition, id_mapping)?;
                    Self::optimize_instructions(loop_br, analysis, id_mapping, vars, ir)?;
                    let loop_br = ir.swap_instruction(inst_before);
                    ir.push_loop(id, condition_block, condition, loop_br, is_true_br_loop);
                }
                Statement::Continue { loop_id, context } => {
                    let context = Self::optimize_context(context, loop_id, analysis);
                    let inst_before = ir.swap_instruction(vec![]);
                    Self::optimize_instructions(context, analysis, id_mapping, vars, ir)?;
                    let mapping = ir.swap_instruction(inst_before);
                    ir.push_continue(loop_id, mapping);
                }
                Statement::Result { offset, len } => {
                    let offset = Self::map_var_id(offset, id_mapping)?;
                    let len = Self::map_var_id(len, id_mapping)?;
                    ir.result(offset, len);
                }
                Statement::Stop => {
                    ir.stop();
                }
                Statement::Abort(code) => {
                    ir.abort(code);
                }
                Statement::MapVar { id, val } => {
                    if analysis.is_reachable(&id) && analysis.is_reachable(&val) {
                        let id = Self::map_var_id(id, id_mapping)?;
                        let val = Self::map_var_id(val, id_mapping)?;
                        ir.map_var(id, val);
                    }
                }
                Statement::Log {
                    offset,
                    len,
                    topics,
                } => {
                    let offset = Self::map_var_id(offset, id_mapping)?;
                    let len = Self::map_var_id(len, id_mapping)?;
                    let topics = topics
                        .into_iter()
                        .map(|t| Self::map_var_id(t, id_mapping))
                        .collect::<Result<Vec<_>, _>>()?;
                    ir.log(offset, len, topics);
                }
            }
        }
        Ok(())
    }

    fn optimize_context(
        context: Vec<Statement>,
        loop_id: BlockId,
        analysis: &ReachabilityAnalysis,
    ) -> Vec<Statement> {
        context
            .into_iter()
            .filter(|inst| match inst {
                Statement::MapVar { id, val: _ } => analysis.in_loop_context(loop_id, id),
                _ => unreachable!(),
            })
            .collect()
    }

    fn map_var_id(id: VarId, id_mapping: &HashMap<VarId, VarId>) -> Result<VarId, Error> {
        id_mapping
            .get(&id)
            .cloned()
            .ok_or_else(|| anyhow!("{:?} is not found", id))
    }

    fn map_var(var: Expr, id_mapping: &HashMap<VarId, VarId>) -> Result<Expr, Error> {
        Ok(match var {
            Expr::Val(val) => Expr::Val(val),
            Expr::UnaryOp(cmd, op) => Expr::UnaryOp(cmd, Self::map_var_id(op, id_mapping)?),
            Expr::BinaryOp(cmd, op1, op2) => Expr::BinaryOp(
                cmd,
                Self::map_var_id(op1, id_mapping)?,
                Self::map_var_id(op2, id_mapping)?,
            ),
            Expr::TernaryOp(cmd, op1, op2, op3) => Expr::TernaryOp(
                cmd,
                Self::map_var_id(op1, id_mapping)?,
                Self::map_var_id(op2, id_mapping)?,
                Self::map_var_id(op3, id_mapping)?,
            ),
            Expr::MLoad(addr) => {
                let addr = Self::map_var_id(addr, id_mapping)?;
                Expr::MLoad(addr)
            }
            Expr::SLoad(addr) => {
                let addr = Self::map_var_id(addr, id_mapping)?;
                Expr::SLoad(addr)
            }
            Expr::MSize => Expr::MSize,
            Expr::Signer => Expr::Signer,
            Expr::ArgsSize => Expr::ArgsSize,
            Expr::Args(arg) => Expr::Args(Self::map_var_id(arg, id_mapping)?),
            Expr::Hash(addr, len) => Expr::Hash(
                Self::map_var_id(addr, id_mapping)?,
                Self::map_var_id(len, id_mapping)?,
            ),
        })
    }
}

#[derive(Default)]
struct VarReachability {
    reachable_vars: HashSet<VarId>,
    vars: HashMap<VarId, HashSet<VarId>>,
}

impl VarReachability {
    fn push_var(&mut self, var: &VarId, ops: &[VarId]) {
        let entry = self.vars.entry(*var).or_insert_with(HashSet::new);
        entry.insert(*var);
        for op in ops {
            entry.insert(*op);
        }
    }

    fn mark_var_as_reachable(&mut self, var: &VarId) {
        self.reachable_vars.insert(*var);
        if let Some(deps) = self.vars.remove(var) {
            for dep in deps.iter() {
                self.mark_var_as_reachable(dep);
            }
        }
    }

    fn check_instructions(&mut self, ir: &Hir, instructions: &[Statement]) {
        for instruction in instructions {
            match instruction {
                Statement::SetVar(var_id) => {
                    self.insert_var(ir, var_id);
                }
                Statement::MemStore { addr, var } => {
                    self.mark_var_as_reachable(addr);
                    self.mark_var_as_reachable(var);
                }
                Statement::MemStore8 { addr, var } => {
                    self.mark_var_as_reachable(addr);
                    self.mark_var_as_reachable(var);
                }
                Statement::SStore { addr, var } => {
                    self.mark_var_as_reachable(addr);
                    self.mark_var_as_reachable(var);
                }
                Statement::Loop {
                    id: _,
                    condition_block,
                    condition,
                    is_true_br_loop: _,
                    loop_br,
                } => {
                    self.check_instructions(ir, condition_block);
                    self.mark_var_as_reachable(condition);
                    self.check_instructions(ir, loop_br);
                }
                Statement::If {
                    condition,
                    true_branch,
                    false_branch,
                } => {
                    self.mark_var_as_reachable(condition);
                    self.check_instructions(ir, true_branch);
                    self.check_instructions(ir, false_branch);
                }
                Statement::Stop => {}
                Statement::Abort(_) => {}
                Statement::Result { offset, len } => {
                    self.mark_var_as_reachable(offset);
                    self.mark_var_as_reachable(len);
                }
                Statement::Continue {
                    loop_id: _,
                    context,
                } => {
                    self.check_instructions(ir, context);
                }
                Statement::MapVar { id, val } => {
                    self.insert_var(ir, id);
                    self.insert_var(ir, val);
                    self.mark_var_as_reachable(val);
                    self.mark_var_as_reachable(id);
                }
                Statement::Log {
                    offset,
                    len,
                    topics,
                } => {
                    self.insert_var(ir, offset);
                    self.insert_var(ir, len);
                    for topic in topics {
                        self.insert_var(ir, topic);
                    }
                    self.mark_var_as_reachable(offset);
                    self.mark_var_as_reachable(len);
                    for topic in topics {
                        self.mark_var_as_reachable(topic);
                    }
                }
            }
        }
    }

    fn insert_var(&mut self, ir: &Hir, var: &VarId) {
        match ir.var(var) {
            Expr::Val(_) => {
                self.push_var(var, &[]);
            }
            Expr::UnaryOp(_, op) => {
                self.push_var(var, &[*op]);
            }
            Expr::BinaryOp(_, op1, op2) => {
                self.push_var(var, &[*op1, *op2]);
            }

            Expr::TernaryOp(_, op1, op2, op3) => {
                self.push_var(var, &[*op1, *op2, *op3]);
            }
            Expr::MLoad(addr) => {
                self.push_var(var, &[*addr]);
            }
            Expr::SLoad(addr) => {
                self.push_var(var, &[*addr]);
            }
            Expr::MSize => {}
            Expr::Signer => {}
            Expr::ArgsSize => {}
            Expr::Args(var_1) => {
                self.push_var(var, &[*var_1]);
            }
            Expr::Hash(var_1, var_2) => {
                self.push_var(var, &[*var_1, *var_2]);
            }
        }
    }

    fn finalize(self) -> HashSet<VarId> {
        self.reachable_vars
    }
}

struct ContextAnalyzer<'r> {
    loops: HashMap<BlockId, HashSet<VarId>>,
    reachable_vars: &'r HashSet<VarId>,
}

impl<'r> ContextAnalyzer<'r> {
    pub fn new(reachable_vars: &'r HashSet<VarId>) -> Self {
        ContextAnalyzer {
            loops: HashMap::new(),
            reachable_vars,
        }
    }

    pub fn analyze(&mut self, ir: &Hir) {
        let instructions = ir.as_ref();
        self.analyze_block(&[], instructions, ir);
    }

    fn analyze_loop(
        &mut self,
        loops: &[BlockId],
        condition: &VarId,
        condition_block: &[Statement],
        loop_br: &[Statement],
        ir: &Hir,
    ) {
        self.push_to_context(loops, condition, ir);
        self.analyze_block(loops, condition_block, ir);
        self.analyze_block(loops, loop_br, ir);
    }

    fn analyze_block(&mut self, loops: &[BlockId], block: &[Statement], ir: &Hir) {
        for inst in block {
            match inst {
                Statement::Loop {
                    id,
                    condition_block,
                    condition,
                    is_true_br_loop: _,
                    loop_br,
                } => {
                    let mut loops = loops.to_vec();
                    loops.push(*id);
                    self.analyze_loop(&loops, condition, condition_block, loop_br, ir);
                }
                Statement::SetVar(var) => {
                    self.push_to_context(loops, var, ir);
                }
                Statement::MapVar { id, val } => {
                    self.push_to_context(loops, id, ir);
                    self.push_to_context(loops, val, ir);
                }
                Statement::MemStore8 { addr, var } => {
                    self.push_to_context(loops, var, ir);
                    self.push_to_context(loops, addr, ir);
                }
                Statement::MemStore { addr, var } => {
                    self.push_to_context(loops, var, ir);
                    self.push_to_context(loops, addr, ir);
                }
                Statement::SStore { addr, var } => {
                    self.push_to_context(loops, var, ir);
                    self.push_to_context(loops, addr, ir);
                }
                Statement::If {
                    condition,
                    true_branch,
                    false_branch,
                } => {
                    self.push_to_context(loops, condition, ir);
                    self.analyze_block(loops, true_branch, ir);
                    self.analyze_block(loops, false_branch, ir);
                }
                Statement::Continue {
                    loop_id: _,
                    context: _,
                } => {
                    // no-op
                }
                Statement::Stop => {}
                Statement::Abort(_) => {}
                Statement::Result { offset, len } => {
                    self.push_to_context(loops, offset, ir);
                    self.push_to_context(loops, len, ir);
                }
                Statement::Log {
                    offset,
                    len,
                    topics,
                } => {
                    self.push_to_context(loops, offset, ir);
                    self.push_to_context(loops, len, ir);
                    for topic in topics {
                        self.push_to_context(loops, topic, ir);
                    }
                }
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn resolve_ids(&self, var_id: &VarId, ir: &Hir, ids: &mut HashSet<VarId>) {
        ids.insert(*var_id);
        match ir.var(var_id) {
            Expr::Val(_) => {}
            Expr::UnaryOp(_, op) => {
                self.resolve_ids(op, ir, ids);
            }
            Expr::BinaryOp(_, op1, op2) => {
                self.resolve_ids(op1, ir, ids);
                self.resolve_ids(op2, ir, ids);
            }
            Expr::TernaryOp(_, op1, op2, op3) => {
                self.resolve_ids(op1, ir, ids);
                self.resolve_ids(op2, ir, ids);
                self.resolve_ids(op3, ir, ids);
            }
            Expr::MLoad(addr) => {
                self.resolve_ids(addr, ir, ids);
            }
            Expr::SLoad(addr) => {
                self.resolve_ids(addr, ir, ids);
            }
            Expr::MSize => {}
            Expr::Signer => {}
            Expr::ArgsSize => {}
            Expr::Args(var) => {
                self.resolve_ids(var, ir, ids);
            }
            Expr::Hash(addr, len) => {
                self.resolve_ids(addr, ir, ids);
                self.resolve_ids(len, ir, ids);
            }
        }
    }

    fn push_to_context(&mut self, loops: &[BlockId], var_id: &VarId, ir: &Hir) {
        if self.reachable_vars.contains(var_id) {
            let mut ids = HashSet::new();
            self.resolve_ids(var_id, ir, &mut ids);

            for loop_id in loops {
                self.loops
                    .entry(*loop_id)
                    .or_insert_with(HashSet::new)
                    .extend(&ids);
            }
        }
    }
}

#[derive(Default, Debug)]
struct ReachabilityAnalysis {
    reachable_vars: HashSet<VarId>,
    loop_context: HashMap<BlockId, HashSet<VarId>>,
}

impl ReachabilityAnalysis {
    pub fn calculate(ir: &Hir) -> Self {
        let mut vars = VarReachability::default();
        let instructions = ir.as_ref();
        vars.check_instructions(ir, instructions);
        let reachable_vars = vars.finalize();

        let mut context_analyzer = ContextAnalyzer::new(&reachable_vars);
        context_analyzer.analyze(ir);
        let loop_context = context_analyzer.loops;

        ReachabilityAnalysis {
            reachable_vars,
            loop_context,
        }
    }

    pub fn is_reachable(&self, var: &VarId) -> bool {
        self.reachable_vars.contains(var)
    }

    pub fn in_loop_context(&self, loop_br: BlockId, var: &VarId) -> bool {
        self.loop_context
            .get(&loop_br)
            .map_or(false, |vars| vars.contains(var))
    }
}
