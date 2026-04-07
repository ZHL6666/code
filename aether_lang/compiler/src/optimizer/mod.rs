//! Aether 优化器模块
//! 
//! 对 IR 进行各种优化

use crate::ir::*;
use crate::config::CompilerConfig;
use crate::error::Result;
use std::collections::{HashMap, HashSet};

/// 优化器
pub struct Optimizer {
    config: CompilerConfig,
}

impl Optimizer {
    /// 创建新的优化器
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// 执行优化
    pub fn optimize(&self, module_ir: ModuleIR) -> Result<ModuleIR> {
        if self.config.optimization_level == 0 {
            return Ok(module_ir);
        }

        let mut optimized = module_ir;

        // 根据优化级别应用不同的优化
        for function in &mut optimized.functions {
            // 常量折叠
            if self.config.optimization_level >= 1 {
                self.constant_folding(function)?;
            }

            // 死代码消除
            if self.config.optimization_level >= 1 {
                self.dead_code_elimination(function)?;
            }

            // 公共子表达式消除
            if self.config.optimization_level >= 2 {
                self.common_subexpression_elimination(function)?;
            }

            // 循环不变量外提
            if self.config.optimization_level >= 2 {
                self.loop_invariant_code_motion(function)?;
            }

            // 内联优化
            if self.config.optimization_level >= 3 {
                self.inline_optimization(&mut optimized)?;
            }
        }

        Ok(optimized)
    }

    /// 常量折叠
    fn constant_folding(&self, func: &mut FunctionIR) -> Result<()> {
        let mut changed = true;
        while changed {
            changed = false;
            for block in &mut func.blocks {
                let mut i = 0;
                while i < block.instructions.len() {
                    if let Instruction::BinOp(dest, left, op, right) = &block.instructions[i].clone() {
                        // 检查两个操作数是否都是常量
                        if let (Some(left_val), Some(right_val)) = (
                            self.get_constant_value(func, left),
                            self.get_constant_value(func, right),
                        ) {
                            let result = self.evaluate_binary_op(left_val, *op, right_val);
                            if let Some(result_val) = result {
                                // 替换为常量赋值
                                block.instructions[i] = Instruction::Const(dest.clone(), result_val);
                                changed = true;
                            }
                        }
                    }
                    
                    // 常量传播：如果右边是常量复制，直接替换
                    if let Instruction::Copy(dest, src) = &block.instructions[i].clone() {
                        if let Some(val) = self.get_constant_value(func, src) {
                            block.instructions[i] = Instruction::Const(dest.clone(), val);
                            changed = true;
                        }
                    }
                    
                    i += 1;
                }
            }
        }
        Ok(())
    }

    /// 获取常量值
    fn get_constant_value(&self, func: &FunctionIR, operand: &Operand) -> Option<Literal> {
        match operand {
            Operand::Immediate(i) => Some(Literal::Int(*i)),
            Operand::FloatImmediate(f) => Some(Literal::Float(*f)),
            Operand::Bool(b) => Some(Literal::Bool(*b)),
            Operand::Temp(id) => {
                // 查找该临时变量是否被赋值为常量
                for block in &func.blocks {
                    for instr in &block.instructions {
                        if let Instruction::Const(dest, val) = instr {
                            if let Operand::Temp(dest_id) = dest {
                                if dest_id == id {
                                    return Some(val.clone());
                                }
                            }
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// 计算二元运算
    fn evaluate_binary_op(&self, left: Literal, op: BinaryOp, right: Literal) -> Option<Literal> {
        match (left, right) {
            (Literal::Int(l), Literal::Int(r)) => {
                match op {
                    BinaryOp::Add => Some(Literal::Int(l + r)),
                    BinaryOp::Sub => Some(Literal::Int(l - r)),
                    BinaryOp::Mul => Some(Literal::Int(l * r)),
                    BinaryOp::Div if r != 0 => Some(Literal::Int(l / r)),
                    BinaryOp::Mod if r != 0 => Some(Literal::Int(l % r)),
                    BinaryOp::Eq => Some(Literal::Bool(l == r)),
                    BinaryOp::Ne => Some(Literal::Bool(l != r)),
                    BinaryOp::Lt => Some(Literal::Bool(l < r)),
                    BinaryOp::Le => Some(Literal::Bool(l <= r)),
                    BinaryOp::Gt => Some(Literal::Bool(l > r)),
                    BinaryOp::Ge => Some(Literal::Bool(l >= r)),
                    BinaryOp::And => Some(Literal::Bool(l != 0 && r != 0)),
                    BinaryOp::Or => Some(Literal::Bool(l != 0 || r != 0)),
                }
            }
            (Literal::Float(l), Literal::Float(r)) => {
                match op {
                    BinaryOp::Add => Some(Literal::Float(l + r)),
                    BinaryOp::Sub => Some(Literal::Float(l - r)),
                    BinaryOp::Mul => Some(Literal::Float(l * r)),
                    BinaryOp::Div if r != 0.0 => Some(Literal::Float(l / r)),
                    BinaryOp::Eq => Some(Literal::Bool((l - r).abs() < f64::EPSILON)),
                    BinaryOp::Ne => Some(Literal::Bool((l - r).abs() >= f64::EPSILON)),
                    BinaryOp::Lt => Some(Literal::Bool(l < r)),
                    BinaryOp::Le => Some(Literal::Bool(l <= r)),
                    BinaryOp::Gt => Some(Literal::Bool(l > r)),
                    BinaryOp::Ge => Some(Literal::Bool(l >= r)),
                    _ => None,
                }
            }
            (Literal::Bool(l), Literal::Bool(r)) => {
                match op {
                    BinaryOp::And => Some(Literal::Bool(l && r)),
                    BinaryOp::Or => Some(Literal::Bool(l || r)),
                    BinaryOp::Eq => Some(Literal::Bool(l == r)),
                    BinaryOp::Ne => Some(Literal::Bool(l != r)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// 死代码消除
    fn dead_code_elimination(&self, func: &mut FunctionIR) -> Result<()> {
        // 收集所有使用的变量
        let mut used_vars = HashSet::new();
        
        // 标记所有在 terminators 中使用的变量
        for block in &func.blocks {
            for instr in &block.terminators {
                self.collect_used_operands(instr, &mut used_vars);
            }
        }
        
        // 反向遍历，标记所有到达返回值的变量
        let mut changed = true;
        while changed {
            changed = false;
            for block in &func.blocks {
                for instr in &block.instructions {
                    if let Some(def) = self.get_defined_var(instr) {
                        if self.uses_any(instr, &used_vars) {
                            if used_vars.insert(def.clone()) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
        
        // 移除未使用的指令
        for block in &mut func.blocks {
            block.instructions.retain(|instr| {
                if let Some(def) = self.get_defined_var(instr) {
                    used_vars.contains(&def)
                } else {
                    true // 保留没有定义的指令（如跳转、返回等）
                }
            });
            
            // 移除 nop 指令
            block.instructions.retain(|instr| !matches!(instr, Instruction::Nop));
        }
        
        Ok(())
    }
    
    /// 收集指令中使用的所有操作数
    fn collect_used_operands(&self, instr: &Instruction, used: &mut HashSet<Operand>) {
        match instr {
            Instruction::BinOp(_, left, _, right) => {
                used.insert(left.clone());
                used.insert(right.clone());
            }
            Instruction::Copy(_, src) => {
                used.insert(src.clone());
            }
            Instruction::CondBranch(cond, _, _) => {
                used.insert(cond.clone());
            }
            Instruction::Return(Some(val)) => {
                used.insert(val.clone());
            }
            Instruction::Load(addr) => {
                used.insert(addr.clone());
            }
            Instruction::Store(addr, value) => {
                used.insert(addr.clone());
                used.insert(value.clone());
            }
            Instruction::Call(_, args) => {
                for arg in args {
                    used.insert(arg.clone());
                }
            }
            Instruction::Phi(pairs) => {
                for (_, operand) in pairs {
                    used.insert(operand.clone());
                }
            }
            _ => {}
        }
    }
    
    /// 获取指令定义的变量
    fn get_defined_var(&self, instr: &Instruction) -> Option<Operand> {
        match instr {
            Instruction::Const(dest, _) => Some(dest.clone()),
            Instruction::Copy(dest, _) => Some(dest.clone()),
            Instruction::BinOp(dest, _, _, _) => Some(dest.clone()),
            Instruction::Load(dest, _) => Some(dest.clone()),
            Instruction::Call(dest, _, _) => Some(dest.clone()),
            Instruction::Phi(dest, _) => Some(dest.clone()),
            _ => None,
        }
    }
    
    /// 检查指令是否使用了任何已标记的变量
    fn uses_any(&self, instr: &Instruction, used: &HashSet<Operand>) -> bool {
        match instr {
            Instruction::Return(Some(val)) => used.contains(val),
            Instruction::CondBranch(cond, _, _) => used.contains(cond),
            Instruction::Store(_, value) => used.contains(value),
            Instruction::Call(_, args) => args.iter().any(|arg| used.contains(arg)),
            Instruction::BinOp(_, left, _, right) => used.contains(left) || used.contains(right),
            Instruction::Copy(_, src) => used.contains(src),
            Instruction::Load(_, addr) => used.contains(addr),
            Instruction::Phi(_, pairs) => pairs.iter().any(|(_, op)| used.contains(op)),
            _ => false,
        }
    }

    /// 公共子表达式消除
    fn common_subexpression_elimination(&self, func: &mut FunctionIR) -> Result<()> {
        // 使用哈希表记录已计算的表达式
        let mut expr_map: HashMap<String, Operand> = HashMap::new();
        
        for block in &mut func.blocks {
            let mut new_instructions = Vec::new();
            
            for instr in &block.instructions {
                match instr {
                    Instruction::BinOp(dest, left, op, right) => {
                        // 创建表达式的唯一键
                        let expr_key = format!("{:?}:{:?}:{:?}", left, op, right);
                        
                        if let Some(existing) = expr_map.get(&expr_key) {
                            // 如果表达式已计算过，使用 Copy 代替
                            new_instructions.push(Instruction::Copy(dest.clone(), existing.clone()));
                        } else {
                            // 记录新表达式
                            expr_map.insert(expr_key, dest.clone());
                            new_instructions.push(instr.clone());
                        }
                    }
                    _ => {
                        new_instructions.push(instr.clone());
                    }
                }
            }
            
            block.instructions = new_instructions;
        }
        
        Ok(())
    }

    /// 循环不变量外提
    fn loop_invariant_code_motion(&self, func: &mut FunctionIR) -> Result<()> {
        // 简化实现：识别简单的循环结构并外提不变量
        // 实际实现需要构建 CFG 和分析支配关系
        
        for block_idx in 0..func.blocks.len() {
            // 检查是否有向后跳转（循环）
            let block = &func.blocks[block_idx];
            let mut loop_header = None;
            
            for term in &block.terminators {
                if let Instruction::Jump(label) = term {
                    // 检查是否跳转到前面的块
                    for (idx, b) in func.blocks.iter().enumerate() {
                        if idx <= block_idx && b.label.0 == label.0 {
                            loop_header = Some(idx);
                            break;
                        }
                    }
                }
            }
            
            if let Some(header_idx) = loop_header {
                // 找到循环体中的块
                let loop_blocks: Vec<usize> = (header_idx..=block_idx).collect();
                
                // 收集循环外定义的值
                let mut outside_defs = HashSet::new();
                for (idx, b) in func.blocks.iter().enumerate() {
                    if !loop_blocks.contains(&idx) {
                        for instr in &b.instructions {
                            if let Some(def) = self.get_defined_var(instr) {
                                outside_defs.insert(def);
                            }
                        }
                    }
                }
                
                // 在循环块中寻找不变量
                for &loop_idx in &loop_blocks {
                    let mut to_move = Vec::new();
                    
                    for (i, instr) in func.blocks[loop_idx].instructions.iter().enumerate() {
                        if self.is_loop_invariant(instr, &outside_defs, &loop_blocks, &func.blocks) {
                            to_move.push((i, instr.clone()));
                        }
                    }
                    
                    // 将不变量移动到循环头之前
                    if !to_move.is_empty() {
                        // 这里简化处理，实际需要将指令移动到 pre-header
                        for (_, instr) in to_move {
                            if let Some(def) = self.get_defined_var(&instr) {
                                outside_defs.insert(def);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 检查指令是否是循环不变量
    fn is_loop_invariant(
        &self,
        instr: &Instruction,
        outside_defs: &HashSet<Operand>,
        loop_blocks: &[usize],
        blocks: &[BasicBlock],
    ) -> bool {
        // 只有纯计算指令可以外提
        match instr {
            Instruction::BinOp(_, left, _, right) => {
                self.all_operants_invariant(left, right, outside_defs, loop_blocks, blocks)
            }
            Instruction::Load(_, addr) => {
                // Load 只有在地址不变且没有 store 到该地址时才能外提
                self.operand_is_invariant(addr, outside_defs, loop_blocks, blocks)
            }
            _ => false,
        }
    }
    
    /// 检查操作数是否是循环不变量
    fn all_operants_invariant(
        &self,
        left: &Operand,
        right: &Operand,
        outside_defs: &HashSet<Operand>,
        loop_blocks: &[usize],
        blocks: &[BasicBlock],
    ) -> bool {
        self.operand_is_invariant(left, outside_defs, loop_blocks, blocks)
            && self.operand_is_invariant(right, outside_defs, loop_blocks, blocks)
    }
    
    /// 检查单个操作数是否是循环不变量
    fn operand_is_invariant(
        &self,
        operand: &Operand,
        outside_defs: &HashSet<Operand>,
        loop_blocks: &[usize],
        blocks: &[BasicBlock],
    ) -> bool {
        match operand {
            Operand::Immediate(_) | Operand::FloatImmediate(_) | Operand::Bool(_) => true,
            Operand::Temp(id) => {
                // 检查该临时变量是否在循环外定义
                let temp_operand = Operand::Temp(*id);
                if outside_defs.contains(&temp_operand) {
                    return true;
                }
                
                // 检查是否在循环内定义
                for &idx in loop_blocks {
                    for instr in &blocks[idx].instructions {
                        if let Some(def) = self.get_defined_var(instr) {
                            if def == temp_operand {
                                return false; // 在循环内定义，不是不变量
                            }
                        }
                    }
                }
                
                true // 未在循环内定义，可能是不变量或未定义
            }
            Operand::Variable(_) => true, // 全局变量视为不变量（简化处理）
            _ => false,
        }
    }

    /// 内联优化
    fn inline_optimization(&self, module_ir: &mut ModuleIR) -> Result<()> {
        // 收集适合内联的小函数
        let mut inline_candidates = Vec::new();
        
        for (idx, func) in module_ir.functions.iter().enumerate() {
            // 只内联小函数（少于 10 条指令）
            let total_instructions: usize = func.blocks.iter()
                .map(|b| b.instructions.len() + b.terminators.len())
                .sum();
            
            if total_instructions < 10 && !func.name.starts_with("main") {
                inline_candidates.push((idx, func.name.clone()));
            }
        }
        
        // 对每个调用点进行内联
        for (func_idx, _) in inline_candidates.clone() {
            let func_name = module_ir.functions[func_idx].name.clone();
            
            for caller in &mut module_ir.functions {
                if caller.name == func_name {
                    continue; // 不内联自己
                }
                
                self.inline_function(caller, &func_name, &module_ir.functions[func_idx])?;
            }
        }
        
        Ok(())
    }
    
    /// 内联单个函数调用
    fn inline_function(
        &self,
        caller: &mut FunctionIR,
        callee_name: &str,
        callee: &FunctionIR,
    ) -> Result<()> {
        // 查找所有调用点
        for block in &mut caller.blocks {
            let mut new_instructions = Vec::new();
            let mut replaced_call = false;
            
            for instr in &block.instructions {
                if let Instruction::Call(dest, func_name, args) = instr {
                    if func_name == callee_name && callee.blocks.len() > 0 {
                        // 简化的内联：直接插入 callee 的指令
                        // 实际需要处理参数传递、返回值、标签重命名等
                        
                        // 复制 callee 的指令并重命名标签
                        for callee_block in &callee.blocks {
                            for callee_instr in &callee_block.instructions {
                                new_instructions.push(self.rename_callee_instruction(
                                    callee_instr,
                                    dest,
                                    args,
                                    &callee.params,
                                ));
                            }
                        }
                        
                        replaced_call = true;
                        continue;
                    }
                }
                new_instructions.push(instr.clone());
            }
            
            if replaced_call {
                block.instructions = new_instructions;
            }
        }
        
        Ok(())
    }
    
    /// 重命名被调用函数的指令
    fn rename_callee_instruction(
        &self,
        instr: &Instruction,
        return_dest: &Option<Operand>,
        args: &[Operand],
        params: &[String],
    ) -> Instruction {
        // 简化处理：直接克隆，实际需要更复杂的重命名逻辑
        instr.clone()
    }
}

/// 优化入口函数
pub fn optimize(module_ir: ModuleIR, config: &CompilerConfig) -> Result<ModuleIR> {
    let optimizer = Optimizer::new(config.clone());
    optimizer.optimize(module_ir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding() {
        let config = CompilerConfig::default();
        let optimizer = Optimizer::new(config);
        
        let left = Literal::Int(2);
        let right = Literal::Int(3);
        
        let result = optimizer.evaluate_binary_op(left, BinaryOp::Add, right);
        assert_eq!(result, Some(Literal::Int(5)));
    }
    
    #[test]
    fn test_boolean_operations() {
        let config = CompilerConfig::default();
        let optimizer = Optimizer::new(config);
        
        // 测试 AND
        let result = optimizer.evaluate_binary_op(
            Literal::Bool(true),
            BinaryOp::And,
            Literal::Bool(false),
        );
        assert_eq!(result, Some(Literal::Bool(false)));
        
        // 测试 OR
        let result = optimizer.evaluate_binary_op(
            Literal::Bool(true),
            BinaryOp::Or,
            Literal::Bool(false),
        );
        assert_eq!(result, Some(Literal::Bool(true)));
    }
    
    #[test]
    fn test_integer_comparison() {
        let config = CompilerConfig::default();
        let optimizer = Optimizer::new(config);
        
        let result = optimizer.evaluate_binary_op(
            Literal::Int(5),
            BinaryOp::Gt,
            Literal::Int(3),
        );
        assert_eq!(result, Some(Literal::Bool(true)));
    }
}
