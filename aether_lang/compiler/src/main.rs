//! Aether 编译器命令行工具

use aether_compiler::{Compiler, CompilerConfig, Error};
use clap::Parser;
use std::path::PathBuf;

/// Aether 编译器
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 输入的 Aether 源文件
    input: PathBuf,

    /// 输出文件路径
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// 优化级别 (0-3)
    #[arg(short = 'O', long, default_value = "2")]
    optimization: u8,

    /// 生成调试信息
    #[arg(long)]
    debug: bool,

    /// 输出格式 (bytecode, llvm-ir, assembly)
    #[arg(long, default_value = "executable")]
    emit: String,

    /// 目标架构 (x86_64, aarch64, wasm32)
    #[arg(long, default_value = "native")]
    target: String,

    /// 详细输出
    #[arg(short, long)]
    verbose: bool,

    /// 显示编译时间
    #[arg(long)]
    time: bool,
}

fn main() {
    let args = Args::parse();

    if args.verbose {
        println!("Aether Compiler v{}", env!("CARGO_PKG_VERSION"));
        println!("Compiling {:?}", args.input);
    }

    let start_time = std::time::Instant::now();

    // 创建配置
    let mut config = match args.optimization {
        0 => CompilerConfig::debug(),
        3 => CompilerConfig::release(),
        _ => CompilerConfig::default(),
    };

    config.optimization_level = args.optimization;
    config.debug_level = if args.debug { 2 } else { 1 };

    // 设置输出格式
    config.output_format = match args.emit.as_str() {
        "llvm-ir" => aether_compiler::config::OutputFormat::LlvmIr,
        "assembly" => aether_compiler::config::OutputFormat::Assembly,
        "bytecode" => aether_compiler::config::OutputFormat::Bytecode,
        _ => aether_compiler::config::OutputFormat::Executable,
    };

    // 设置目标架构
    config.target = match args.target.as_str() {
        "x86_64" => aether_compiler::config::TargetArch::X86_64,
        "aarch64" => aether_compiler::config::TargetArch::AArch64,
        "wasm32" => aether_compiler::config::TargetArch::WebAssembly,
        _ => aether_compiler::config::TargetArch::default(),
    };

    // 创建编译器
    let compiler = Compiler::new(config);

    // 编译
    match compiler.compile_file(&args.input) {
        Ok(bytecode) => {
            let compilation_time = start_time.elapsed();

            if args.time {
                eprintln!("Compilation completed in {:.2?}", compilation_time);
            }

            // 确定输出路径
            let output_path = args.output.unwrap_or_else(|| {
                let mut path = args.input.clone();
                path.set_extension(match config.output_format {
                    aether_compiler::config::OutputFormat::Executable => {
                        if cfg!(windows) { "exe" } else { "" }
                    }
                    aether_compiler::config::OutputFormat::LlvmIr => "ll",
                    aether_compiler::config::OutputFormat::Assembly => "s",
                    aether_compiler::config::OutputFormat::Bytecode => "aethc",
                    aether_compiler::config::OutputFormat::WebAssembly => "wasm",
                });
                path
            });

            // 写入输出文件
            match std::fs::write(&output_path, &bytecode) {
                Ok(_) => {
                    if args.verbose {
                        println!("Successfully compiled {:?} to {:?}", args.input, output_path);
                        println!("Output size: {} bytes", bytecode.len());
                    }
                }
                Err(e) => {
                    eprintln!("Error writing output file: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            print_error_details(&e);
            std::process::exit(1);
        }
    }
}

fn print_error_details(error: &Error) {
    eprintln!("\nError details:");
    eprintln!("  Location: {}", error.span);
    eprintln!("  Type: {:?}", error.error);
    
    // TODO: 显示源代码上下文
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(["aetherc", "test.aether"]);
        assert_eq!(args.input, PathBuf::from("test.aether"));
    }
}
