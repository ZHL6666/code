//! Aether 代码生成模块
//! 
//! 将 IR 转换为目标代码（LLVM IR 或字节码）

use crate::ir::*;
use crate::config::CompilerConfig;
use crate::error::{Error, Result};

/// 代码生成器
pub struct CodeGenerator {
    config: CompilerConfig,
}

impl CodeGenerator {
    /// 创建新的代码生成器
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// 生成代码
    pub fn generate(&self, module_ir: ModuleIR) -> Result<Vec<u8>> {
        match self.config.output_format {
            OutputFormat::Bytecode => self.generate_bytecode(module_ir),
            OutputFormat::LlvmIr => self.generate_llvm_ir(module_ir),
            OutputFormat::Assembly => self.generate_assembly(module_ir),
            _ => Err(Error::codegen(
                "Unsupported output format".to_string(),
            )),
        }
    }

    /// 生成字节码
    fn generate_bytecode(&self, module_ir: ModuleIR) -> Result<Vec<u8>> {
        let mut bytecode = Vec::new();

        // 魔数
        bytecode.extend_from_slice(b"AETH");
        
        // 版本号
        bytecode.push(0);
        bytecode.push(1);
        bytecode.push(0);
        bytecode.push(0);

        // 函数数量
        bytecode.extend_from_slice(&(module_ir.functions.len() as u32).to_le_bytes());

        for func in &module_ir.functions {
            self.encode_function(func, &mut bytecode)?;
        }

        Ok(bytecode)
    }

    /// 编码函数
    fn encode_function(&self, func: &FunctionIR, bytecode: &mut Vec<u8>) -> Result<()> {
        // 函数名长度和名称
        let name_bytes = func.name.as_bytes();
        bytecode.push(name_bytes.len() as u8);
        bytecode.extend_from_slice(name_bytes);

        // 参数数量
        bytecode.push(func.params.len() as u8);
        for param in &func.params {
            let param_bytes = param.as_bytes();
            bytecode.push(param_bytes.len() as u8);
            bytecode.extend_from_slice(param_bytes);
        }

        // 返回值类型
        match &func.return_type {
            Some(_) => bytecode.push(1),
            None => bytecode.push(0),
        }

        // 基本块数量
        bytecode.extend_from_slice(&(func.blocks.len() as u32).to_le_bytes());

        for block in &func.blocks {
            self.encode_block(block, bytecode)?;
        }

        Ok(())
    }

    /// 编码基本块
    fn encode_block(&self, block: &BasicBlock, bytecode: &mut Vec<u8>) -> Result<()> {
        // 块标签
        let label_bytes = block.label.0.as_bytes();
        bytecode.extend_from_slice(&(label_bytes.len() as u16).to_le_bytes());
        bytecode.extend_from_slice(label_bytes);

        // 指令数量
        let instr_count = block.instructions.len() + block.terminators.len();
        bytecode.extend_from_slice(&(instr_count as u32).to_le_bytes());

        // 编码指令
        for instr in &block.instructions {
            self.encode_instruction(instr, bytecode)?;
        }
        for instr in &block.terminators {
            self.encode_instruction(instr, bytecode)?;
        }

        Ok(())
    }

    /// 编码指令
    fn encode_instruction(&self, instr: &Instruction, bytecode: &mut Vec<u8>) -> Result<()> {
        match instr {
            Instruction::Const(dest, literal) => {
                bytecode.push(0x01); // CONST opcode
                self.encode_operand(dest, bytecode)?;
                self.encode_literal(literal, bytecode)?;
            }
            Instruction::Copy(dest, src) => {
                bytecode.push(0x02); // COPY opcode
                self.encode_operand(dest, bytecode)?;
                self.encode_operand(src, bytecode)?;
            }
            Instruction::BinOp(dest, left, op, right) => {
                bytecode.push(0x03); // BINOP opcode
                self.encode_operand(dest, bytecode)?;
                self.encode_operand(left, bytecode)?;
                bytecode.push(self.encode_binary_op(op));
                self.encode_operand(right, bytecode)?;
            }
            Instruction::Jump(label) => {
                bytecode.push(0x04); // JUMP opcode
                let label_bytes = label.0.as_bytes();
                bytecode.extend_from_slice(&(label_bytes.len() as u16).to_le_bytes());
                bytecode.extend_from_slice(label_bytes);
            }
            Instruction::CondBranch(cond, label1, label2) => {
                bytecode.push(0x05); // CBRANCH opcode
                self.encode_operand(cond, bytecode)?;
                let label1_bytes = label1.0.as_bytes();
                bytecode.extend_from_slice(&(label1_bytes.len() as u16).to_le_bytes());
                bytecode.extend_from_slice(label1_bytes);
                let label2_bytes = label2.0.as_bytes();
                bytecode.extend_from_slice(&(label2_bytes.len() as u16).to_le_bytes());
                bytecode.extend_from_slice(label2_bytes);
            }
            Instruction::Return(value) => {
                bytecode.push(0x06); // RETURN opcode
                match value {
                    Some(v) => {
                        bytecode.push(1);
                        self.encode_operand(v, bytecode)?;
                    }
                    None => bytecode.push(0),
                }
            }
            Instruction::Label(label) => {
                bytecode.push(0x07); // LABEL opcode
                let label_bytes = label.0.as_bytes();
                bytecode.extend_from_slice(&(label_bytes.len() as u16).to_le_bytes());
                bytecode.extend_from_slice(label_bytes);
            }
            _ => {
                bytecode.push(0x00); // NOP for unimplemented instructions
            }
        }
        Ok(())
    }

    /// 编码操作数
    fn encode_operand(&self, operand: &Operand, bytecode: &mut Vec<u8>) -> Result<()> {
        match operand {
            Operand::Temp(id) => {
                bytecode.push(0x01);
                bytecode.extend_from_slice(&(*id as u32).to_le_bytes());
            }
            Operand::Variable(name) => {
                bytecode.push(0x02);
                let name_bytes = name.as_bytes();
                bytecode.push(name_bytes.len() as u8);
                bytecode.extend_from_slice(name_bytes);
            }
            Operand::Immediate(val) => {
                bytecode.push(0x03);
                bytecode.extend_from_slice(&val.to_le_bytes());
            }
            Operand::FloatImmediate(val) => {
                bytecode.push(0x04);
                bytecode.extend_from_slice(&val.to_le_bytes());
            }
            Operand::Bool(val) => {
                bytecode.push(0x05);
                bytecode.push(if *val { 1 } else { 0 });
            }
            _ => {
                bytecode.push(0x00);
            }
        }
        Ok(())
    }

    /// 编码字面量
    fn encode_literal(&self, literal: &Literal, bytecode: &mut Vec<u8>) -> Result<()> {
        match literal {
            Literal::Int(val) => {
                bytecode.push(0x01);
                bytecode.extend_from_slice(&val.to_le_bytes());
            }
            Literal::Float(val) => {
                bytecode.push(0x02);
                bytecode.extend_from_slice(&val.to_le_bytes());
            }
            Literal::Bool(val) => {
                bytecode.push(0x03);
                bytecode.push(if *val { 1 } else { 0 });
            }
            Literal::Str(s) => {
                bytecode.push(0x04);
                let bytes = s.as_bytes();
                bytecode.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                bytecode.extend_from_slice(bytes);
            }
            Literal::None => {
                bytecode.push(0x05);
            }
            Literal::Unit => {
                bytecode.push(0x06);
            }
            Literal::Char(c) => {
                bytecode.push(0x07);
                bytecode.push(*c as u8);
            }
        }
        Ok(())
    }

    /// 编码二元运算符
    fn encode_binary_op(&self, op: &BinaryOp) -> u8 {
        match op {
            BinaryOp::Add => 0x01,
            BinaryOp::Sub => 0x02,
            BinaryOp::Mul => 0x03,
            BinaryOp::Div => 0x04,
            BinaryOp::Mod => 0x05,
            BinaryOp::Eq => 0x06,
            BinaryOp::Ne => 0x07,
            BinaryOp::Lt => 0x08,
            BinaryOp::Le => 0x09,
            BinaryOp::Gt => 0x0A,
            BinaryOp::Ge => 0x0B,
            BinaryOp::And => 0x0C,
            BinaryOp::Or => 0x0D,
        }
    }

    /// 生成 LLVM IR
    fn generate_llvm_ir(&self, module_ir: ModuleIR) -> Result<Vec<u8>> {
        let mut ir_text = String::new();
        
        ir_text.push_str("; Aether LLVM IR\n");
        ir_text.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");

        for func in &module_ir.functions {
            self.generate_llvm_function(func, &mut ir_text);
        }

        Ok(ir_text.into_bytes())
    }

    /// 生成 LLVM 函数
    fn generate_llvm_function(&self, func: &FunctionIR, ir_text: &mut String) {
        let return_type = match &func.return_type {
            Some(_) => "i64",
            None => "void",
        };

        let params: Vec<String> = func.params.iter().map(|_| "i64".to_string()).collect();
        let param_str = params.join(", ");

        ir_text.push_str(&format!(
            "define {} @{}({}) {{\n",
            return_type, func.name, param_str
        ));

        // 入口块
        ir_text.push_str("entry:\n");

        for block in &func.blocks {
            if block.label.0 != "entry" {
                ir_text.push_str(&format!("{}:\n", block.label.0));
            }

            for instr in &block.instructions {
                self.generate_llvm_instruction(instr, ir_text);
            }

            for instr in &block.terminators {
                self.generate_llvm_instruction(instr, ir_text);
            }
        }

        ir_text.push_str("}\n\n");
    }

    /// 生成 LLVM 指令
    fn generate_llvm_instruction(&self, instr: &Instruction, ir_text: &mut String) {
        match instr {
            Instruction::Const(dest, literal) => {
                let llvm_val = self.literal_to_llvm(literal);
                ir_text.push_str(&format!(
                    "  %{} = add i64 {}, 0\n",
                    self.operand_to_name(dest),
                    llvm_val
                ));
            }
            Instruction::BinOp(dest, left, op, right) => {
                let llvm_op = self.binary_op_to_llvm(op);
                ir_text.push_str(&format!(
                    "  %{} = {} i64 {}, {}\n",
                    self.operand_to_name(dest),
                    llvm_op,
                    self.operand_to_name(left),
                    self.operand_to_name(right)
                ));
            }
            Instruction::Return(value) => {
                match value {
                    Some(v) => {
                        ir_text.push_str(&format!(
                            "  ret i64 {}\n",
                            self.operand_to_name(v)
                        ));
                    }
                    None => {
                        ir_text.push_str("  ret void\n");
                    }
                }
            }
            Instruction::Jump(label) => {
                ir_text.push_str(&format!("  br label %{}\n", label.0));
            }
            Instruction::CondBranch(cond, label1, label2) => {
                ir_text.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    self.operand_to_name(cond),
                    label1.0,
                    label2.0
                ));
            }
            _ => {}
        }
    }

    fn literal_to_llvm(&self, literal: &Literal) -> String {
        match literal {
            Literal::Int(n) => n.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::Bool(b) => if *b { "1" } else { "0" }.to_string(),
            _ => "0".to_string(),
        }
    }

    fn binary_op_to_llvm(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "add",
            BinaryOp::Sub => "sub",
            BinaryOp::Mul => "mul",
            BinaryOp::Div => "sdiv",
            BinaryOp::Mod => "srem",
            BinaryOp::Eq => "icmp eq",
            BinaryOp::Ne => "icmp ne",
            BinaryOp::Lt => "icmp slt",
            BinaryOp::Le => "icmp sle",
            BinaryOp::Gt => "icmp sgt",
            BinaryOp::Ge => "icmp sge",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
        }
    }

    fn operand_to_name(&self, operand: &Operand) -> String {
        match operand {
            Operand::Temp(id) => format!("{}", id),
            Operand::Variable(name) => format!("%{}", name),
            Operand::Immediate(n) => n.to_string(),
            _ => "0".to_string(),
        }
    }

    /// 生成汇编代码
    fn generate_assembly(&self, _module_ir: ModuleIR) -> Result<Vec<u8>> {
        Err(Error::codegen(
            "Assembly generation not yet implemented".to_string(),
        ))
    }
}

/// 代码生成入口函数
pub fn generate(module_ir: ModuleIR, config: &CompilerConfig) -> Result<Vec<u8>> {
    let generator = CodeGenerator::new(config.clone());
    generator.generate(module_ir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytecode_generation() {
        let config = CompilerConfig::default();
        let generator = CodeGenerator::new(config);
        
        let mut module_ir = ModuleIR::new();
        let mut func = FunctionIR::new("test".to_string(), vec![], None);
        
        let entry = BasicBlock::new(Label::new("entry"));
        func.blocks.push(entry);
        
        module_ir.functions.push(func);
        
        let bytecode = generator.generate(module_ir).unwrap();
        assert!(bytecode.starts_with(b"AETH"));
    }
}
