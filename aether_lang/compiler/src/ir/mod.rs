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
    // TODO: 实现完整的 IR 生成
    // 这里提供一个简单的框架
    
    let mut module_ir = ModuleIR::new();
    
    for item in ast.items {
        match item {
            crate::ast::Item::Function(func) => {
                let func_ir = generate_function(func)?;
                module_ir.functions.push(func_ir);
            }
            // 其他项暂不处理
            _ => {}
        }
    }
    
    Ok(module_ir)
}

/// 生成函数 IR
fn generate_function(func: crate::ast::Function) -> crate::error::Result<FunctionIR> {
    let mut func_ir = FunctionIR::new(
        func.name.clone(),
        func.params.iter().map(|p| p.name.clone()).collect(),
        func.return_type.clone(),
    );
    
    // 创建入口基本块
    let entry_block = BasicBlock::new(Label::new("entry"));
    func_ir.blocks.push(entry_block);
    
    // TODO: 实现完整的函数体 IR 生成
    
    Ok(func_ir)
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
