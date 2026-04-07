//! Aether 编译器库
//! 
//! 提供完整的 Aether 编程语言编译功能，包括：
//! - 词法分析 (lexer)
//! - 语法分析 (parser)
//! - 语义分析 (sema)
//! - 中间代码生成 (ir)
//! - 优化 (optimizer)
//! - 代码生成 (codegen)

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod sema;
pub mod ir;
pub mod optimizer;
pub mod codegen;
pub mod runtime;

mod error;
mod config;

pub use error::{Error, Result, CompilerError};
pub use config::CompilerConfig;

use std::path::Path;
use std::sync::Arc;

/// 编译器主结构
pub struct Compiler {
    config: CompilerConfig,
}

impl Compiler {
    /// 创建新的编译器实例
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// 从文件编译 Aether 源代码
    pub fn compile_file(&self, path: &Path) -> Result<Vec<u8>> {
        let source = std::fs::read_to_string(path)?;
        self.compile_source(&source, path.to_str().unwrap_or("<unknown>"))
    }

    /// 从字符串编译 Aether 源代码
    pub fn compile_source(&self, source: &str, filename: &str) -> Result<Vec<u8>> {
        // 阶段 1: 词法分析
        let tokens = lexer::lex(source, filename)?;
        
        // 阶段 2: 语法分析
        let ast = parser::parse(tokens)?;
        
        // 阶段 3: 语义分析
        let typed_ast = sema::analyze(ast)?;
        
        // 阶段 4: 生成中间表示
        let ir = ir::generate(typed_ast)?;
        
        // 阶段 5: 优化
        let optimized_ir = optimizer::optimize(ir, &self.config)?;
        
        // 阶段 6: 代码生成
        let bytecode = codegen::generate(optimized_ir, &self.config)?;
        
        Ok(bytecode)
    }

    /// 编译并输出到文件
    pub fn compile_to_file(&self, input: &Path, output: &Path) -> Result<()> {
        let bytecode = self.compile_file(input)?;
        std::fs::write(output, bytecode)?;
        Ok(())
    }
}

/// 便捷编译函数
pub fn compile(source: &str, filename: &str) -> Result<Vec<u8>> {
    let config = CompilerConfig::default();
    let compiler = Compiler::new(config);
    compiler.compile_source(source, filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let source = r#"
            fn main() {
                println("Hello, World!");
            }
        "#;
        
        let result = compile(source, "test.aether");
        assert!(result.is_ok());
    }

    #[test]
    fn test_fibonacci() {
        let source = r#"
            fn fib(n: Int) -> Int {
                if n <= 1 {
                    return n;
                }
                return fib(n - 1) + fib(n - 2);
            }
            
            fn main() {
                println(fib(10));
            }
        "#;
        
        let result = compile(source, "fib.aether");
        assert!(result.is_ok());
    }
}
