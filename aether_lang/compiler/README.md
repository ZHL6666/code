# Aether 编译器

一个完整的 Aether 编程语言编译器实现，包含词法分析、语法分析、语义分析、中间代码生成、优化和代码生成。

## 目录结构

```
compiler/
├── src/
│   ├── lexer/          # 词法分析器
│   ├── parser/         # 语法分析器
│   ├── ast/            # 抽象语法树
│   ├── sema/           # 语义分析
│   ├── ir/             # 中间表示
│   ├── optimizer/      # 优化器
│   ├── codegen/        # 代码生成
│   └── runtime/        # 运行时库
├── tests/              # 测试用例
├── examples/           # 示例程序
└── build.rs            # 构建脚本
```

## 功能特性

- **词法分析**: 支持所有 Aether 语法元素的词法标记
- **语法分析**: 递归下降解析器，完整的语法树构建
- **语义分析**: 类型检查、作用域管理、借用检查
- **中间表示**: 三地址码形式的 IR
- **优化器**: 常量折叠、死代码消除、内联优化
- **代码生成**: LLVM IR 生成，支持 x86_64 和 WebAssembly

## 快速开始

```bash
# 编译编译器
cargo build --release

# 编译 Aether 程序
./target/release/aetherc hello.aether -o hello

# 运行程序
./hello

# 查看生成的 LLVM IR
./target/release/aetherc hello.aether --emit=llvm-ir
```

## 架构设计

编译器采用多阶段管道架构：

1. **Lexer**: 源代码 → Token 流
2. **Parser**: Token 流 → AST
3. **Sema**: AST → 类型标注的 AST
4. **IR Gen**: 类型标注的 AST → IR
5. **Optimizer**: IR → 优化后的 IR
6. **Codegen**: 优化后的 IR → 目标代码

## 许可证

MIT License
