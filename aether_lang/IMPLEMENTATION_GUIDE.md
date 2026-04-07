# Aether 编译器架构实现指南

## 1. 编译器流水线 (Compiler Pipeline)

Aether 编译器采用多阶段设计，确保高性能和优秀的错误报告。

### 1.1 阶段概览

```
源代码 (.aether) 
    ↓
[1] 词法分析 (Lexer) → Token 流
    ↓
[2] 语法分析 (Parser) → 抽象语法树 (AST)
    ↓
[3] 语义分析 (Semantic Analyzer) → 注解 AST + 符号表
    ↓
[4] 中间代码生成 (IR Generator) → Aether IR
    ↓
[5] 优化器 (Optimizer) → 优化后的 IR
    ↓
[6] 代码生成 (Codegen) → LLVM IR / WebAssembly / C
    ↓
[7] 后端 (Backend) → 机器码 / WASM / 对象文件
    ↓
可执行文件 / 库
```

## 2. 核心数据结构

### 2.1 Token 定义

```rust
// src/compiler/lexer/token.rs
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // 字面量
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Char(char),
    
    // 标识符与关键字
    Identifier(String),
    Keyword(Keyword),
    
    // 运算符
    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual, NotEqual,
    Less, LessEqual, Greater, GreaterEqual,
    And, Or, Not,
    
    // 分隔符
    LParen, RParen, LBrace, RBrace,
    LBracket, RBracket,
    Comma, Dot, Semicolon, Colon,
    Arrow, FatArrow,
    
    // 特殊
    Eof,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Let, Mut, Const,
    Fn, Return, If, Else, Match,
    While, For, Loop, Break, Continue,
    Struct, Enum, Trait, Impl, Type,
    Pub, Priv, Protected,
    Async, Await, Spawn,
    Use, Mod,
    True, False, Nil,
    Self, Super,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub line: usize,
    pub column: usize,
    pub span: Span,
}
```

### 2.2 AST 节点定义

```rust
// src/compiler/ast/mod.rs
use crate::compiler::span::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(Ident),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>, Option<Box<Expr>>), // func(args, ?block)
    FieldAccess(Box<Expr>, Ident),
    Index(Box<Expr>, Box<Expr>),
    Lambda(Vec<Param>, Option<Type>, Box<Expr>),
    Block(Vec<Stmt>, Option<Box<Expr>>),
    If(Box<Expr>, Block, Option<Block>),
    Match(Box<Expr>, Vec<MatchArm>),
    Range(Box<Expr>, Box<Expr>, bool), // start, end, inclusive
    Tuple(Vec<Expr>),
    RecordUpdate(Box<Expr>, Vec<FieldInit>),
    Try(Box<Expr>),
    Await(Box<Expr>),
    Spawn(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(Pattern, Option<Type>, Option<Expr>),
    Const(Ident, Type, Expr),
    ExprStmt(Expr),
    Return(Option<Expr>),
    Break(Option<Expr>),
    Continue,
    While(Box<Expr>, Block),
    For(Pattern, Box<Expr>, Block),
    Loop(Block),
    Item(Item),
}

#[derive(Debug, Clone)]
pub enum Item {
    Fn(FnDef),
    Struct(StructDef),
    Enum(EnumDef),
    Trait(TraitDef),
    Impl(ImplDef),
    TypeAlias(TypeAliasDef),
    Mod(ModDef),
    Use(UseDef),
}
```

### 2.3 符号表与作用域

```rust
// src/compiler/sema/symbol_table.rs
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub typ: Type,
    pub span: Span,
    pub visibility: Visibility,
    pub mutability: Mutability,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable,
    Constant,
    Function,
    Struct,
    Enum,
    EnumVariant,
    Trait,
    TraitMethod,
    TypeAlias,
    Module,
}

pub struct Scope {
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub symbols: HashMap<String, Symbol>,
    pub level: usize,
}

pub struct SymbolTable {
    pub global_scope: Rc<RefCell<Scope>>,
    pub current_scope: Rc<RefCell<Scope>>,
}

impl SymbolTable {
    pub fn new() -> Self { /* ... */ }
    pub fn push_scope(&mut self) { /* ... */ }
    pub fn pop_scope(&mut self) { /* ... */ }
    pub fn insert(&mut self, symbol: Symbol) -> Result<()> { /* ... */ }
    pub fn lookup(&self, name: &str) -> Option<Symbol> { /* ... */ }
    pub fn resolve_type(&self, type_ref: &TypeRef) -> Result<Type> { /* ... */ }
}
```

## 3. 关键算法实现

### 3.1 递归下降解析器 (Recursive Descent Parser)

```rust
// src/compiler/parser/mod.rs
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn parse(&mut self) -> Result<Ast> {
        let mut items = Vec::new();
        while !self.is_at_end() {
            items.push(self.declaration()?);
        }
        Ok(Ast { items })
    }
    
    fn declaration(&mut self) -> Result<Item> {
        if self.match_keyword(Keyword::Pub) || self.match_keyword(Keyword::Priv) {
            // 处理可见性
        }
        
        if self.match_keyword(Keyword::Fn) { return self.fn_def(); }
        if self.match_keyword(Keyword::Struct) { return self.struct_def(); }
        if self.match_keyword(Keyword::Enum) { return self.enum_def(); }
        if self.match_keyword(Keyword::Trait) { return self.trait_def(); }
        if self.match_keyword(Keyword::Impl) { return self.impl_def(); }
        
        self.stmt()
    }
    
    fn expr(&mut self) -> Result<Expr> {
        self.assignment()
    }
    
    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or_expr()?;
        
        if self.match_token(TokenType::Equal) {
            let value = self.assignment()?;
            return Ok(Expr::Assign(Box::new(expr), Box::new(value)));
        }
        
        Ok(expr)
    }
    
    // ... 其他优先级层次的表达式解析
}
```

### 3.2 类型推断算法 (Hindley-Milner)

```rust
// src/compiler/sema/type_inference.rs
use std::collections::HashMap;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TypeVar {
    Named(String),
    Generated(usize),
}

#[derive(Debug, Clone)]
pub enum InferType {
    Var(TypeVar),
    Int,
    Float,
    Bool,
    String,
    Array(Box<InferType>),
    Option(Box<InferType>),
    Result(Box<InferType>, Box<InferType>),
    Fn(Vec<InferType>, Box<InferType>),
    Named(String, Vec<InferType>),
}

pub struct TypeInferencer {
    constraints: Vec<(InferType, InferType)>,
    substitutions: HashMap<TypeVar, InferType>,
    next_var: usize,
}

impl TypeInferencer {
    pub fn fresh_var(&mut self) -> InferType {
        let var = TypeVar::Generated(self.next_var);
        self.next_var += 1;
        InferType::Var(var)
    }
    
    pub fn unify(&mut self, t1: InferType, t2: InferType) -> Result<()> {
        match (t1, t2) {
            (InferType::Var(v), _) => self.bind_var(v, t2),
            (_, InferType::Var(v)) => self.bind_var(v, t1),
            (InferType::Int, InferType::Int) => Ok(()),
            (InferType::Bool, InferType::Bool) => Ok(()),
            (InferType::Array(a1), InferType::Array(a2)) => {
                self.unify(*a1, *a2)
            }
            (InferType::Fn(params1, ret1), InferType::Fn(params2, ret2)) => {
                if params1.len() != params2.len() {
                    return Err(TypeError::ArityMismatch);
                }
                for (p1, p2) in params1.into_iter().zip(params2) {
                    self.unify(p1, p2)?;
                }
                self.unify(*ret1, *ret2)
            }
            _ => Err(TypeError::Mismatch(t1, t2)),
        }
    }
    
    fn bind_var(&mut self, var: TypeVar, ty: InferType) -> Result<()> {
        if let InferType::Var(v) = &ty {
            if v == &var {
                return Ok(()); // 避免无限递归
            }
        }
        self.substitutions.insert(var, ty);
        Ok(())
    }
    
    pub fn infer(&mut self, expr: &Expr) -> Result<InferType> {
        match expr {
            Expr::Literal(Literal::Int(_)) => Ok(InferType::Int),
            Expr::Literal(Literal::Bool(_)) => Ok(InferType::Bool),
            Expr::Binary(left, op, right) => {
                let t1 = self.infer(left)?;
                let t2 = self.infer(right)?;
                
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        self.unify(t1, t2.clone())?;
                        self.unify(t1.clone(), InferType::Int)?;
                        Ok(InferType::Int)
                    }
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        self.unify(t1, t2)?;
                        Ok(InferType::Bool)
                    }
                    _ => Err(TypeError::InvalidOperator),
                }
            }
            // ... 其他表达式类型
            _ => Ok(self.fresh_var()),
        }
    }
}
```

### 3.3 借用检查器 (Borrow Checker)

```rust
// src/compiler/sema/borrow_checker.rs
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorrowKind {
    Immutable,
    Mutable,
}

#[derive(Debug, Clone)]
pub struct Loan {
    pub place: String,
    pub kind: BorrowKind,
    pub span: Span,
    pub lifetime: Lifetime,
}

pub struct BorrowChecker {
    loans: Vec<Loan>,
    moved_places: HashSet<String>,
    current_lifetime: usize,
}

impl BorrowChecker {
    pub fn check(&mut self, stmts: &[Stmt]) -> Result<()> {
        for stmt in stmts {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }
    
    fn check_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let(pattern, _, init) => {
                if let Some(expr) = init {
                    self.check_expr(expr)?;
                }
                // 记录新变量的所有权
            }
            Stmt::ExprStmt(expr) => self.check_expr(expr)?,
            Stmt::Return(Some(expr)) => {
                self.check_expr(expr)?;
                // 检查返回值是否包含已借用的数据
            }
            // ... 其他语句
            _ => {}
        }
        Ok(())
    }
    
    fn check_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::FieldAccess(obj, field) => {
                self.check_expr(obj)?;
                // 检查字段访问的合法性
            }
            Expr::Call(func, args, _) => {
                self.check_expr(func)?;
                for arg in args {
                    self.check_expr(arg)?;
                }
                // 检查参数传递的所有权转移
            }
            Expr::Unary(UnOp::Deref, inner) => {
                self.check_expr(inner)?;
                // 检查解引用的合法性
            }
            // ... 其他表达式
            _ => {}
        }
        Ok(())
    }
    
    fn check_loan_conflicts(&self, new_loan: &Loan) -> Result<()> {
        for loan in &self.loans {
            if loan.place == new_loan.place {
                if loan.kind == BorrowKind::Mutable || new_loan.kind == BorrowKind::Mutable {
                    return Err(BorrowError::ConflictingBorrows(
                        loan.span,
                        new_loan.span,
                    ));
                }
            }
        }
        Ok(())
    }
}
```

## 4. 中间表示 (Aether IR)

### 4.1 IR 指令集

```rust
// src/compiler/ir/mod.rs
#[derive(Debug, Clone)]
pub enum IrInstr {
    // 算术运算
    Add(Reg, Reg, Reg),
    Sub(Reg, Reg, Reg),
    Mul(Reg, Reg, Reg),
    Div(Reg, Reg, Reg),
    
    // 逻辑运算
    And(Reg, Reg, Reg),
    Or(Reg, Reg, Reg),
    Not(Reg, Reg),
    
    // 比较
    Eq(Reg, Reg, Reg),
    Ne(Reg, Reg, Reg),
    Lt(Reg, Reg, Reg),
    Le(Reg, Reg, Reg),
    Gt(Reg, Reg, Reg),
    Ge(Reg, Reg, Reg),
    
    // 内存
    Load(Reg, Reg),      // load reg, [addr]
    Store(Reg, Reg),     // store [addr], reg
    Alloca(Reg, Type),   // allocate stack memory
    
    // 控制流
    Jump(BlockId),
    Branch(Reg, BlockId, BlockId),
    Return(Option<Reg>),
    Call(Reg, FuncId, Vec<Reg>),
    
    // 特殊
    Phi(Vec<(BlockId, Reg)>),
    Nop,
}

#[derive(Debug, Clone)]
pub struct IrBlock {
    pub id: BlockId,
    pub instrs: Vec<IrInstr>,
    pub terminators: Vec<Terminator>,
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub blocks: Vec<IrBlock>,
    pub entry_block: BlockId,
}
```

### 4.2 IR 优化示例

```rust
// src/compiler/optimizer/mod.rs
pub struct Optimizer {
    passes: Vec<Box<dyn OptimizationPass>>,
}

impl Optimizer {
    pub fn new() -> Self {
        let mut opt = Self { passes: Vec::new() };
        
        // 注册优化通道
        opt.add_pass(Box::new(ConstFoldPass));
        opt.add_pass(Box::new(DeadCodeElimination));
        opt.add_pass(Box::new(InliningPass));
        opt.add_pass(Box::new(LoopInvariantCodeMotion));
        opt.add_pass(Box::new(SimpleLoopUnroll));
        
        opt
    }
    
    pub fn optimize(&self, ir: &mut IrModule) {
        for pass in &self.passes {
            pass.run(ir);
        }
    }
}

// 常量折叠示例
struct ConstFoldPass;

impl OptimizationPass for ConstFoldPass {
    fn run(&self, module: &mut IrModule) {
        for func in &mut module.functions {
            for block in &mut func.blocks {
                let mut i = 0;
                while i < block.instrs.len() {
                    if let IrInstr::Add(dest, src1, src2) = &block.instrs[i] {
                        if let (Some(v1), Some(v2)) = 
                            (self.get_const_value(src1), self.get_const_value(src2)) 
                        {
                            block.instrs[i] = IrInstr::LoadConst(*dest, v1 + v2);
                            continue;
                        }
                    }
                    i += 1;
                }
            }
        }
    }
}
```

## 5. 错误处理与诊断

### 5.1 友好的错误消息

```rust
// src/compiler/diagnostic.rs
use codespan_reporting::term::termcolor::ColorChoice;
use codespan_reporting::{files, term};

pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    pub spans: Vec<Span>,
    pub notes: Vec<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self {
            level: Level::Error,
            message: message.into(),
            spans: vec![span],
            notes: Vec::new(),
            help: None,
        }
    }
    
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
    
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
    
    pub fn emit(&self, file: &SimpleFile<String, String>) {
        let config = term::Config::default();
        let writer = StandardStream::stderr(ColorChoice::Auto);
        
        let diagnostic = codespan::Diagnostic::new(self.level.to_codespan())
            .with_message(&self.message)
            .with_labels(
                self.spans.iter()
                    .map(|span| {
                        Label::primary((), span.start..span.end)
                            .with_message("here")
                    })
                    .collect()
            )
            .with_notes(self.notes.clone())
            .with_help_message(self.help.as_deref());
        
        term::emit(&mut writer.lock(), &config, file, &diagnostic).unwrap();
    }
}
```

### 5.2 错误恢复策略

```rust
// src/compiler/parser/error_recovery.rs
impl Parser {
    fn synchronize(&mut self) {
        loop {
            if let Some(token) = self.peek() {
                match token.kind {
                    TokenType::Semicolon => {
                        self.advance();
                        return;
                    }
                    TokenType::Keyword(kw) if matches!(kw, 
                        Keyword::Fn | Keyword::Struct | Keyword::Enum | 
                        Keyword::Trait | Keyword::Impl | Keyword::Let
                    ) => return,
                    TokenType::Eof => return,
                    _ => self.advance(),
                }
            }
        }
    }
    
    fn safe_parse<T>(&mut self, parser: impl FnOnce(&mut Self) -> Result<T>) -> Option<T> {
        match parser(self) {
            Ok(result) => Some(result),
            Err(e) => {
                self.errors.push(e);
                self.synchronize();
                None
            }
        }
    }
}
```

## 6. 构建与编译

### 6.1 Cargo.toml 配置

```toml
[package]
name = "aether-compiler"
version = "0.1.0"
edition = "2021"
authors = ["Aether Team"]
description = "Compiler for the Aether programming language"

[dependencies]
# 词法分析和解析
logos = "0.13"

# 错误报告
codespan-reporting = "0.11"

# 数据结构
indexmap = "2.0"
smallvec = "1.11"

# LLVM 绑定
inkwell = { version = "0.2", features = ["llvm16-0"] }

# 并行处理
rayon = "1.8"

# 序列化 (用于缓存)
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"

# 命令行
clap = { version = "4.4", features = ["derive"] }

[dev-dependencies]
insta = "1.34"  # 快照测试
criterion = "0.5"  # 基准测试

[[bin]]
name = "aether"
path = "src/main.rs"

[lib]
name = "aether_compiler"
path = "src/lib.rs"
```

### 6.2 主入口点

```rust
// src/main.rs
use clap::{Parser, Subcommand};
use aether_compiler::{compile, CompileOptions, Target};

#[derive(Parser)]
#[command(name = "aether")]
#[command(about = "Aether Programming Language Compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile Aether source code
    Build {
        #[arg(required = true)]
        input: String,
        
        #[arg(short, long, default_value = "native")]
        target: String,
        
        #[arg(short, long)]
        output: Option<String>,
        
        #[arg(long)]
        optimize: bool,
        
        #[arg(long)]
        debug: bool,
    },
    
    /// Run Aether source code directly
    Run {
        #[arg(required = true)]
        input: String,
        
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    
    /// Start language server
    Lsp,
    
    /// Format Aether code
    Fmt {
        #[arg(required = true)]
        path: String,
        
        #[arg(short, long)]
        check: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Build { input, target, output, optimize, debug } => {
            let options = CompileOptions {
                target: Target::from_str(&target).unwrap(),
                optimization_level: if optimize { 3 } else { 0 },
                debug_info: debug,
                ..Default::default()
            };
            
            match compile(&input, &options) {
                Ok(output_path) => {
                    println!("✓ Compiled successfully: {}", output_path);
                }
                Err(errors) => {
                    for err in errors {
                        err.emit();
                    }
                    std::process::exit(1);
                }
            }
        }
        
        Commands::Run { input, args } => {
            // 编译到临时文件并执行
        }
        
        Commands::Lsp => {
            // 启动语言服务器
            aether_lsp::start();
        }
        
        Commands::Fmt { path, check } => {
            // 格式化代码
            aether_tools::format(&path, check);
        }
    }
}
```

## 7. 下一步开发计划

### Phase 1: 基础编译器 (1-2 个月)
- [ ] 完成词法分析和语法分析
- [ ] 实现基本类型检查
- [ ] 生成简单 LLVM IR
- [ ] 支持基本数据类型和控制流

### Phase 2: 高级特性 (2-3 个月)
- [ ] 完整的所有权和借用检查
- [ ] 泛型和 Traits 实现
- [ ] 模式匹配编译
- [ ] async/await 状态机转换

### Phase 3: 优化与工具 (2-3 个月)
- [ ] IR 优化通道
- [ ] 增量编译
- [ ] LSP 语言服务器
- [ ] 代码格式化和 linter

### Phase 4: 生态系统 (持续)
- [ ] 包管理器
- [ ] 标准库完善
- [ ] 文档生成工具
- [ ] 测试框架

---

本指南为 Aether 编译器实现提供详细的技术路线，开发者可按照此架构逐步实现完整的编译器。
