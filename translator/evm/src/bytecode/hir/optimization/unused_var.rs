use crate::bytecode::hir::ir::instruction::Instruction;
use crate::bytecode::hir::ir::var::{Var, VarId, Vars};
use crate::{BlockId, Hir};
use anyhow::{anyhow, Error};
use std::collections::{HashMap, HashSet};

pub struct UnusedVarClipper;

impl UnusedVarClipper {
    pub fn optimize(ir: Hir) -> Result<Hir, Error> {
        let analysis = ReachabilityAnalysis::calculate(&ir);
        let (mut vars, instructions) = ir.into_inner();
        let mut ir = Hir::default();
        let mut id_mapping = HashMap::new();
        Self::optimize_instructions(instructions, &analysis, &mut id_mapping, &mut vars, &mut ir)?;
        Ok(ir)
    }

    fn optimize_instructions(
        inst: Vec<Instruction>,
        analysis: &ReachabilityAnalysis,
        id_mapping: &mut HashMap<VarId, VarId>,
        vars: &mut Vars,
        ir: &mut Hir,
    ) -> Result<(), Error> {
        for inst in inst {
            match inst {
                Instruction::SetVar(id) => {
                    if analysis.is_reachable(&id) {
                        let var = vars.take(id)?;
                        let var = Self::map_var(var, id_mapping)?;
                        let new_id = ir.create_var(var);
                        id_mapping.insert(id, new_id);
                    }
                }
                Instruction::MemStore(addr, var_id) => {
                    if analysis.is_reachable(&var_id) {
                        let var_id = Self::map_var_id(var_id, id_mapping)?;
                        ir.mem_store(addr, var_id);
                    }
                }
                Instruction::MemLoad(addr, var_id) => {
                    if analysis.is_reachable(&var_id) {
                        let var_id = Self::map_var_id(var_id, id_mapping)?;
                        ir.mem_load(addr, var_id);
                    }
                }
                Instruction::If {
                    condition,
                    true_branch,
                    false_branch,
                } => {
                    let condition = Self::map_var_id(condition, id_mapping)?;
                    let inst_before = ir.swap_instruction(vec![]);
                    Self::optimize_instructions(true_branch, &analysis, id_mapping, vars, ir)?;
                    let true_br = ir.swap_instruction(vec![]);
                    Self::optimize_instructions(false_branch, &analysis, id_mapping, vars, ir)?;
                    let false_br = ir.swap_instruction(inst_before);
                    ir.push_if(condition, true_br, false_br);
                }
                Instruction::Loop {
                    id,
                    condition_block,
                    condition,
                    is_true_br_loop,
                    loop_br,
                } => {
                    let inst_before = ir.swap_instruction(vec![]);
                    Self::optimize_instructions(condition_block, &analysis, id_mapping, vars, ir)?;
                    let condition_block = ir.swap_instruction(vec![]);
                    let condition = Self::map_var_id(condition, id_mapping)?;
                    Self::optimize_instructions(loop_br, &analysis, id_mapping, vars, ir)?;
                    let loop_br = ir.swap_instruction(inst_before);
                    ir.push_loop(id, condition_block, condition, loop_br, is_true_br_loop);
                }
                Instruction::Continue { loop_id, context } => {
                    let context = Self::optimize_context(context, loop_id, &analysis);
                    let inst_before = ir.swap_instruction(vec![]);
                    Self::optimize_instructions(context, &analysis, id_mapping, vars, ir)?;
                    let mapping = ir.swap_instruction(inst_before);
                    ir.push_continue(loop_id, mapping);
                }
                Instruction::Result(vars) => {
                    let res = vars
                        .iter()
                        .filter_map(|var| id_mapping.get(var).cloned())
                        .collect();
                    ir.result(res);
                }
                Instruction::Stop => {
                    ir.stop();
                }
                Instruction::Abort(code) => {
                    ir.abort(code);
                }
                Instruction::MapVar { id, val } => {
                    if analysis.is_reachable(&id) && analysis.is_reachable(&val) {
                        let id = Self::map_var_id(id, id_mapping)?;
                        let val = Self::map_var_id(val, id_mapping)?;
                        ir.map_var(id, val);
                    }
                }
            }
        }
        Ok(())
    }

    fn optimize_context(
        context: Vec<Instruction>,
        loop_id: BlockId,
        analysis: &ReachabilityAnalysis,
    ) -> Vec<Instruction> {
        context
            .into_iter()
            .filter(|inst| match inst {
                Instruction::MapVar { id, val: _ } => analysis.in_loop_context(loop_id, &id),
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

    fn map_var(var: Var, id_mapping: &HashMap<VarId, VarId>) -> Result<Var, Error> {
        Ok(match var {
            Var::Val(val) => Var::Val(val),
            Var::Param(param) => Var::Param(param),
            Var::UnaryOp(cmd, op) => Var::UnaryOp(cmd, Self::map_var_id(op, id_mapping)?),
            Var::BinaryOp(cmd, op1, op2) => Var::BinaryOp(
                cmd,
                Self::map_var_id(op1, id_mapping)?,
                Self::map_var_id(op2, id_mapping)?,
            ),
            Var::TernaryOp(cmd, op1, op2, op3) => Var::TernaryOp(
                cmd,
                Self::map_var_id(op1, id_mapping)?,
                Self::map_var_id(op2, id_mapping)?,
                Self::map_var_id(op3, id_mapping)?,
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

    fn check_instructions(&mut self, ir: &Hir, instructions: &[Instruction]) {
        for instruction in instructions {
            match instruction {
                Instruction::SetVar(var_id) => {
                    self.insert_var(ir, var_id);
                }
                Instruction::MemStore(_, var_id) => {
                    self.mark_var_as_reachable(var_id);
                }
                Instruction::MemLoad(_, var_id) => {
                    self.insert_var(ir, var_id);
                }
                Instruction::Loop {
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
                Instruction::If {
                    condition,
                    true_branch,
                    false_branch,
                } => {
                    self.mark_var_as_reachable(condition);
                    self.check_instructions(ir, true_branch);
                    self.check_instructions(ir, false_branch);
                }
                Instruction::Stop => {}
                Instruction::Abort(_) => {}
                Instruction::Result(vars) => {
                    for var_id in vars {
                        self.mark_var_as_reachable(var_id);
                    }
                }
                Instruction::Continue {
                    loop_id: _,
                    context,
                } => {
                    self.check_instructions(ir, context);
                }
                Instruction::MapVar { id, val } => {
                    self.insert_var(ir, id);
                    self.insert_var(ir, val);
                    self.mark_var_as_reachable(val);
                    self.mark_var_as_reachable(id);
                }
            }
        }
    }

    fn insert_var(&mut self, ir: &Hir, var: &VarId) {
        match ir.var(var) {
            Var::Val(_) => {
                self.push_var(var, &[]);
            }
            Var::Param(_) => {}
            Var::UnaryOp(_, op) => {
                self.push_var(var, &[*op]);
            }
            Var::BinaryOp(_, op1, op2) => {
                self.push_var(var, &[*op1, *op2]);
            }

            Var::TernaryOp(_, op1, op2, op3) => {
                self.push_var(var, &[*op1, *op2, *op3]);
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
        condition_block: &[Instruction],
        loop_br: &[Instruction],
        ir: &Hir,
    ) {
        self.push_to_context(loops, condition, ir);
        self.analyze_block(loops, condition_block, ir);
        self.analyze_block(loops, loop_br, ir);
    }

    fn analyze_block(&mut self, loops: &[BlockId], block: &[Instruction], ir: &Hir) {
        for inst in block {
            match inst {
                Instruction::Loop {
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
                Instruction::SetVar(var) => {
                    self.push_to_context(loops, var, ir);
                }
                Instruction::MapVar { id, val } => {
                    self.push_to_context(loops, id, ir);
                    self.push_to_context(loops, val, ir);
                }
                Instruction::MemStore(_, id) => {
                    self.push_to_context(loops, id, ir);
                }
                Instruction::MemLoad(_, id) => {
                    self.push_to_context(loops, id, ir);
                }
                Instruction::If {
                    condition,
                    true_branch,
                    false_branch,
                } => {
                    self.push_to_context(loops, condition, ir);
                    self.analyze_block(loops, true_branch, ir);
                    self.analyze_block(loops, false_branch, ir);
                }
                Instruction::Continue {
                    loop_id: _,
                    context: _,
                } => {
                    // no-op
                }
                Instruction::Stop => {}
                Instruction::Abort(_) => {}
                Instruction::Result(val) => {
                    for v in val {
                        self.push_to_context(loops, v, ir);
                    }
                }
            }
        }
    }

    fn resolve_ids(&self, var_id: &VarId, ir: &Hir, ids: &mut HashSet<VarId>) {
        ids.insert(*var_id);
        match ir.var(var_id) {
            Var::Val(_) => {}
            Var::Param(_) => {}
            Var::UnaryOp(_, op) => {
                self.resolve_ids(op, ir, ids);
            }
            Var::BinaryOp(_, op1, op2) => {
                self.resolve_ids(op1, ir, ids);
                self.resolve_ids(op2, ir, ids);
            }
            Var::TernaryOp(_, op1, op2, op3) => {
                self.resolve_ids(op1, ir, ids);
                self.resolve_ids(op2, ir, ids);
                self.resolve_ids(op3, ir, ids);
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
        vars.check_instructions(ir, &instructions);
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
