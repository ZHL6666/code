//! Aether 抽象语法树 (AST) 定义模块

use std::fmt;

/// 模块（编译单元）
#[derive(Debug, Clone)]
pub struct Module {
    pub items: Vec<Item>,
}

/// 顶层项
#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Struct(StructDef),
    Enum(EnumDef),
    Trait(TraitDef),
    Impl(ImplBlock),
    TypeAlias(TypeAlias),
    Const(ConstDef),
    Use(UseStatement),
    Module(ModuleDef),
    Statement(Statement),
}

/// 可见性
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

/// 函数定义
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub async_keyword: bool,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub where_clauses: Vec<WhereClause>,
    pub body: Block,
    pub visibility: Visibility,
}

/// 泛型参数
#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<Type>,
}

/// 函数参数
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
    pub mutable: bool,
}

/// Where 子句
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub type_name: Type,
    pub bounds: Vec<Type>,
}

/// 类型定义
#[derive(Debug, Clone)]
pub enum Type {
    Simple(String),
    Generic(String, Vec<Type>),
    Function(Vec<Type>, Box<Type>),
    Array(Box<Type>, Option<usize>),
    Tuple(Vec<Type>),
    Reference(Box<Type>, bool), // &T, &mut T
    Pointer(Box<Type>, bool),   // *const T, *mut T
    Self_,
    Never,
    Unit,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Simple(name) => write!(f, "{}", name),
            Type::Generic(name, generics) => {
                write!(f, "{}<{}>", name, generics.iter().map(|g| format!("{}", g)).collect::<Vec<_>>().join(", "))
            }
            Type::Function(params, ret) => {
                write!(f, "fn({}) -> {}", 
                    params.iter().map(|p| format!("{}", p)).collect::<Vec<_>>().join(", "),
                    ret
                )
            }
            Type::Array(elem, size) => {
                match size {
                    Some(s) => write!(f, "[{}; {}]", elem, s),
                    None => write!(f, "[{}]", elem),
                }
            }
            Type::Tuple(types) => {
                write!(f, "({})", types.iter().map(|t| format!("{}", t)).collect::<Vec<_>>().join(", "))
            }
            Type::Reference(t, mutable) => {
                if *mutable {
                    write!(f, "&mut {}", t)
                } else {
                    write!(f, "&{}", t)
                }
            }
            Type::Pointer(t, constant) => {
                if *constant {
                    write!(f, "*const {}", t)
                } else {
                    write!(f, "*mut {}", t)
                }
            }
            Type::Self_ => write!(f, "Self"),
            Type::Never => write!(f, "!"),
            Type::Unit => write!(f, "()"),
        }
    }
}

/// 结构体定义
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<Field>,
    pub visibility: Visibility,
}

/// 字段定义
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub visibility: Visibility,
}

/// 枚举定义
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<Variant>,
    pub visibility: Visibility,
}

/// 枚举变体
#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub data: Option<Type>,
}

/// Trait 定义
#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub methods: Vec<Function>,
    pub associated_types: Vec<String>,
    pub visibility: Visibility,
}

/// Impl 块
#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub trait_name: Option<String>,
    pub type_name: Type,
    pub generics: Vec<GenericParam>,
    pub methods: Vec<Function>,
}

/// 类型别名
#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub name: String,
    pub aliased_type: Type,
    pub generics: Vec<GenericParam>,
    pub visibility: Visibility,
}

/// 常量定义
#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: String,
    pub const_type: Type,
    pub value: Expression,
    pub visibility: Visibility,
}

/// Use 语句
#[derive(Debug, Clone)]
pub struct UseStatement {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub visibility: Visibility,
}

/// 模块定义
#[derive(Debug, Clone)]
pub struct ModuleDef {
    pub name: String,
    pub visibility: Visibility,
}

/// 语句
#[derive(Debug, Clone)]
pub enum Statement {
    Let(String, Option<Type>, Option<Box<Expression>>, bool), // name, type, init, mutable
    Assign(String, Box<Expression>),
    Expr(Expression),
    Return(Option<Box<Expression>>),
    Break,
    Continue,
    If(Box<Expression>, Block, Option<Box<Statement>>),
    While(Box<Expression>, Block),
    For(String, Box<Expression>, Block),
    Match(Box<Expression>, Vec<MatchArm>),
    Block(Block),
}

/// 匹配臂
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expression,
}

/// 模式
#[derive(Debug, Clone)]
pub enum Pattern {
    Variable(String),
    Literal(Literal),
    Tuple(String, Vec<Pattern>),
    Struct(String, Vec<(String, Pattern)>),
    Wildcard,
}

/// 块
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

/// 表达式
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    Binary(Box<Expression>, BinaryOp, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    MethodCall(Box<Expression>, String, Vec<Expression>),
    FieldAccess(Box<Expression>, String),
    Index(Box<Expression>, Box<Expression>),
    Lambda(Vec<String>, Box<Expression>),
    Array(Vec<Expression>),
    Tuple(Vec<Expression>),
    StructLiteral(Vec<(String, Expression)>),
    Range(Option<Box<Expression>>, Option<Box<Expression>>),
    Assign(Box<String>, Box<Expression>),
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

/// 二元运算符
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    Mod,      // %
    Eq,       // ==
    Ne,       // !=
    Lt,       // <
    Le,       // <=
    Gt,       // >
    Ge,       // >=
    And,      // &&
    Or,       // ||
    BitAnd,   // &
    BitOr,    // |
    BitXor,   // ^
    ShiftLeft,// <<
    ShiftRight,// >>
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Ne => write!(f, "!="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Le => write!(f, "<="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Ge => write!(f, ">="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
            BinaryOp::BitAnd => write!(f, "&"),
            BinaryOp::BitOr => write!(f, "|"),
            BinaryOp::BitXor => write!(f, "^"),
            BinaryOp::ShiftLeft => write!(f, "<<"),
            BinaryOp::ShiftRight => write!(f, ">>"),
        }
    }
}

/// 一元运算符
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg,  // -
    Not,  // !
    BitNot, // ~
    Deref, // *
    AddrOf, // &
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
            UnaryOp::BitNot => write!(f, "~"),
            UnaryOp::Deref => write!(f, "*"),
            UnaryOp::AddrOf => write!(f, "&"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        let t = Type::Generic("Vec".to_string(), vec![Type::Simple("Int".to_string())]);
        assert_eq!(format!("{}", t), "Vec<Int>");
    }

    #[test]
    fn test_binary_op_display() {
        assert_eq!(format!("{}", BinaryOp::Add), "+");
        assert_eq!(format!("{}", BinaryOp::Eq), "==");
    }
}
