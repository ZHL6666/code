//! Aether 语义分析模块
//! 
//! 执行类型检查、作用域分析和借用检查

use crate::ast::*;
use crate::error::{Error, Result};
use std::collections::HashMap;

pub mod borrow_check;
pub use borrow_check::BorrowChecker;

/// 语义分析器
pub struct SemanticAnalyzer {
    /// 符号表
    symbols: HashMap<String, Symbol>,
    /// 当前作用域
    scope_stack: Vec<Scope>,
    /// 类型环境
    type_env: HashMap<String, TypeDef>,
}

/// 符号
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub mutable: bool,
}

/// 符号类型
#[derive(Debug, Clone)]
pub enum SymbolType {
    Variable(Type),
    Function(FunctionType),
    Type(Type),
    Const(Type),
}

/// 函数类型
#[derive(Debug, Clone)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub return_type: Option<Type>,
    pub generics: Vec<String>,
}

/// 类型定义
#[derive(Debug, Clone)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
    Alias(Type),
    Trait(TraitDef),
}

/// 作用域
#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>,
}

impl Scope {
    pub fn new(parent: Option<usize>) -> Self {
        Self {
            symbols: HashMap::new(),
            parent,
        }
    }
}

impl SemanticAnalyzer {
    /// 创建新的语义分析器
    pub fn new() -> Self {
        let mut analyzer = Self {
            symbols: HashMap::new(),
            scope_stack: Vec::new(),
            type_env: HashMap::new(),
        };
        
        // 注册内置类型
        analyzer.register_builtin_types();
        
        // 进入全局作用域
        analyzer.enter_scope();
        
        analyzer
    }
    
    /// 注册内置类型
    fn register_builtin_types(&mut self) {
        self.type_env.insert("Int".to_string(), TypeDef::Alias(Type::Simple("Int".to_string())));
        self.type_env.insert("Float".to_string(), TypeDef::Alias(Type::Simple("Float".to_string())));
        self.type_env.insert("Bool".to_string(), TypeDef::Alias(Type::Simple("Bool".to_string())));
        self.type_env.insert("Char".to_string(), TypeDef::Alias(Type::Simple("Char".to_string())));
        self.type_env.insert("String".to_string(), TypeDef::Alias(Type::Simple("String".to_string())));
        self.type_env.insert("Unit".to_string(), TypeDef::Alias(Type::Unit));
    }
    
    /// 进入新作用域
    fn enter_scope(&mut self) {
        let parent = if self.scope_stack.is_empty() {
            None
        } else {
            Some(self.scope_stack.len() - 1)
        };
        self.scope_stack.push(Scope::new(parent));
    }
    
    /// 退出作用域
    fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }
    
    /// 定义符号
    fn define_symbol(&mut self, symbol: Symbol) -> Result<()> {
        if let Some(current_scope) = self.scope_stack.last_mut() {
            if current_scope.symbols.contains_key(&symbol.name) {
                return Err(Error::semantic(
                    format!("Duplicate definition: {}", symbol.name),
                    0, 0, "<semantic>",
                ));
            }
            current_scope.symbols.insert(symbol.name.clone(), symbol);
        }
        Ok(())
    }
    
    /// 查找符号
    fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(symbol) = scope.symbols.get(name) {
                return Some(symbol);
            }
        }
        None
    }
    
    /// 分析模块
    pub fn analyze(&mut self, module: Module) -> Result<Module> {
        // 第一遍：收集所有类型和函数声明
        for item in &module.items {
            self.collect_declaration(item)?;
        }
        
        // 第二遍：分析函数体和表达式
        for item in module.items {
            self.analyze_item(item)?;
        }
        
        Ok(module)
    }
    
    /// 收集声明
    fn collect_declaration(&mut self, item: &Item) -> Result<()> {
        match item {
            Item::Function(func) => {
                let func_type = FunctionType {
                    params: func.params.iter().map(|p| p.param_type.clone()).collect(),
                    return_type: func.return_type.clone(),
                    generics: func.generics.iter().map(|g| g.name.clone()).collect(),
                };
                
                let symbol = Symbol {
                    name: func.name.clone(),
                    symbol_type: SymbolType::Function(func_type),
                    mutable: false,
                };
                self.define_symbol(symbol)?;
            }
            Item::Struct(struct_def) => {
                let type_def = TypeDef::Struct(struct_def.clone());
                self.type_env.insert(struct_def.name.clone(), type_def);
            }
            Item::Enum(enum_def) => {
                let type_def = TypeDef::Enum(enum_def.clone());
                self.type_env.insert(enum_def.name.clone(), type_def);
            }
            Item::Const(const_def) => {
                let symbol = Symbol {
                    name: const_def.name.clone(),
                    symbol_type: SymbolType::Const(const_def.const_type.clone()),
                    mutable: false,
                };
                self.define_symbol(symbol)?;
            }
            _ => {}
        }
        Ok(())
    }
    
    /// 分析项
    fn analyze_item(&mut self, item: Item) -> Result<Item> {
        match item {
            Item::Function(mut func) => {
                self.enter_scope();
                
                // 添加参数到符号表
                for param in &func.params {
                    let symbol = Symbol {
                        name: param.name.clone(),
                        symbol_type: SymbolType::Variable(param.param_type.clone()),
                        mutable: param.mutable,
                    };
                    self.define_symbol(symbol)?;
                }
                
                // 分析函数体
                func.body = self.analyze_block(func.body)?;
                
                self.exit_scope();
                
                Ok(Item::Function(func))
            }
            Item::Statement(stmt) => {
                Ok(Item::Statement(self.analyze_statement(stmt)?))
            }
            _ => Ok(item),
        }
    }
    
    /// 分析块
    fn analyze_block(&mut self, block: Block) -> Result<Block> {
        self.enter_scope();
        
        let mut statements = Vec::new();
        for stmt in block.statements {
            statements.push(self.analyze_statement(stmt)?);
        }
        
        self.exit_scope();
        
        Ok(Block { statements })
    }
    
    /// 分析语句
    fn analyze_statement(&mut self, stmt: Statement) -> Result<Statement> {
        match stmt {
            Statement::Let(name, var_type, init, mutable) => {
                let expr_type = if let Some(expr) = init {
                    let analyzed_expr = self.analyze_expression(*expr)?;
                    Some(Box::new(analyzed_expr))
                } else {
                    None
                };
                
                // 推断或验证类型
                let final_type = if let Some(ref t) = var_type {
                    t.clone()
                } else if let Some(ref expr) = expr_type {
                    self.infer_expression_type(expr)?
                } else {
                    return Err(Error::semantic(
                        format!("Cannot infer type for variable '{}'", name),
                        0, 0, "<semantic>",
                    ));
                };
                
                let symbol = Symbol {
                    name: name.clone(),
                    symbol_type: SymbolType::Variable(final_type),
                    mutable,
                };
                self.define_symbol(symbol)?;
                
                Ok(Statement::Let(name, var_type, expr_type, mutable))
            }
            
            Statement::Assign(var_name, expr) => {
                // 检查变量是否存在且可变
                match self.lookup_symbol(&var_name) {
                    Some(symbol) => {
                        if !symbol.mutable {
                            return Err(Error::semantic(
                                format!("Cannot assign to immutable variable '{}'", var_name),
                                0, 0, "<semantic>",
                            ));
                        }
                    }
                    None => {
                        return Err(Error::semantic(
                            format!("Undefined variable: {}", var_name),
                            0, 0, "<semantic>",
                        ));
                    }
                }
                
                let analyzed_expr = self.analyze_expression(*expr)?;
                Ok(Statement::Assign(var_name, Box::new(analyzed_expr)))
            }
            
            Statement::If(condition, then_block, else_branch) => {
                let analyzed_condition = self.analyze_expression(*condition)?;
                let analyzed_then = self.analyze_block(then_block)?;
                let analyzed_else = else_branch
                    .map(|e| -> Result<_> {
                        Ok(Box::new(self.analyze_statement(*e)?))
                    })
                    .transpose()?;
                
                Ok(Statement::If(Box::new(analyzed_condition), analyzed_then, analyzed_else.map(Box::new)))
            }
            
            Statement::While(condition, body) => {
                let analyzed_condition = self.analyze_expression(*condition)?;
                let analyzed_body = self.analyze_block(body)?;
                Ok(Statement::While(Box::new(analyzed_condition), analyzed_body))
            }
            
            Statement::For(var, iterable, body) => {
                self.enter_scope();
                
                // 添加循环变量
                let iter_type = self.infer_expression_type(&iterable)?;
                let element_type = self.get_iterator_element_type(&iter_type)?;
                
                let symbol = Symbol {
                    name: var.clone(),
                    symbol_type: SymbolType::Variable(element_type),
                    mutable: true, // 循环变量通常是可变的
                };
                self.define_symbol(symbol)?;
                
                let analyzed_iterable = self.analyze_expression(*iterable)?;
                let analyzed_body = self.analyze_block(body)?;
                
                self.exit_scope();
                
                Ok(Statement::For(var, Box::new(analyzed_iterable), analyzed_body))
            }
            
            Statement::Return(expr) => {
                let analyzed_expr = expr.map(|e| -> Result<_> {
                    Ok(Box::new(self.analyze_expression(*e)?))
                }).transpose()?;
                Ok(Statement::Return(analyzed_expr))
            }
            
            Statement::Expr(expr) => {
                Ok(Statement::Expr(self.analyze_expression(expr)?))
            }
            
            _ => Ok(stmt),
        }
    }
    
    /// 分析表达式
    fn analyze_expression(&mut self, expr: Expression) -> Result<Expression> {
        match expr {
            Expression::Binary(left, op, right) => {
                let analyzed_left = self.analyze_expression(*left)?;
                let analyzed_right = self.analyze_expression(*right)?;
                
                // 类型检查
                let left_type = self.infer_expression_type(&analyzed_left)?;
                let right_type = self.infer_expression_type(&analyzed_right)?;
                
                if left_type != right_type {
                    return Err(Error::type_error(
                        format!("Binary operator {:?} requires same types", op),
                        0, 0,
                        format!("{}", left_type),
                        format!("{}", right_type),
                        "<semantic>",
                    ));
                }
                
                Ok(Expression::Binary(Box::new(analyzed_left), op, Box::new(analyzed_right)))
            }
            
            Expression::Unary(op, operand) => {
                let analyzed_operand = self.analyze_expression(*operand)?;
                Ok(Expression::Unary(op, Box::new(analyzed_operand)))
            }
            
            Expression::Call(func, args) => {
                let analyzed_args: Vec<Expression> = args
                    .into_iter()
                    .map(|arg| self.analyze_expression(arg))
                    .collect::<Result<_>>()?;
                
                // 查找函数
                match self.lookup_symbol(&func) {
                    Some(Symbol { symbol_type: SymbolType::Function(func_type), .. }) => {
                        // 检查参数数量
                        if analyzed_args.len() != func_type.params.len() {
                            return Err(Error::semantic(
                                format!(
                                    "Function '{}' expects {} arguments but got {}",
                                    func,
                                    func_type.params.len(),
                                    analyzed_args.len()
                                ),
                                0, 0, "<semantic>",
                            ));
                        }
                        
                        // 检查参数类型
                        for (i, (arg, expected)) in analyzed_args.iter().zip(&func_type.params).enumerate() {
                            let arg_type = self.infer_expression_type(arg)?;
                            if arg_type != *expected {
                                return Err(Error::type_error(
                                    format!("Argument {} type mismatch", i),
                                    0, 0,
                                    format!("{}", expected),
                                    format!("{}", arg_type),
                                    "<semantic>",
                                ));
                            }
                        }
                    }
                    Some(_) => {
                        return Err(Error::semantic(
                            format!("'{}' is not a function", func),
                            0, 0, "<semantic>",
                        ));
                    }
                    None => {
                        return Err(Error::semantic(
                            format!("Undefined function: {}", func),
                            0, 0, "<semantic>",
                        ));
                    }
                }
                
                Ok(Expression::Call(Box::new(Expression::Variable(func)), analyzed_args))
            }
            
            Expression::Variable(name) => {
                match self.lookup_symbol(&name) {
                    Some(_) => Ok(Expression::Variable(name)),
                    None => Err(Error::semantic(
                        format!("Undefined variable: {}", name),
                        0, 0, "<semantic>",
                    )),
                }
            }
            
            // 字面量和其他表达式直接返回
            _ => Ok(expr),
        }
    }
    
    /// 推断表达式类型
    fn infer_expression_type(&self, expr: &Expression) -> Result<Type> {
        match expr {
            Expression::Literal(literal) => {
                Ok(match literal {
                    Literal::Int(_) => Type::Simple("Int".to_string()),
                    Literal::Float(_) => Type::Simple("Float".to_string()),
                    Literal::Bool(_) => Type::Simple("Bool".to_string()),
                    Literal::Char(_) => Type::Simple("Char".to_string()),
                    Literal::Str(_) => Type::Simple("String".to_string()),
                    Literal::None => Type::Simple("None".to_string()),
                    Literal::Unit => Type::Unit,
                })
            }
            
            Expression::Variable(name) => {
                match self.lookup_symbol(name) {
                    Some(Symbol { symbol_type: SymbolType::Variable(t), .. }) => Ok(t.clone()),
                    Some(Symbol { symbol_type: SymbolType::Const(t), .. }) => Ok(t.clone()),
                    _ => Err(Error::semantic(
                        format!("Cannot infer type of '{}'", name),
                        0, 0, "<semantic>",
                    )),
                }
            }
            
            Expression::Binary(_, _, _) => {
                // 二元运算的结果类型通常与操作数相同
                // 这里简化处理，实际需要更复杂的逻辑
                Ok(Type::Simple("Int".to_string()))
            }
            
            _ => Err(Error::semantic(
                "Cannot infer type".to_string(),
                0, 0, "<semantic>",
            )),
        }
    }
    
    /// 获取迭代器元素类型
    fn get_iterator_element_type(&self, iter_type: &Type) -> Result<Type> {
        // 简化处理，假设迭代器返回 Int
        Ok(Type::Simple("Int".to_string()))
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 语义分析入口函数
pub fn analyze(ast: Module) -> Result<Module> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_types() {
        let analyzer = SemanticAnalyzer::new();
        assert!(analyzer.type_env.contains_key("Int"));
        assert!(analyzer.type_env.contains_key("Bool"));
    }
}
