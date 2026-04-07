//! 编译器错误处理模块

use thiserror::Error;
use std::fmt;

/// 编译器错误类型
#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("词法错误：{message}")]
    LexicalError {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("语法错误：{message}")]
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
        token: String,
    },

    #[error("语义错误：{message}")]
    SemanticError {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("类型错误：{message}")]
    TypeError {
        message: String,
        line: usize,
        column: usize,
        expected: String,
        found: String,
    },

    #[error("借用检查错误：{message}")]
    BorrowError {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("代码生成错误：{message}")]
    CodeGenError {
        message: String,
    },

    #[error("IO 错误：{0}")]
    IoError(#[from] std::io::Error),

    #[error("内部编译器错误：{0}")]
    InternalError(String),
}

/// 错误位置信息
#[derive(Debug, Clone)]
pub struct Span {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub file: String,
}

impl Span {
    pub fn new(start_line: usize, start_column: usize, end_line: usize, end_column: usize, file: &str) -> Self {
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
            file: file.to_string(),
        }
    }

    pub fn point(line: usize, column: usize, file: &str) -> Self {
        Self::new(line, column, line, column, file)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}-{}:{}",
            self.file,
            self.start_line,
            self.start_column,
            self.end_line,
            self.end_column
        )
    }
}

/// 带位置的错误
#[derive(Debug)]
pub struct Error {
    pub error: CompilerError,
    pub span: Span,
}

impl Error {
    pub fn lexical(message: String, line: usize, column: usize, file: &str) -> Self {
        Self {
            error: CompilerError::LexicalError { message, line, column },
            span: Span::point(line, column, file),
        }
    }

    pub fn syntax(message: String, line: usize, column: usize, token: String, file: &str) -> Self {
        Self {
            error: CompilerError::SyntaxError { message, line, column, token },
            span: Span::point(line, column, file),
        }
    }

    pub fn semantic(message: String, line: usize, column: usize, file: &str) -> Self {
        Self {
            error: CompilerError::SemanticError { message, line, column },
            span: Span::point(line, column, file),
        }
    }

    pub fn type_error(
        message: String,
        line: usize,
        column: usize,
        expected: String,
        found: String,
        file: &str,
    ) -> Self {
        Self {
            error: CompilerError::TypeError {
                message,
                line,
                column,
                expected,
                found,
            },
            span: Span::point(line, column, file),
        }
    }

    pub fn borrow(message: String, line: usize, column: usize, file: &str) -> Self {
        Self {
            error: CompilerError::BorrowError { message, line, column },
            span: Span::point(line, column, file),
        }
    }

    pub fn codegen(message: String) -> Self {
        Self {
            error: CompilerError::CodeGenError { message },
            span: Span::point(0, 0, "<codegen>"),
        }
    }

    pub fn internal(message: String) -> Self {
        Self {
            error: CompilerError::InternalError(message),
            span: Span::point(0, 0, "<internal>"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.error, self.span)
    }
}

impl std::error::Error for Error {}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, Error>;

/// 错误报告器
pub struct ErrorReporter {
    errors: Vec<Error>,
    warnings: Vec<Error>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn error(&mut self, error: Error) {
        self.errors.push(error);
    }

    pub fn warning(&mut self, warning: Error) {
        self.warnings.push(warning);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn emit(&self) -> Result<()> {
        if !self.warnings.is_empty() {
            eprintln!("Warnings:");
            for warning in &self.warnings {
                eprintln!("  {}", warning);
            }
        }

        if !self.errors.is_empty() {
            eprintln!("Errors:");
            for error in &self.errors {
                eprintln!("  {}", error);
            }
            Err(self.errors[0].clone())
        } else {
            Ok(())
        }
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}
