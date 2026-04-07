//! Aether 语法分析器模块
//! 
//! 将 Token 流转换为抽象语法树 (AST)

use crate::error::{Error, Result};
use crate::lexer::{Token, TokenType, Keyword};
use crate::ast::*;

/// 语法分析器
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    filename: String,
}

impl Parser {
    /// 创建新的语法分析器
    pub fn new(tokens: Vec<Token>, filename: &str) -> Self {
        Self {
            tokens,
            current: 0,
            filename: filename.to_string(),
        }
    }

    /// 执行语法分析，返回 AST
    pub fn parse(mut self) -> Result<Module> {
        let mut items = Vec::new();
        
        while !self.is_at_end() {
            if let Some(item) = self.parse_item()? {
                items.push(item);
            }
        }
        
        Ok(Module { items })
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn check(&self, token_type: &TokenType) -> bool {
        std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(Error::syntax(
                message.to_string(),
                self.peek().line,
                self.peek().column,
                format!("{:?}", self.peek().token_type),
                &self.filename,
            ))
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while !self.is_at_end() {
            match &self.peek().token_type {
                TokenType::Whitespace(_) | TokenType::Newline | TokenType::Comment(_) => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// 解析顶层项
    fn parse_item(&mut self) -> Result<Option<Item>> {
        self.skip_whitespace_and_comments();
        
        if self.is_at_end() {
            return Ok(None);
        }

        // 可见性修饰符
        let visibility = if self.match_token(&TokenType::Keyword(Keyword::Pub)) {
            Visibility::Public
        } else {
            Visibility::Private
        };

        // 解析不同类型的项
        let item = if self.check(&TokenType::Keyword(Keyword::Fn)) 
            || self.check(&TokenType::Keyword(Keyword::Async))
        {
            Item::Function(self.parse_function(visibility)?)
        } else if self.check(&TokenType::Keyword(Keyword::Struct)) {
            Item::Struct(self.parse_struct(visibility)?)
        } else if self.check(&TokenType::Keyword(Keyword::Enum)) {
            Item::Enum(self.parse_enum(visibility)?)
        } else if self.check(&TokenType::Keyword(Keyword::Trait)) {
            Item::Trait(self.parse_trait(visibility)?)
        } else if self.check(&TokenType::Keyword(Keyword::Impl)) {
            Item::Impl(self.parse_impl()?)
        } else if self.check(&TokenType::Keyword(Keyword::Type)) {
            Item::TypeAlias(self.parse_type_alias(visibility)?)
        } else if self.check(&TokenType::Keyword(Keyword::Const)) {
            Item::Const(self.parse_const(visibility)?)
        } else if self.check(&TokenType::Keyword(Keyword::Use)) {
            Item::Use(self.parse_use(visibility)?)
        } else if self.check(&TokenType::Keyword(Keyword::Mod)) {
            Item::Module(self.parse_module(visibility)?)
        } else {
            // 可能是语句
            return self.parse_statement().map(|stmt| Some(Item::Statement(stmt)));
        };

        Ok(Some(item))
    }

    /// 解析函数
    fn parse_function(&mut self, visibility: Visibility) -> Result<Function> {
        let async_keyword = self.match_token(&TokenType::Keyword(Keyword::Async));
        self.consume(&TokenType::Keyword(Keyword::Fn), "Expected 'fn'")?;
        
        let name = self.consume(
            &TokenType::Identifier(String::new()),
            "Expected function name",
        )?;
        let name_str = match &name.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };

        // 泛型参数
        let generics = if self.match_token(&TokenType::Less) {
            self.parse_generics()?
        } else {
            Vec::new()
        };

        // 函数参数
        self.consume(&TokenType::LeftParen, "Expected '('")?;
        let params = self.parse_parameters()?;
        self.consume(&TokenType::RightParen, "Expected ')'")?;

        // 返回类型
        let return_type = if self.match_token(&TokenType::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Where 子句
        let where_clauses = if self.match_token(&TokenType::Keyword(Keyword::Where)) {
            self.parse_where_clauses()?
        } else {
            Vec::new()
        };

        // 函数体
        let body = if self.match_token(&TokenType::Semicolon) {
            // 函数声明（无实现）
            Block { statements: Vec::new() }
        } else {
            self.parse_block()?
        };

        Ok(Function {
            name: name_str,
            async_keyword,
            generics,
            params,
            return_type,
            where_clauses,
            body,
            visibility,
        })
    }

    /// 解析函数参数
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>> {
        let mut params = Vec::new();

        if self.check(&TokenType::RightParen) {
            return Ok(params);
        }

        loop {
            let mutable = self.match_token(&TokenType::Keyword(Keyword::Mut));
            
            let name = self.consume(
                &TokenType::Identifier(String::new()),
                "Expected parameter name",
            )?;
            let name_str = match &name.token_type {
                TokenType::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };

            self.consume(&TokenType::Colon, "Expected ':'")?;
            let param_type = self.parse_type()?;

            params.push(Parameter {
                name: name_str,
                param_type,
                mutable,
            });

            if !self.match_token(&TokenType::Comma) {
                break;
            }
        }

        Ok(params)
    }

    /// 解析类型
    fn parse_type(&mut self) -> Result<Type> {
        self.skip_whitespace_and_comments();

        if self.match_token(&TokenType::Keyword(Keyword::Self_)) {
            return Ok(Type::Self_);
        }

        let type_name = self.consume(
            &TokenType::Identifier(String::new()),
            "Expected type name",
        )?;
        let name = match &type_name.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };

        // 检查泛型参数
        let generics = if self.match_token(&TokenType::Less) {
            let mut generic_types = Vec::new();
            loop {
                generic_types.push(self.parse_type()?);
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
            self.consume(&TokenType::Greater, "Expected '>'")?;
            generic_types
        } else {
            Vec::new()
        };

        // 检查是否为函数类型
        if self.match_token(&TokenType::Arrow) {
            let return_type = Box::new(self.parse_type()?);
            return Ok(Type::Function(Vec::new(), return_type));
        }

        if generics.is_empty() {
            Ok(Type::Simple(name))
        } else {
            Ok(Type::Generic(name, generics))
        }
    }

    /// 解析泛型参数
    fn parse_generics(&mut self) -> Result<Vec<GenericParam>> {
        let mut generics = Vec::new();

        loop {
            let name = self.consume(
                &TokenType::Identifier(String::new()),
                "Expected generic parameter name",
            )?;
            let name_str = match &name.token_type {
                TokenType::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };

            // 约束条件
            let bounds = if self.match_token(&TokenType::Colon) {
                let mut bounds = Vec::new();
                loop {
                    let bound = self.parse_type()?;
                    bounds.push(bound);
                    if !self.match_token(&TokenType::Plus) {
                        break;
                    }
                }
                bounds
            } else {
                Vec::new()
            };

            generics.push(GenericParam {
                name: name_str,
                bounds,
            });

            if !self.match_token(&TokenType::Comma) {
                break;
            }
        }

        self.consume(&TokenType::Greater, "Expected '>'")?;
        Ok(generics)
    }

    /// 解析 where 子句
    fn parse_where_clauses(&mut self) -> Result<Vec<WhereClause>> {
        let mut clauses = Vec::new();

        loop {
            let type_name = self.parse_type()?;
            self.consume(&TokenType::Colon, "Expected ':'")?;
            
            let mut bounds = Vec::new();
            loop {
                bounds.push(self.parse_type()?);
                if !self.match_token(&TokenType::Plus) {
                    break;
                }
            }

            clauses.push(WhereClause {
                type_name,
                bounds,
            });

            if !self.match_token(&TokenType::Comma) {
                break;
            }
        }

        Ok(clauses)
    }

    /// 解析块
    fn parse_block(&mut self) -> Result<Block> {
        self.consume(&TokenType::LeftBrace, "Expected '{'")?;
        
        let mut statements = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}'")?;
        
        Ok(Block { statements })
    }

    /// 解析语句
    fn parse_statement(&mut self) -> Result<Statement> {
        self.skip_whitespace_and_comments();

        if self.match_token(&TokenType::Keyword(Keyword::Let)) {
            return self.parse_let_statement();
        }

        if self.match_token(&TokenType::Keyword(Keyword::Return)) {
            let expr = if !self.check(&TokenType::Semicolon) && !self.check(&TokenType::RightBrace) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            if self.match_token(&TokenType::Semicolon) {
                return Ok(Statement::Return(expr.map(Box::new)));
            }
            return Ok(Statement::Expr(expr.unwrap_or_else(|| Expression::Literal(Literal::Unit))));
        }

        if self.match_token(&TokenType::Keyword(Keyword::If)) {
            return self.parse_if_statement();
        }

        if self.match_token(&TokenType::Keyword(Keyword::While)) {
            return self.parse_while_statement();
        }

        if self.match_token(&TokenType::Keyword(Keyword::For)) {
            return self.parse_for_statement();
        }

        if self.match_token(&TokenType::Keyword(Keyword::Match)) {
            return self.parse_match_statement();
        }

        if self.match_token(&TokenType::Keyword(Keyword::Break)) {
            return Ok(Statement::Break);
        }

        if self.match_token(&TokenType::Keyword(Keyword::Continue)) {
            return Ok(Statement::Continue);
        }

        // 表达式语句或赋值
        let expr = self.parse_expression()?;
        
        if self.match_token(&TokenType::Semicolon) {
            return Ok(Statement::Expr(expr));
        }

        // 检查是否是赋值
        if matches!(expr, Expression::Variable(_)) {
            if self.match_token(&TokenType::Equal) {
                let value = self.parse_expression()?;
                if self.match_token(&TokenType::Semicolon) {
                    if let Expression::Variable(name) = expr {
                        return Ok(Statement::Assign(name, Box::new(value)));
                    }
                }
            }
        }

        Ok(Statement::Expr(expr))
    }

    /// 解析 let 语句
    fn parse_let_statement(&mut self) -> Result<Statement> {
        let mutable = self.match_token(&TokenType::Keyword(Keyword::Mut));
        
        let name = self.consume(
            &TokenType::Identifier(String::new()),
            "Expected variable name",
        )?;
        let name_str = match &name.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };

        // 可选的类型注解
        let var_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // 可选的初始化表达式
        let init = if self.match_token(&TokenType::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(&TokenType::Semicolon, "Expected ';'")?;

        Ok(Statement::Let(name_str, var_type, init.map(Box::new), mutable))
    }

    /// 解析 if 语句
    fn parse_if_statement(&mut self) -> Result<Statement> {
        let condition = self.parse_expression()?;
        let then_block = self.parse_block()?;
        
        let else_branch = if self.match_token(&TokenType::Keyword(Keyword::Else)) {
            if self.match_token(&TokenType::Keyword(Keyword::If)) {
                // else if
                Some(Box::new(self.parse_if_statement()?))
            } else {
                // else block
                Some(Box::new(Statement::Block(self.parse_block()?)))
            }
        } else {
            None
        };

        Ok(Statement::If(Box::new(condition), then_block, else_branch.map(Box::new)))
    }

    /// 解析 while 语句
    fn parse_while_statement(&mut self) -> Result<Statement> {
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Statement::While(Box::new(condition), body))
    }

    /// 解析 for 语句
    fn parse_for_statement(&mut self) -> Result<Statement> {
        let var = self.consume(
            &TokenType::Identifier(String::new()),
            "Expected variable name",
        )?;
        let var_name = match &var.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };

        self.consume(&TokenType::Keyword(Keyword::In), "Expected 'in'")?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;

        Ok(Statement::For(var_name, Box::new(iterable), body))
    }

    /// 解析 match 语句
    fn parse_match_statement(&mut self) -> Result<Statement> {
        let expr = self.parse_expression()?;
        self.consume(&TokenType::LeftBrace, "Expected '{'")?;
        
        let mut arms = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            self.consume(&TokenType::FatArrow, "Expected '=>'")?;
            let body = self.parse_expression()?;
            
            if self.match_token(&TokenType::Comma) {
                // optional
            }
            
            arms.push(MatchArm { pattern, body });
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}'")?;
        
        Ok(Statement::Match(Box::new(expr), arms))
    }

    /// 解析模式
    fn parse_pattern(&mut self) -> Result<Pattern> {
        self.skip_whitespace_and_comments();
        
        if self.match_token(&TokenType::Underscore) {
            return Ok(Pattern::Wildcard);
        }

        if let TokenType::Integer(n) = self.peek().token_type.clone() {
            self.advance();
            return Ok(Pattern::Literal(Literal::Int(n)));
        }

        if let TokenType::String(s) = self.peek().token_type.clone() {
            self.advance();
            return Ok(Pattern::Literal(Literal::Str(s)));
        }

        let name = self.consume(
            &TokenType::Identifier(String::new()),
            "Expected identifier",
        )?;
        let name_str = match &name.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };

        // 检查是否有解构
        if self.match_token(&TokenType::LeftParen) {
            let mut patterns = Vec::new();
            loop {
                patterns.push(self.parse_pattern()?);
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
            self.consume(&TokenType::RightParen, "Expected ')'")?;
            return Ok(Pattern::Tuple(name_str, patterns));
        }

        if self.match_token(&TokenType::LeftBrace) {
            let mut fields = Vec::new();
            loop {
                let field_name = self.consume(
                    &TokenType::Identifier(String::new()),
                    "Expected field name",
                )?;
                let field_name_str = match &field_name.token_type {
                    TokenType::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };

                let pattern = if self.match_token(&TokenType::Colon) {
                    self.parse_pattern()?
                } else {
                    Pattern::Variable(field_name_str.clone())
                };

                fields.push((field_name_str, pattern));

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
            self.consume(&TokenType::RightBrace, "Expected '}'")?;
            return Ok(Pattern::Struct(name_str, fields));
        }

        Ok(Pattern::Variable(name_str))
    }

    /// 解析表达式
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_assignment()
    }

    /// 解析赋值表达式
    fn parse_assignment(&mut self) -> Result<Expression> {
        let left = self.parse_or()?;
        
        if self.match_token(&TokenType::Equal) {
            let right = self.parse_assignment()?;
            if let Expression::Variable(name) = left {
                return Ok(Expression::Assign(Box::new(name), Box::new(right)));
            }
        }
        
        Ok(left)
    }

    /// 解析逻辑或
    fn parse_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_and()?;
        
        while self.match_token(&TokenType::Or) {
            let right = self.parse_and()?;
            expr = Expression::Binary(Box::new(expr), BinaryOp::Or, Box::new(right));
        }
        
        Ok(expr)
    }

    /// 解析逻辑与
    fn parse_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;
        
        while self.match_token(&TokenType::And) {
            let right = self.parse_equality()?;
            expr = Expression::Binary(Box::new(expr), BinaryOp::And, Box::new(right));
        }
        
        Ok(expr)
    }

    /// 解析相等性
    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;
        
        loop {
            if self.match_token(&TokenType::EqualEqual) {
                let right = self.parse_comparison()?;
                expr = Expression::Binary(Box::new(expr), BinaryOp::Eq, Box::new(right));
            } else if self.match_token(&TokenType::NotEqual) {
                let right = self.parse_comparison()?;
                expr = Expression::Binary(Box::new(expr), BinaryOp::Ne, Box::new(right));
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    /// 解析比较
    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;
        
        loop {
            if self.match_token(&TokenType::Less) {
                let right = self.parse_term()?;
                expr = Expression::Binary(Box::new(expr), BinaryOp::Lt, Box::new(right));
            } else if self.match_token(&TokenType::LessEqual) {
                let right = self.parse_term()?;
                expr = Expression::Binary(Box::new(expr), BinaryOp::Le, Box::new(right));
            } else if self.match_token(&TokenType::Greater) {
                let right = self.parse_term()?;
                expr = Expression::Binary(Box::new(expr), BinaryOp::Gt, Box::new(right));
            } else if self.match_token(&TokenType::GreaterEqual) {
                let right = self.parse_term()?;
                expr = Expression::Binary(Box::new(expr), BinaryOp::Ge, Box::new(right));
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    /// 解析加减法
    fn parse_term(&mut self) -> Result<Expression> {
        let mut expr = self.parse_factor()?;
        
        loop {
            if self.match_token(&TokenType::Plus) {
                let right = self.parse_factor()?;
                expr = Expression::Binary(Box::new(expr), BinaryOp::Add, Box::new(right));
            } else if self.match_token(&TokenType::Minus) {
                let right = self.parse_factor()?;
                expr = Expression::Binary(Box::new(expr), BinaryOp::Sub, Box::new(right));
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    /// 解析乘除法
    fn parse_factor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;
        
        loop {
            if self.match_token(&TokenType::Star) {
                let right = self.parse_unary()?;
                expr = Expression::Binary(Box::new(expr), BinaryOp::Mul, Box::new(right));
            } else if self.match_token(&TokenType::Slash) {
                let right = self.parse_unary()?;
                expr = Expression::Binary(Box::new(expr), BinaryOp::Div, Box::new(right));
            } else if self.match_token(&TokenType::Percent) {
                let right = self.parse_unary()?;
                expr = Expression::Binary(Box::new(expr), BinaryOp::Mod, Box::new(right));
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    /// 解析一元表达式
    fn parse_unary(&mut self) -> Result<Expression> {
        if self.match_token(&TokenType::Minus) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary(UnaryOp::Neg, Box::new(expr)));
        }
        
        if self.match_token(&TokenType::Not) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary(UnaryOp::Not, Box::new(expr)));
        }
        
        self.parse_call()
    }

    /// 解析函数调用和方法调用
    fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.match_token(&TokenType::LeftParen) {
                let mut args = Vec::new();
                
                if !self.check(&TokenType::RightParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }
                
                self.consume(&TokenType::RightParen, "Expected ')'")?;
                expr = Expression::Call(Box::new(expr), args);
            } else if self.match_token(&TokenType::Dot) {
                let field_or_method = self.consume(
                    &TokenType::Identifier(String::new()),
                    "Expected field or method name",
                )?;
                let name = match &field_or_method.token_type {
                    TokenType::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                
                if self.match_token(&TokenType::LeftParen) {
                    // Method call
                    let mut args = Vec::new();
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token(&TokenType::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(&TokenType::RightParen, "Expected ')'")?;
                    expr = Expression::MethodCall(Box::new(expr), name, args);
                } else {
                    // Field access
                    expr = Expression::FieldAccess(Box::new(expr), name);
                }
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    /// 解析基本表达式
    fn parse_primary(&mut self) -> Result<Expression> {
        self.skip_whitespace_and_comments();

        if self.match_token(&TokenType::Keyword(Keyword::True)) {
            return Ok(Expression::Literal(Literal::Bool(true)));
        }

        if self.match_token(&TokenType::Keyword(Keyword::False)) {
            return Ok(Expression::Literal(Literal::Bool(false)));
        }

        if self.match_token(&TokenType::Keyword(Keyword::None)) {
            return Ok(Expression::Literal(Literal::None));
        }

        if let TokenType::Integer(n) = self.peek().token_type.clone() {
            self.advance();
            return Ok(Expression::Literal(Literal::Int(n)));
        }

        if let TokenType::Float(f) = self.peek().token_type.clone() {
            self.advance();
            return Ok(Expression::Literal(Literal::Float(f)));
        }

        if let TokenType::String(s) = self.peek().token_type.clone() {
            self.advance();
            return Ok(Expression::Literal(Literal::Str(s)));
        }

        if let TokenType::Char(c) = self.peek().token_type.clone() {
            self.advance();
            return Ok(Expression::Literal(Literal::Char(c)));
        }

        if self.match_token(&TokenType::LeftParen) {
            let expr = self.parse_expression()?;
            self.consume(&TokenType::RightParen, "Expected ')'")?;
            return Ok(expr);
        }

        if self.match_token(&TokenType::LeftBracket) {
            let mut elements = Vec::new();
            if !self.check(&TokenType::RightBracket) {
                loop {
                    elements.push(self.parse_expression()?);
                    if !self.match_token(&TokenType::Comma) {
                        break;
                    }
                }
            }
            self.consume(&TokenType::RightBracket, "Expected ']'")?;
            return Ok(Expression::Array(elements));
        }

        if self.match_token(&TokenType::LeftBrace) {
            // 可能是匿名结构体或块
            if self.check(&TokenType::RightBrace) {
                self.advance();
                return Ok(Expression::Literal(Literal::Unit));
            }
            
            // 尝试解析为结构体字面量
            let mut fields = Vec::new();
            loop {
                let name = self.consume(
                    &TokenType::Identifier(String::new()),
                    "Expected field name",
                )?;
                let name_str = match &name.token_type {
                    TokenType::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                
                let value = if self.match_token(&TokenType::Colon) {
                    self.parse_expression()?
                } else {
                    Expression::Variable(name_str.clone())
                };
                
                fields.push((name_str, value));
                
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
            self.consume(&TokenType::RightBrace, "Expected '}'")?;
            return Ok(Expression::StructLiteral(fields));
        }

        if self.match_token(&TokenType::Keyword(Keyword::Fn)) {
            return self.parse_lambda();
        }

        let name = self.consume(
            &TokenType::Identifier(String::new()),
            "Expected expression",
        )?;
        let name_str = match &name.token_type {
            TokenType::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };

        Ok(Expression::Variable(name_str))
    }

    /// 解析 Lambda 表达式
    fn parse_lambda(&mut self) -> Result<Expression> {
        self.consume(&TokenType::LeftParen, "Expected '('")?;
        let mut params = Vec::new();
        
        if !self.check(&TokenType::RightParen) {
            loop {
                let name = self.consume(
                    &TokenType::Identifier(String::new()),
                    "Expected parameter name",
                )?;
                let name_str = match &name.token_type {
                    TokenType::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                params.push(name_str);
                
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&TokenType::RightParen, "Expected ')'")?;
        self.consume(&TokenType::Arrow, "Expected '->'")?;
        
        let body = self.parse_expression()?;
        
        Ok(Expression::Lambda(params, Box::new(body)))
    }

    // 以下方法用于解析 struct、enum、trait、impl 等
    // 为简洁起见，这里只实现基本框架
    
    fn parse_struct(&mut self, _visibility: Visibility) -> Result<StructDef> {
        // TODO: 实现 struct 解析
        unimplemented!("Struct parsing not yet implemented")
    }

    fn parse_enum(&mut self, _visibility: Visibility) -> Result<EnumDef> {
        // TODO: 实现 enum 解析
        unimplemented!("Enum parsing not yet implemented")
    }

    fn parse_trait(&mut self, _visibility: Visibility) -> Result<TraitDef> {
        // TODO: 实现 trait 解析
        unimplemented!("Trait parsing not yet implemented")
    }

    fn parse_impl(&mut self) -> Result<ImplBlock> {
        // TODO: 实现 impl 解析
        unimplemented!("Impl parsing not yet implemented")
    }

    fn parse_type_alias(&mut self, _visibility: Visibility) -> Result<TypeAlias> {
        // TODO: 实现 type alias 解析
        unimplemented!("Type alias parsing not yet implemented")
    }

    fn parse_const(&mut self, _visibility: Visibility) -> Result<ConstDef> {
        // TODO: 实现 const 解析
        unimplemented!("Const parsing not yet implemented")
    }

    fn parse_use(&mut self, _visibility: Visibility) -> Result<UseStatement> {
        // TODO: 实现 use 解析
        unimplemented!("Use statement parsing not yet implemented")
    }

    fn parse_module(&mut self, _visibility: Visibility) -> Result<ModuleDef> {
        // TODO: 实现 module 解析
        unimplemented!("Module parsing not yet implemented")
    }
}

/// 语法分析入口函数
pub fn parse(tokens: Vec<Token>) -> Result<Module> {
    let filename = "<input>";
    let parser = Parser::new(tokens, filename);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    #[test]
    fn test_parse_function() {
        let source = r#"
            fn add(a: Int, b: Int) -> Int {
                return a + b;
            }
        "#;
        
        let tokens = lex(source, "test.aether").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn test_parse_let() {
        let source = r#"
            fn main() {
                let x = 42;
                let mut y = 10;
            }
        "#;
        
        let tokens = lex(source, "test.aether").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.items.len(), 1);
    }

    #[test]
    fn test_parse_if() {
        let source = r#"
            fn test() {
                if x > 0 {
                    return 1;
                } else {
                    return 0;
                }
            }
        "#;
        
        let tokens = lex(source, "test.aether").unwrap();
        let ast = parse(tokens).unwrap();
        
        assert!(!ast.items.is_empty());
    }
}
