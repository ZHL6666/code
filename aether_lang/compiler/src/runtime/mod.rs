//! Aether 运行时库模块
//! 
//! 提供运行时支持功能

use std::collections::HashMap;
use std::sync::Arc;

/// 运行时虚拟机
pub struct Runtime {
    /// 全局变量存储
    globals: HashMap<String, Value>,
    /// 函数表
    functions: HashMap<String, Arc<Function>>,
    /// 类型信息
    types: HashMap<String, TypeInfo>,
}

/// 值
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    None,
    Unit,
    Array(Vec<Value>),
    Struct(HashMap<String, Value>),
    Closure(Arc<Function>, Vec<Value>),
}

/// 函数
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Instruction>,
    pub is_builtin: bool,
}

/// 指令（用于解释执行）
#[derive(Debug, Clone)]
pub enum Instruction {
    Push(Value),
    Pop,
    Load(String),
    Store(String),
    Call(String, usize),
    Return,
    Jump(usize),
    JumpIf(usize),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Not,
    Neg,
    Nop,
}

/// 类型信息
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub kind: TypeKind,
}

/// 类型种类
#[derive(Debug, Clone)]
pub enum TypeKind {
    Primitive,
    Struct(Vec<FieldInfo>),
    Enum(Vec<VariantInfo>),
    Array(Box<TypeInfo>),
    Function(Vec<TypeInfo>, Box<TypeInfo>),
}

/// 字段信息
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub type_info: TypeInfo,
}

/// 变体信息
#[derive(Debug, Clone)]
pub struct VariantInfo {
    pub name: String,
    pub data_type: Option<TypeInfo>,
}

impl Runtime {
    /// 创建新的运行时
    pub fn new() -> Self {
        let mut runtime = Self {
            globals: HashMap::new(),
            functions: HashMap::new(),
            types: HashMap::new(),
        };
        
        // 注册内置函数
        runtime.register_builtins();
        
        runtime
    }
    
    /// 注册内置函数
    fn register_builtins(&mut self) {
        // println 函数
        self.functions.insert(
            "println".to_string(),
            Arc::new(Function {
                name: "println".to_string(),
                params: vec!["value".to_string()],
                body: vec![],
                is_builtin: true,
            }),
        );
        
        // panic 函数
        self.functions.insert(
            "panic".to_string(),
            Arc::new(Function {
                name: "panic".to_string(),
                params: vec!["message".to_string()],
                body: vec![],
                is_builtin: true,
            }),
        );
    }
    
    /// 注册函数
    pub fn register_function(&mut self, func: Function) {
        self.functions.insert(func.name.clone(), Arc::new(func));
    }
    
    /// 执行字节码
    pub fn execute(&mut self, bytecode: &[u8]) -> Result<Value, String> {
        // 验证魔数
        if bytecode.len() < 4 || &bytecode[0..4] != b"AETH" {
            return Err("Invalid bytecode format".to_string());
        }
        
        // 解析并执行
        self.interpret(bytecode)
    }
    
    /// 解释执行字节码
    fn interpret(&self, bytecode: &[u8]) -> Result<Value, String> {
        // 简化实现：只返回 Unit
        Ok(Value::Unit)
    }
    
    /// 调用函数
    pub fn call_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        match self.functions.get(name) {
            Some(func) => {
                if func.is_builtin {
                    self.call_builtin(name, args)
                } else {
                    // TODO: 实现用户函数调用
                    Err(format!("User function '{}' not yet implemented", name))
                }
            }
            None => Err(format!("Function '{}' not found", name)),
        }
    }
    
    /// 调用内置函数
    fn call_builtin(&self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        match name {
            "println" => {
                if let Some(value) = args.first() {
                    println!("{:?}", value);
                }
                Ok(Value::Unit)
            }
            "panic" => {
                if let Some(Value::String(msg)) = args.first() {
                    Err(format!("Panic: {}", msg))
                } else {
                    Err("Panic called with invalid argument".to_string())
                }
            }
            _ => Err(format!("Unknown builtin function: {}", name)),
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = Runtime::new();
        assert!(runtime.functions.contains_key("println"));
        assert!(runtime.functions.contains_key("panic"));
    }

    #[test]
    fn test_value_clone() {
        let value = Value::Int(42);
        let cloned = value.clone();
        assert_eq!(value, cloned);
    }
}
