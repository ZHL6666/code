//! Aether 优化器模块
//! 
//! 对 IR 进行各种优化

use crate::ir::*;
use crate::config::CompilerConfig;
use crate::error::Result;

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
                        }
                    }
                }
                i += 1;
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
                    _ => None,
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
            _ => None,
        }
    }

    /// 死代码消除
    fn dead_code_elimination(&self, func: &mut FunctionIR) -> Result<()> {
        // 简化实现：移除未使用的临时变量和不可达代码
        for block in &mut func.blocks {
            // 移除 nop 指令
            block.instructions.retain(|instr| !matches!(instr, Instruction::Nop));
        }
        Ok(())
    }

    /// 公共子表达式消除
    fn common_subexpression_elimination(&self, func: &mut FunctionIR) -> Result<()> {
        // TODO: 实现 CSE
        Ok(())
    }

    /// 循环不变量外提
    fn loop_invariant_code_motion(&self, func: &mut FunctionIR) -> Result<()> {
        // TODO: 实现 LICM
        Ok(())
    }

    /// 内联优化
    fn inline_optimization(&self, module_ir: &mut ModuleIR) -> Result<()> {
        // TODO: 实现函数内联
        Ok(())
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
}
