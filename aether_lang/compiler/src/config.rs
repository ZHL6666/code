//! 编译器配置模块

/// 编译器配置选项
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// 优化级别 (0-3)
    pub optimization_level: u8,
    
    /// 调试信息级别 (0-2)
    pub debug_level: u8,
    
    /// 目标架构
    pub target: TargetArch,
    
    /// 输出格式
    pub output_format: OutputFormat,
    
    /// 是否进行借用检查
    pub borrow_check: bool,
    
    /// 是否进行类型推断
    pub type_inference: bool,
    
    /// 最大内联大小
    pub max_inline_size: usize,
    
    /// 是否启用 SIMD 优化
    pub enable_simd: bool,
    
    /// 是否启用并行编译
    pub parallel_compilation: bool,
}

/// 目标架构
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetArch {
    X86_64,
    AArch64,
    WebAssembly,
    RiscV64,
}

impl Default for TargetArch {
    fn default() -> Self {
        #[cfg(target_arch = "x86_64")]
        return TargetArch::X86_64;
        
        #[cfg(target_arch = "aarch64")]
        return TargetArch::AArch64;
        
        #[cfg(target_arch = "wasm32")]
        return TargetArch::WebAssembly;
        
        #[cfg(target_arch = "riscv64")]
        return TargetArch::RiscV64;
        
        #[allow(unreachable_code)]
        TargetArch::X86_64
    }
}

/// 输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// 原生可执行文件
    Executable,
    
    /// LLVM IR
    LlvmIr,
    
    /// WebAssembly
    WebAssembly,
    
    /// 字节码
    Bytecode,
    
    /// 汇编代码
    Assembly,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Executable
    }
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            optimization_level: 2,
            debug_level: 1,
            target: TargetArch::default(),
            output_format: OutputFormat::default(),
            borrow_check: true,
            type_inference: true,
            max_inline_size: 100,
            enable_simd: true,
            parallel_compilation: true,
        }
    }
}

impl CompilerConfig {
    /// 创建发布构建配置
    pub fn release() -> Self {
        Self {
            optimization_level: 3,
            debug_level: 0,
            ..Default::default()
        }
    }

    /// 创建调试构建配置
    pub fn debug() -> Self {
        Self {
            optimization_level: 0,
            debug_level: 2,
            ..Default::default()
        }
    }

    /// 创建 WebAssembly 配置
    pub fn wasm() -> Self {
        Self {
            target: TargetArch::WebAssembly,
            output_format: OutputFormat::WebAssembly,
            ..Default::default()
        }
    }

    /// 是否启用优化
    pub fn is_optimized(&self) -> bool {
        self.optimization_level > 0
    }

    /// 是否生成调试信息
    pub fn has_debug_info(&self) -> bool {
        self.debug_level > 0
    }
}
