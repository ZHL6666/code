//! Aether 中间表示 (IR) 模块
//! 
//! 定义三地址码形式的中间表示

use crate::ast::{Type, BinaryOp, UnaryOp};

/// IR 指令
#[derive(Debug, Clone)]
pub enum Instruction {
    /// 常量赋值：dest = const value
    Const(Operand, Literal),
    
    /// 变量赋值：dest = src
    Copy(Operand, Operand),
    
    /// 二元运算：dest = left op right
    BinOp(Operand, Operand, BinaryOp, Operand),
    
    /// 一元运算：dest = op src
    UnaryOp(Operand, UnaryOp, Operand),
    
    /// 函数调用：dest = call func(args)
    Call(Operand, String, Vec<Operand>),
    
    /// 方法调用：dest = call obj.method(args)
    MethodCall(Operand, Operand, String, Vec<Operand>),
    
    /// 跳转：goto label
    Jump(Label),
    
    /// 条件跳转：if cond goto label
    Branch(Operand, Label),
    
    /// 条件跳转：if cond goto label1 else goto label2
    CondBranch(Operand, Label, Label),
    
    /// 返回：return value
    Return(Option<Operand>),
    
    /// 标签：label:
    Label(Label),
    
    /// Phi 函数（用于 SSA）：dest = phi [(label1, val1), (label2, val2), ...]
    Phi(Operand, Vec<(Label, Operand)>),
    
    /// 加载字段：dest = src.field
    LoadField(Operand, Operand, String),
    
    /// 存储字段：dest.field = src
    StoreField(Operand, String, Operand),
    
    /// 数组索引：dest = arr[index]
    LoadIndex(Operand, Operand, Operand),
    
    /// 数组存储：arr[index] = src
    StoreIndex(Operand, Operand, Operand),
    
    /// 分配内存：dest = alloc size
    Alloc(Operand, usize),
    
    /// 释放内存：free ptr
    Free(Operand),
    
    /// 类型转换：dest = convert src as type
    Convert(Operand, Operand, Type),
    
    /// 无操作
    Nop,
}

/// 操作数
#[derive(Debug, Clone)]
pub enum Operand {
    /// 临时变量：%0, %1, ...
    Temp(usize),
    
    /// 命名变量：x, y, ...
    Variable(String),
    
    /// 寄存器（用于代码生成）
    Register(String),
    
    /// 立即数
    Immediate(i64),
    
    /// 浮点立即数
    FloatImmediate(f64),
    
    /// 字符串字面量
    StringLiteral(String),
    
    /// 布尔值
    Bool(bool),
    
    /// 空值
    None,
    
    /// 单元值
    Unit,
}

impl Operand {
    /// 创建新的临时变量
    pub fn temp(id: usize) -> Self {
        Operand::Temp(id)
    }
    
    /// 创建命名变量
    pub fn var(name: &str) -> Self {
        Operand::Variable(name.to_string())
    }
}

/// 标签
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Label(pub String);

impl Label {
    pub fn new(name: &str) -> Self {
        Label(name.to_string())
    }
    
    pub fn unique(prefix: &str, id: usize) -> Self {
        Label(format!("{}_{}", prefix, id))
    }
}

/// 字面量
#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(String),
    None,
    Unit,
}

/// 基本块
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub label: Label,
    pub instructions: Vec<Instruction>,
    pub terminators: Vec<Instruction>,
}

impl BasicBlock {
    pub fn new(label: Label) -> Self {
        Self {
            label,
            instructions: Vec::new(),
            terminators: Vec::new(),
        }
    }
    
    pub fn add_instruction(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }
    
    pub fn add_terminator(&mut self, instr: Instruction) {
        self.terminators.push(instr);
    }
}

/// 函数 IR
#[derive(Debug, Clone)]
pub struct FunctionIR {
    pub name: String,
    pub params: Vec<String>,
    pub return_type: Option<Type>,
    pub blocks: Vec<BasicBlock>,
    pub temp_count: usize,
}

impl FunctionIR {
    pub fn new(name: String, params: Vec<String>, return_type: Option<Type>) -> Self {
        Self {
            name,
            params,
            return_type,
            blocks: Vec::new(),
            temp_count: 0,
        }
    }
    
    /// 生成新的临时变量 ID
    pub fn new_temp(&mut self) -> usize {
        let id = self.temp_count;
        self.temp_count += 1;
        id
    }
}

/// 模块 IR
#[derive(Debug, Clone)]
pub struct ModuleIR {
    pub functions: Vec<FunctionIR>,
    pub globals: Vec<GlobalVar>,
}

impl ModuleIR {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            globals: Vec::new(),
        }
    }
}

impl Default for ModuleIR {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局变量
#[derive(Debug, Clone)]
pub struct GlobalVar {
    pub name: String,
    pub var_type: Type,
    pub initial_value: Option<Literal>,
}

/// IR 生成器上下文
pub struct IRContext {
    pub current_function: Option<FunctionIR>,
    pub current_block: Option<Label>,
    pub temp_counter: usize,
}

impl IRContext {
    pub fn new() -> Self {
        Self {
            current_function: None,
            current_block: None,
            temp_counter: 0,
        }
    }
    
    pub fn new_temp(&mut self) -> usize {
        let id = self.temp_counter;
        self.temp_counter += 1;
        id
    }
}

impl Default for IRContext {
    fn default() -> Self {
        Self::new()
    }
}

/// 从 AST 生成 IR 的入口函数
pub fn generate(ast: crate::ast::Module) -> crate::error::Result<ModuleIR> {
    let mut module_ir = ModuleIR::new();
    let mut context = IRContext::new();
    
    // 先生成全局变量
    for item in &ast.items {
        if let crate::ast::Item::Global(var_decl) = item {
            let global = GlobalVar {
                name: var_decl.name.clone(),
                var_type: var_decl.var_type.clone(),
                initial_value: var_decl.initial_value.as_ref().map(|lit| match lit {
                    crate::ast::Literal::Int(v) => Literal::Int(*v),
                    crate::ast::Literal::Float(v) => Literal::Float(*v),
                    crate::ast::Literal::Bool(v) => Literal::Bool(*v),
                    crate::ast::Literal::Str(v) => Literal::Str(v.clone()),
                    crate::ast::Literal::Char(v) => Literal::Char(*v),
                    crate::ast::Literal::None => Literal::None,
                }),
            };
            module_ir.globals.push(global);
        }
    }
    
    // 生成函数
    for item in ast.items {
        match item {
            crate::ast::Item::Function(func) => {
                let func_ir = generate_function(func, &mut context)?;
                module_ir.functions.push(func_ir);
            }
            // 其他项暂不处理
            _ => {}
        }
    }
    
    Ok(module_ir)
}

/// 生成函数 IR
fn generate_function(func: crate::ast::Function, context: &mut IRContext) -> crate::error::Result<FunctionIR> {
    let mut func_ir = FunctionIR::new(
        func.name.clone(),
        func.params.iter().map(|p| p.name.clone()).collect(),
        func.return_type.clone(),
    );
    
    // 创建入口基本块
    let entry_block = BasicBlock::new(Label::new("entry"));
    func_ir.blocks.push(entry_block);
    
    // 将参数复制到局部变量
    if let Some(block) = func_ir.blocks.last_mut() {
        for (i, param) in func.params.iter().enumerate() {
            block.add_instruction(Instruction::Copy(
                Operand::Variable(param.name.clone()),
                Operand::Temp(i),
            ));
        }
    }
    
    // 生成函数体 IR
    if let Some(body) = func.body {
        generate_block(&body, &mut func_ir, context)?;
    } else {
        // 空函数返回 unit
        if let Some(block) = func_ir.blocks.last_mut() {
            block.add_terminator(Instruction::Return(None));
        }
    }
    
    Ok(func_ir)
}

/// 生成语句块的 IR
fn generate_block(
    block: &crate::ast::Block,
    func_ir: &mut FunctionIR,
    context: &mut IRContext,
) -> crate::error::Result<Option<Operand>> {
    let mut last_value = None;
    
    for stmt in &block.statements {
        last_value = generate_statement(stmt, func_ir, context)?;
    }
    
    // 返回最后一个表达式的值（如果有）
    if let Some(expr) = &block.final_expr {
        last_value = generate_expression(expr, func_ir, context)?;
    }
    
    Ok(last_value)
}

/// 生成语句的 IR
fn generate_statement(
    stmt: &crate::ast::Statement,
    func_ir: &mut FunctionIR,
    context: &mut IRContext,
) -> crate::error::Result<Option<Operand>> {
    match stmt {
        crate::ast::Statement::Let(let_decl) => {
            let value = if let Some(expr) = &let_decl.initializer {
                generate_expression(expr, func_ir, context)?
            } else {
                Some(Operand::Unit)
            };
            
            if let Some(val_operand) = value {
                let current_block = func_ir.blocks.last_mut().unwrap();
                current_block.add_instruction(Instruction::Copy(
                    Operand::Variable(let_decl.name.clone()),
                    val_operand,
                ));
            }
            Ok(None)
        }
        crate::ast::Statement::Expr(expr) => {
            generate_expression(expr, func_ir, context)
        }
        crate::ast::Statement::Item(item) => {
            // 局部 item（如嵌套函数），暂时跳过
            Ok(None)
        }
    }
}

/// 生成表达式的 IR
fn generate_expression(
    expr: &crate::ast::Expression,
    func_ir: &mut FunctionIR,
    context: &mut IRContext,
) -> crate::error::Result<Option<Operand>> {
    let current_block = func_ir.blocks.last_mut().unwrap();
    
    match expr {
        crate::ast::Expression::Literal(lit) => {
            let temp_id = func_ir.new_temp();
            let literal = match lit {
                crate::ast::Literal::Int(v) => Literal::Int(*v),
                crate::ast::Literal::Float(v) => Literal::Float(*v),
                crate::ast::Literal::Bool(v) => Literal::Bool(*v),
                crate::ast::Literal::Str(v) => Literal::Str(v.clone()),
                crate::ast::Literal::Char(v) => Literal::Char(*v),
                crate::ast::Literal::None => Literal::None,
            };
            current_block.add_instruction(Instruction::Const(
                Operand::Temp(temp_id),
                literal,
            ));
            Ok(Some(Operand::Temp(temp_id)))
        }
        
        crate::ast::Expression::Identifier(name) => {
            Ok(Some(Operand::Variable(name.clone())))
        }
        
        crate::ast::Expression::Binary(left, op, right) => {
            let left_op = generate_expression(left, func_ir, context)?;
            let right_op = generate_expression(right, func_ir, context)?;
            
            if let (Some(left_val), Some(right_val)) = (left_op, right_op) {
                let temp_id = func_ir.new_temp();
                current_block.add_instruction(Instruction::BinOp(
                    Operand::Temp(temp_id),
                    left_val,
                    op.clone(),
                    right_val,
                ));
                Ok(Some(Operand::Temp(temp_id)))
            } else {
                Ok(None)
            }
        }
        
        crate::ast::Expression::Unary(op, operand) => {
            let operand_op = generate_expression(operand, func_ir, context)?;
            if let Some(val) = operand_op {
                let temp_id = func_ir.new_temp();
                current_block.add_instruction(Instruction::UnaryOp(
                    Operand::Temp(temp_id),
                    op.clone(),
                    val,
                ));
                Ok(Some(Operand::Temp(temp_id)))
            } else {
                Ok(None)
            }
        }
        
        crate::ast::Expression::Call(func_name, args) => {
            let mut arg_operands = Vec::new();
            for arg in args {
                if let Some(arg_op) = generate_expression(arg, func_ir, context)? {
                    arg_operands.push(arg_op);
                }
            }
            
            let temp_id = func_ir.new_temp();
            current_block.add_instruction(Instruction::Call(
                Operand::Temp(temp_id),
                func_name.clone(),
                arg_operands,
            ));
            Ok(Some(Operand::Temp(temp_id)))
        }
        
        crate::ast::Expression::If(cond, then_block, else_block) => {
            let cond_op = generate_expression(cond, func_ir, context)?;
            
            let then_label = Label::unique("then", func_ir.new_temp());
            let else_label = Label::unique("else", func_ir.new_temp());
            let end_label = Label::unique("if_end", func_ir.new_temp());
            
            // 条件跳转
            if let Some(cond_val) = cond_op {
                current_block.add_terminator(Instruction::CondBranch(
                    cond_val,
                    then_label.clone(),
                    else_label.clone(),
                ));
            }
            
            // then 分支
            let mut then_func_ir = FunctionIR::new("".to_string(), vec![], None);
            then_func_ir.blocks.push(BasicBlock::new(then_label.clone()));
            let then_value = generate_block(then_block, &mut then_func_ir, context)?;
            
            // else 分支
            let mut else_func_ir = FunctionIR::new("".to_string(), vec![], None);
            else_func_ir.blocks.push(BasicBlock::new(else_label.clone()));
            let else_value = if let Some(else_blk) = else_block {
                generate_block(else_blk, &mut else_func_ir, context)?
            } else {
                Some(Operand::Unit)
            };
            
            // 合并分支到主函数
            func_ir.blocks.extend(then_func_ir.blocks);
            func_ir.blocks.extend(else_func_ir.blocks);
            
            // 添加结束块
            let mut end_block = BasicBlock::new(end_label.clone());
            if let Some(then_val) = then_value {
                end_block.add_instruction(Instruction::Copy(
                    Operand::Temp(func_ir.new_temp()),
                    then_val,
                ));
            }
            if let Some(else_val) = else_value {
                end_block.add_instruction(Instruction::Copy(
                    Operand::Temp(func_ir.new_temp()),
                    else_val,
                ));
            }
            func_ir.blocks.push(end_block);
            
            Ok(None)
        }
        
        crate::ast::Expression::Return(expr) => {
            let return_val = if let Some(e) = expr {
                generate_expression(e, func_ir, context)?
            } else {
                None
            };
            
            current_block.add_terminator(Instruction::Return(return_val));
            Ok(None)
        }
        
        // 其他表达式类型暂时返回 None
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operand_display() {
        let temp = Operand::temp(0);
        assert!(matches!(temp, Operand::Temp(0)));
        
        let var = Operand::var("x");
        assert!(matches!(var, Operand::Variable(name) if name == "x"));
    }

    #[test]
    fn test_label_creation() {
        let label = Label::new("loop");
        assert_eq!(label.0, "loop");
        
        let unique = Label::unique("block", 42);
        assert_eq!(unique.0, "block_42");
    }
}
