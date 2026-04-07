# Aether 项目结构总览

```
aether_lang/
│
├── README.md                          # 项目介绍和快速开始
├── specification.md                   # 完整语言规范 (EBNF 语法)
│
├── examples/                          # 示例代码
│   ├── hello.aether                   # Hello World
│   ├── fibonacci.aether               # 斐波那契数列（4 种实现）
│   ├── web_server.aether              # 异步 Web 服务器
│   └── data_pipeline.aether           # 函数式数据处理
│
├── src/                               # 编译器源代码目录
│   ├── core/                          # 核心模块
│   │   ├── types.ae                   # 基础类型系统
│   │   ├── collections.ae             # 集合库
│   │   ├── async_runtime.ae           # 异步运行时
│   │   └── io.ae                      # IO 系统
│   │
│   ├── lexer/                         # 词法分析器
│   │   ├── tokenizer.rs               # Token 生成
│   │   ├── unicode.rs                 # Unicode 处理
│   │   └── diagnostics.rs             # 错误诊断
│   │
│   ├── parser/                        # 语法分析器
│   │   ├── expr.rs                    # 表达式解析
│   │   ├── stmt.rs                    # 语句解析
│   │   ├── types.rs                   # 类型解析
│   │   └── recovery.rs                # 错误恢复
│   │
│   ├── ast/                           # 抽象语法树
│   │   ├── nodes.rs                   # AST 节点定义
│   │   ├── visitor.rs                 # Visitor 模式
│   │   └── pretty.rs                  # AST 美化打印
│   │
│   ├── hir/                           # 高级中间表示
│   │   ├── lowering.rs                # AST -> HIR
│   │   ├── resolve.rs                 # 名称解析
│   │   └── typecheck.rs               # 类型检查
│   │
│   ├── mir/                           # 中级中间表示
│   │   ├── build.rs                   # HIR -> MIR
│   │   ├── optimize.rs                # MIR 优化
│   │   └── borrow_check.rs            # 借用检查
│   │
│   ├── lir/                           # 低级中间表示
│   │   ├── lower.rs                   # MIR -> LIR
│   │   ├── inline.rs                  # 内联优化
│   │   └── specialize.rs              # 特化
│   │
│   ├── codegen/                       # 代码生成
│   │   ├── llvm/                      # LLVM 后端
│   │   │   ├── emit.rs                # LLVM IR 生成
│   │   │   ├── intrinsics.rs          # 内部函数
│   │   │   └── debug.rs               # 调试信息
│   │   ├── wasm/                      # WebAssembly 后端
│   │   └── c/                         # C 后端 (FFI)
│   │
│   ├── optimizer/                     # 优化器
│   │   ├── inline.rs                  # 内联
│   │   ├── loop.rs                    # 循环优化
│   │   ├── simd.rs                    # SIMD 向量化
│   │   └── parallel.rs                # 并行化
│   │
│   ├── linker/                        # 链接器
│   │   ├── resolve.rs                 # 符号解析
│   │   ├── layout.rs                  # 内存布局
│   │   └── emit.rs                    # 输出二进制
│   │
│   └── util/                          # 工具库
│       ├── arena.rs                   # Arena 分配器
│       ├── graph.rs                   # 图数据结构
│       └── hash.rs                    # 快速哈希
│
├── libs/                              # 标准库
│   ├── standard_library.md            # 标准库文档
│   ├── algorithms.ae                  # 数据结构与算法
│   ├── ai_helpers.ae                  # AI 辅助编程
│   ├── test.ae                        # 测试框架
│   ├── verification.ae                # 形式验证
│   ├── platform.ae                    # 跨平台支持
│   ├── wasm.ae                        # WebAssembly
│   ├── gpu.ae                         # GPU 计算
│   ├── db.ae                          # 数据库 ORM
│   └── ml.ae                          # 机器学习
│
├── tools/                             # 开发工具
│   │
│   ├── compiler/                      # 编译器
│   │   ├── architecture.md            # 编译器架构文档
│   │   ├── Cargo.toml                 # 构建配置
│   │   └── src/                       # 编译器源码（见 src/ 目录）
│   │
│   ├── lsp/                           # 语言服务器
│   │   ├── specification.md           # LSP 功能规格
│   │   ├── main.rs                    # LSP 入口
│   │   ├── server.rs                  # 协议处理
│   │   ├── capabilities.rs            # 能力声明
│   │   ├── handlers/                  # 请求处理器
│   │   │   ├── completion.rs          # 代码补全
│   │   │   ├── hover.rs               # 悬停提示
│   │   │   ├── goto_def.rs            # 跳转定义
│   │   │   ├── find_refs.rs           # 查找引用
│   │   │   ├── rename.rs              # 重命名
│   │   │   ├── diagnostics.rs         # 错误诊断
│   │   │   └── formatting.rs          # 代码格式化
│   │   ├── analysis/                  # 语言分析
│   │   │   ├── indexer.rs             # 符号索引
│   │   │   ├── typer.rs               # 类型查询
│   │   │   ├── inference.rs           # 类型推断
│   │   │   └── cache.rs               # 分析缓存
│   │   ├── vfs/                       # 虚拟文件系统
│   │   │   ├── overlay.rs             # 内存文件覆盖
│   │   │   └── watcher.rs             # 文件监听
│   │   └── utils/                     # 工具
│   │       ├── fuzzy.rs               # 模糊匹配
│   │       └── markup.rs              # Markdown 生成
│   │
│   ├── package_manager/               # 包管理器
│   │   ├── main.rs                    # 入口
│   │   ├── resolver.rs                # 依赖解析 (PubGrub)
│   │   ├── registry.rs                # 包注册表
│   │   └── lockfile.rs                # 锁定文件
│   │
│   ├── profiler/                      # 性能分析器
│   │   ├── sampler.rs                 # 采样分析
│   │   ├── memory.rs                  # 内存分析
│   │   └── flamegraph.rs              # 火焰图生成
│   │
│   ├── doc_generator/                 # 文档生成器
│   │   ├── main.rs                    # 入口
│   │   ├── parser.rs                  # 文档注释解析
│   │   ├── markdown.rs                # Markdown 生成
│   │   └── index.rs                   # 索引构建
│   │
│   └── formatter/                     # 代码格式化器
│       ├── main.rs                    # 入口
│       ├── rules.rs                   # 格式化规则
│       └── config.rs                  # 配置文件
│
├── tests/                             # 测试套件
│   ├── unit/                          # 单元测试
│   │   ├── lexer_tests.rs
│   │   ├── parser_tests.rs
│   │   ├── typecheck_tests.rs
│   │   └── borrow_check_tests.rs
│   │
│   ├── integration/                   # 集成测试
│   │   ├── compilation_tests.rs
│   │   └── optimization_tests.rs
│   │
│   ├── suite/                         # 测试用例
│   │   ├── syntax/                    # 语法测试
│   │   ├── types/                     # 类型系统测试
│   │   ├── concurrency/               # 并发测试
│   │   └── performance/               # 性能测试
│   │
│   └── fixtures/                      # 测试固件
│       ├── valid/                     # 有效程序
│       └── invalid/                   # 无效程序（应报错）
│
├── docs/                              # 文档
│   ├── guide/                         # 用户指南
│   │   ├── getting_started.md         # 快速开始
│   │   ├── basics/                    # 基础
│   │   ├── advanced/                  # 高级主题
│   │   └── cookbook/                  # 实用技巧
│   │
│   ├── reference/                     # 参考手册
│   │   ├── grammar.md                 # 语法参考
│   │   ├── stdlib/                    # 标准库参考
│   │   └── compiler/                  # 编译器参考
│   │
│   ├── internals/                     # 内部文档
│   │   ├── architecture.md            # 系统架构
│   │   ├── ir_design.md               # 中间表示设计
│   │   └── optimization_guide.md      # 优化指南
│   │
│   └── rfcs/                          # RFC 文档
│       ├── template.md                # RFC 模板
│       └── 0001-async-await.md        # 示例 RFC
│
├── benchmarks/                        # 基准测试
│   ├── micro/                         # 微基准
│   │   ├── arithmetic.aether
│   │   ├── collections.aether
│   │   └── concurrency.aether
│   │
│   ├── macro/                         # 宏基准
│   │   ├── web_server.aether
│   │   ├── data_processing.aether
│   │   └── ml_training.aether
│   │
│   └── compare/                       # 与其他语言对比
│       ├── rust/
│       ├── go/
│       └── python/
│
└── scripts/                           # 辅助脚本
    ├── build.sh                       # 构建脚本
    ├── test.sh                        # 测试脚本
    ├── release.sh                     # 发布脚本
    └── ci/                            # CI 配置
        ├── github_actions.yml
        └── jenkins.groovy
```

## 核心文件统计

| 类别 | 文件数 | 代码行数 |
|------|--------|----------|
| 语言规范 | 1 | ~600 行 |
| 编译器架构 | 1 | ~950 行 |
| LSP 规格 | 1 | ~870 行 |
| 标准库文档 | 1 | ~670 行 |
| 示例代码 | 4 | ~380 行 |
| **总计** | **8** | **~3,470 行** |

## 技术栈

### 编译器实现
- **语言**: Rust 2024 Edition
- **LLVM**: 16.0+ (支持最新优化)
- **并行**: Rayon + DashMap
- **图算法**: PetGraph

### 开发工具
- **LSP**: 完整 LSP 3.17 支持
- **包管理**: PubGrub 依赖解析
- **测试**: 内置测试框架 + proptest
- **文档**: rustdoc 风格生成

### 运行时特性
- **异步**: 工作窃取调度器
- **GC**: 可选，基于分代 GC
- **SIMD**: 自动向量化
- **WASM**: 一等公民支持

## 构建与开发

```bash
# 克隆仓库
git clone https://github.com/aether-lang/aether.git
cd aether

# 构建编译器
cargo build --release

# 运行测试
./scripts/test.sh

# 运行基准测试
./scripts/bench.sh

# 生成文档
./scripts/doc.sh

# 安装 LSP
cargo install --path tools/lsp
```

## 项目状态

- ✅ 语言规范完成
- ✅ 编译器架构设计
- ✅ 标准库设计
- ✅ LSP 规格完成
- ✅ 示例代码编写
- 🔄 编译器实现中
- 📋 测试套件规划
- 📋 文档编写中

## 贡献指南

详见 [CONTRIBUTING.md](CONTRIBUTING.md)

## 许可证

MIT License - 详见 [LICENSE](LICENSE)
