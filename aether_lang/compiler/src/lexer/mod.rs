//! Aether 词法分析器模块
//! 
//! 将源代码转换为 Token 流

use crate::error::{Error, Result};
use std::collections::HashMap;

/// Token 类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // 字面量
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    
    // 标识符和关键字
    Identifier(String),
    Keyword(Keyword),
    
    // 运算符
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    PlusPlus,       // ++
    MinusMinus,     // --
    PlusEqual,      // +=
    MinusEqual,     // -=
    StarEqual,      // *=
    SlashEqual,     // /=
    Equal,          // =
    EqualEqual,     // ==
    NotEqual,       // !=
    Less,           // <
    LessEqual,      // <=
    Greater,        // >
    GreaterEqual,   // >=
    And,            // &&
    Or,             // ||
    Not,            // !
    BitAnd,         // &
    BitOr,          // |
    BitXor,         // ^
    BitNot,         // ~
    ShiftLeft,      // <<
    ShiftRight,     // >>
    
    // 分隔符
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    Comma,          // ,
    Dot,            // .
    Semicolon,      // ;
    Colon,          // :
    DoubleColon,    // ::
    Arrow,          // ->
    FatArrow,       // =>
    Question,       // ?
    At,             // @
    
    // 特殊
    Eof,
    Newline,
    Comment(String),
    Whitespace(String),
}

/// 关键字枚举
#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    // 控制流
    If,
    Else,
    Match,
    While,
    For,
    Loop,
    Break,
    Continue,
    Return,
    Yield,
    
    // 函数和类型
    Fn,
    Async,
    Await,
    Let,
    Const,
    Mut,
    Type,
    Struct,
    Enum,
    Union,
    Trait,
    Impl,
    Where,
    
    // 模式匹配
    Ref,
    Box,
    Move,
    
    // 可见性
    Pub,
    Priv,
    
    // 其他
    Use,
    Mod,
    Super,
    Self_,
    Static,
    Unsafe,
    Extern,
    Crate,
    
    // 特殊值
    True,
    False,
    None,
    
    // 并发
    Spawn,
    Channel,
    Select,
    
    // 内存管理
    Drop,
    Clone,
    Copy,
}

impl Keyword {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "if" => Some(Keyword::If),
            "else" => Some(Keyword::Else),
            "match" => Some(Keyword::Match),
            "while" => Some(Keyword::While),
            "for" => Some(Keyword::For),
            "loop" => Some(Keyword::Loop),
            "break" => Some(Keyword::Break),
            "continue" => Some(Keyword::Continue),
            "return" => Some(Keyword::Return),
            "yield" => Some(Keyword::Yield),
            "fn" => Some(Keyword::Fn),
            "async" => Some(Keyword::Async),
            "await" => Some(Keyword::Await),
            "let" => Some(Keyword::Let),
            "const" => Some(Keyword::Const),
            "mut" => Some(Keyword::Mut),
            "type" => Some(Keyword::Type),
            "struct" => Some(Keyword::Struct),
            "enum" => Some(Keyword::Enum),
            "union" => Some(Keyword::Union),
            "trait" => Some(Keyword::Trait),
            "impl" => Some(Keyword::Impl),
            "where" => Some(Keyword::Where),
            "ref" => Some(Keyword::Ref),
            "box" => Some(Keyword::Box),
            "move" => Some(Keyword::Move),
            "pub" => Some(Keyword::Pub),
            "priv" => Some(Keyword::Priv),
            "use" => Some(Keyword::Use),
            "mod" => Some(Keyword::Mod),
            "super" => Some(Keyword::Super),
            "self" => Some(Keyword::Self_),
            "static" => Some(Keyword::Static),
            "unsafe" => Some(Keyword::Unsafe),
            "extern" => Some(Keyword::Extern),
            "crate" => Some(Keyword::Crate),
            "true" => Some(Keyword::True),
            "false" => Some(Keyword::False),
            "none" => Some(Keyword::None),
            "spawn" => Some(Keyword::Spawn),
            "channel" => Some(Keyword::Channel),
            "select" => Some(Keyword::Select),
            "drop" => Some(Keyword::Drop),
            "clone" => Some(Keyword::Clone),
            "copy" => Some(Keyword::Copy),
            _ => None,
        }
    }
}

/// Token 结构
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize, length: usize) -> Self {
        Self {
            token_type,
            line,
            column,
            length,
        }
    }
}

/// 词法分析器
pub struct Lexer {
    source: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    filename: String,
}

impl Lexer {
    /// 创建新的词法分析器
    pub fn new(source: &str, filename: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            filename: filename.to_string(),
        }
    }

    /// 执行词法分析，返回 Token 流
    pub fn lex(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            if let Some(token) = self.scan_token()? {
                tokens.push(token);
            }
        }

        tokens.push(Token::new(TokenType::Eof, self.line, self.column, 0));
        Ok(tokens)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.source[self.position];
        self.position += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.position]
        }
    }

    fn peek_next(&self) -> char {
        if self.position + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.position + 1]
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    fn scan_token(&mut self) -> Result<Option<Token>> {
        let start_line = self.line;
        let start_column = self.column;
        
        let ch = self.advance();

        match ch {
            // 空白字符
            ' ' | '\t' | '\r' => {
                let mut text = ch.to_string();
                while !self.is_at_end() && matches!(self.peek(), ' ' | '\t' | '\r') {
                    text.push(self.advance());
                }
                return Ok(Some(Token::new(
                    TokenType::Whitespace(text),
                    start_line,
                    start_column,
                    text.len(),
                )));
            }

            // 换行符
            '\n' => Ok(Some(Token::new(TokenType::Newline, start_line, start_column, 1))),

            // 注释
            '/' if self.match_char('/') => self.scan_line_comment(start_line, start_column),
            '/' if self.match_char('*') => self.scan_block_comment(start_line, start_column),

            // 字符串
            '"' => self.scan_string(start_line, start_column),
            '\'' => self.scan_char(start_line, start_column),

            // 数字
            c if c.is_ascii_digit() => self.scan_number(start_line, start_column, c),

            // 标识符或关键字
            c if c.is_alphabetic() || c == '_' => self.scan_identifier(start_line, start_column, c),

            // 运算符和分隔符
            '(' => Ok(Some(Token::new(TokenType::LeftParen, start_line, start_column, 1))),
            ')' => Ok(Some(Token::new(TokenType::RightParen, start_line, start_column, 1))),
            '{' => Ok(Some(Token::new(TokenType::LeftBrace, start_line, start_column, 1))),
            '}' => Ok(Some(Token::new(TokenType::RightBrace, start_line, start_column, 1))),
            '[' => Ok(Some(Token::new(TokenType::LeftBracket, start_line, start_column, 1))),
            ']' => Ok(Some(Token::new(TokenType::RightBracket, start_line, start_column, 1))),
            ',' => Ok(Some(Token::new(TokenType::Comma, start_line, start_column, 1))),
            ';' => Ok(Some(Token::new(TokenType::Semicolon, start_line, start_column, 1))),
            ':' if self.match_char(':') => Ok(Some(Token::new(TokenType::DoubleColon, start_line, start_column, 2))),
            ':' => Ok(Some(Token::new(TokenType::Colon, start_line, start_column, 1))),
            '.' => Ok(Some(Token::new(TokenType::Dot, start_line, start_column, 1))),
            '?' => Ok(Some(Token::new(TokenType::Question, start_line, start_column, 1))),
            '@' => Ok(Some(Token::new(TokenType::At, start_line, start_column, 1))),
            
            '+' if self.match_char('+') => Ok(Some(Token::new(TokenType::PlusPlus, start_line, start_column, 2))),
            '+' if self.match_char('=') => Ok(Some(Token::new(TokenType::PlusEqual, start_line, start_column, 2))),
            '+' => Ok(Some(Token::new(TokenType::Plus, start_line, start_column, 1))),
            
            '-' if self.match_char('-') => Ok(Some(Token::new(TokenType::MinusMinus, start_line, start_column, 2))),
            '-' if self.match_char('=') => Ok(Some(Token::new(TokenType::MinusEqual, start_line, start_column, 2))),
            '-' if self.match_char('>') => Ok(Some(Token::new(TokenType::Arrow, start_line, start_column, 2))),
            '-' => Ok(Some(Token::new(TokenType::Minus, start_line, start_column, 1))),
            
            '*' if self.match_char('=') => Ok(Some(Token::new(TokenType::StarEqual, start_line, start_column, 2))),
            '*' => Ok(Some(Token::new(TokenType::Star, start_line, start_column, 1))),
            
            '/' if self.match_char('=') => Ok(Some(Token::new(TokenType::SlashEqual, start_line, start_column, 2))),
            '/' => Ok(Some(Token::new(TokenType::Slash, start_line, start_column, 1))),
            
            '%' => Ok(Some(Token::new(TokenType::Percent, start_line, start_column, 1))),
            
            '=' if self.match_char('=') => Ok(Some(Token::new(TokenType::EqualEqual, start_line, start_column, 2))),
            '=' if self.match_char('>') => Ok(Some(Token::new(TokenType::FatArrow, start_line, start_column, 2))),
            '=' => Ok(Some(Token::new(TokenType::Equal, start_line, start_column, 1))),
            
            '!' if self.match_char('=') => Ok(Some(Token::new(TokenType::NotEqual, start_line, start_column, 2))),
            '!' => Ok(Some(Token::new(TokenType::Not, start_line, start_column, 1))),
            
            '<' if self.match_char('=') => Ok(Some(Token::new(TokenType::LessEqual, start_line, start_column, 2))),
            '<' if self.match_char('<') => Ok(Some(Token::new(TokenType::ShiftLeft, start_line, start_column, 2))),
            '<' => Ok(Some(Token::new(TokenType::Less, start_line, start_column, 1))),
            
            '>' if self.match_char('=') => Ok(Some(Token::new(TokenType::GreaterEqual, start_line, start_column, 2))),
            '>' if self.match_char('>') => Ok(Some(Token::new(TokenType::ShiftRight, start_line, start_column, 2))),
            '>' => Ok(Some(Token::new(TokenType::Greater, start_line, start_column, 1))),
            
            '&' if self.match_char('&') => Ok(Some(Token::new(TokenType::And, start_line, start_column, 2))),
            '&' => Ok(Some(Token::new(TokenType::BitAnd, start_line, start_column, 1))),
            
            '|' if self.match_char('|') => Ok(Some(Token::new(TokenType::Or, start_line, start_column, 2))),
            '|' => Ok(Some(Token::new(TokenType::BitOr, start_line, start_column, 1))),
            
            '^' => Ok(Some(Token::new(TokenType::BitXor, start_line, start_column, 1))),
            '~' => Ok(Some(Token::new(TokenType::BitNot, start_line, start_column, 1))),

            // 未知字符
            _ => Err(Error::lexical(
                format!("Unexpected character: '{}'", ch),
                start_line,
                start_column,
                &self.filename,
            )),
        }
    }

    fn scan_line_comment(&mut self, start_line: usize, start_column: usize) -> Result<Option<Token>> {
        let mut text = String::from("//");
        while !self.is_at_end() && self.peek() != '\n' {
            text.push(self.advance());
        }
        Ok(Some(Token::new(TokenType::Comment(text), start_line, start_column, text.len())))
    }

    fn scan_block_comment(&mut self, start_line: usize, start_column: usize) -> Result<Option<Token>> {
        let mut text = String::from("/*");
        let mut depth = 1;
        
        while !self.is_at_end() && depth > 0 {
            let ch = self.advance();
            text.push(ch);
            
            if ch == '/' && self.peek() == '*' {
                self.advance();
                text.push('*');
                depth += 1;
            } else if ch == '*' && self.peek() == '/' {
                self.advance();
                text.push('/');
                depth -= 1;
            }
        }
        
        if depth > 0 {
            return Err(Error::lexical(
                "Unterminated block comment".to_string(),
                start_line,
                start_column,
                &self.filename,
            ));
        }
        
        Ok(Some(Token::new(TokenType::Comment(text), start_line, start_column, text.len())))
    }

    fn scan_string(&mut self, start_line: usize, start_column: usize) -> Result<Option<Token>> {
        let mut value = String::new();
        
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                return Err(Error::lexical(
                    "Unterminated string literal".to_string(),
                    start_line,
                    start_column,
                    &self.filename,
                ));
            }
            
            if self.peek() == '\\' {
                self.advance(); // consume backslash
                if self.is_at_end() {
                    return Err(Error::lexical(
                        "Unterminated escape sequence".to_string(),
                        start_line,
                        start_column,
                        &self.filename,
                    ));
                }
                let escape = self.advance();
                match escape {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    '0' => value.push('\0'),
                    _ => {
                        return Err(Error::lexical(
                            format!("Invalid escape sequence: '\\{}'", escape),
                            self.line,
                            self.column,
                            &self.filename,
                        ));
                    }
                }
            } else {
                value.push(self.advance());
            }
        }
        
        if self.is_at_end() {
            return Err(Error::lexical(
                "Unterminated string literal".to_string(),
                start_line,
                start_column,
                &self.filename,
            ));
        }
        
        self.advance(); // consume closing quote
        
        let length = value.len() + 2; // +2 for quotes
        Ok(Some(Token::new(TokenType::String(value), start_line, start_column, length)))
    }

    fn scan_char(&mut self, start_line: usize, start_column: usize) -> Result<Option<Token>> {
        if self.is_at_end() {
            return Err(Error::lexical(
                "Unterminated character literal".to_string(),
                start_line,
                start_column,
                &self.filename,
            ));
        }
        
        let ch = if self.peek() == '\\' {
            self.advance(); // consume backslash
            if self.is_at_end() {
                return Err(Error::lexical(
                    "Unterminated escape sequence".to_string(),
                    start_line,
                    start_column,
                    &self.filename,
                ));
            }
            let escape = self.advance();
            match escape {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                '0' => '\0',
                _ => {
                    return Err(Error::lexical(
                        format!("Invalid escape sequence: '\\{}'", escape),
                        self.line,
                        self.column,
                        &self.filename,
                    ));
                }
            }
        } else {
            self.advance()
        };
        
        if self.is_at_end() || self.peek() != '\'' {
            return Err(Error::lexical(
                "Unterminated character literal".to_string(),
                start_line,
                start_column,
                &self.filename,
            ));
        }
        
        self.advance(); // consume closing quote
        
        Ok(Some(Token::new(TokenType::Char(ch), start_line, start_column, 3)))
    }

    fn scan_number(&mut self, start_line: usize, start_column: usize, first_digit: char) -> Result<Option<Token>> {
        let mut text = first_digit.to_string();
        let mut is_float = false;
        
        // 处理十六进制、八进制、二进制
        if first_digit == '0' && !self.is_at_end() {
            match self.peek() {
                'x' | 'X' => {
                    self.advance();
                    text.push('x');
                    while !self.is_at_end() && self.peek().is_ascii_hexdigit() {
                        text.push(self.advance());
                    }
                    let value = i64::from_str_radix(&text[2..], 16).map_err(|_| {
                        Error::lexical("Invalid hexadecimal number".to_string(), start_line, start_column, &self.filename)
                    })?;
                    return Ok(Some(Token::new(TokenType::Integer(value), start_line, start_column, text.len())));
                }
                'o' | 'O' => {
                    self.advance();
                    text.push('o');
                    while !self.is_at_end() && matches!(self.peek(), '0'..='7') {
                        text.push(self.advance());
                    }
                    let value = i64::from_str_radix(&text[2..], 8).map_err(|_| {
                        Error::lexical("Invalid octal number".to_string(), start_line, start_column, &self.filename)
                    })?;
                    return Ok(Some(Token::new(TokenType::Integer(value), start_line, start_column, text.len())));
                }
                'b' | 'B' => {
                    self.advance();
                    text.push('b');
                    while !self.is_at_end() && matches!(self.peek(), '0' | '1') {
                        text.push(self.advance());
                    }
                    let value = i64::from_str_radix(&text[2..], 2).map_err(|_| {
                        Error::lexical("Invalid binary number".to_string(), start_line, start_column, &self.filename)
                    })?;
                    return Ok(Some(Token::new(TokenType::Integer(value), start_line, start_column, text.len())));
                }
                _ => {}
            }
        }
        
        // 处理十进制整数或浮点数
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            text.push(self.advance());
        }
        
        // 检查小数部分
        if !self.is_at_end() && self.peek() == '.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            text.push(self.advance()); // consume '.'
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                text.push(self.advance());
            }
        }
        
        // 检查指数部分
        if !self.is_at_end() && matches!(self.peek(), 'e' | 'E') {
            is_float = true;
            text.push(self.advance());
            if !self.is_at_end() && matches!(self.peek(), '+' | '-') {
                text.push(self.advance());
            }
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                text.push(self.advance());
            }
        }
        
        if is_float {
            let value = text.parse::<f64>().map_err(|_| {
                Error::lexical("Invalid floating-point number".to_string(), start_line, start_column, &self.filename)
            })?;
            Ok(Some(Token::new(TokenType::Float(value), start_line, start_column, text.len())))
        } else {
            // 移除可能的类型后缀
            let num_str = text.trim_end_matches(|c: char| c.is_alphabetic());
            let value = num_str.parse::<i64>().map_err(|_| {
                Error::lexical("Invalid integer number".to_string(), start_line, start_column, &self.filename)
            })?;
            Ok(Some(Token::new(TokenType::Integer(value), start_line, start_column, text.len())))
        }
    }

    fn scan_identifier(&mut self, start_line: usize, start_column: usize, first_char: char) -> Result<Option<Token>> {
        let mut text = first_char.to_string();
        
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            text.push(self.advance());
        }
        
        let token_type = if let Some(keyword) = Keyword::from_string(&text) {
            TokenType::Keyword(keyword)
        } else {
            TokenType::Identifier(text)
        };
        
        Ok(Some(Token::new(token_type, start_line, start_column, text.len())))
    }
}

/// 词法分析入口函数
pub fn lex(source: &str, filename: &str) -> Result<Vec<Token>> {
    let lexer = Lexer::new(source, filename);
    lexer.lex()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let source = "let x = 42;";
        let tokens = lex(source, "test.aether").unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Keyword(Keyword::Let));
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Equal);
        assert_eq!(tokens[3].token_type, TokenType::Integer(42));
        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
        assert_eq!(tokens[5].token_type, TokenType::Eof);
    }

    #[test]
    fn test_string_literal() {
        let source = r#""Hello, World!""#;
        let tokens = lex(source, "test.aether").unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::String("Hello, World!".to_string()));
    }

    #[test]
    fn test_comments() {
        let source = "// This is a comment\nlet x = 1; /* block comment */";
        let tokens = lex(source, "test.aether").unwrap();
        
        // Comments should be preserved
        assert!(tokens.iter().any(|t| matches!(t.token_type, TokenType::Comment(_))));
    }
}
