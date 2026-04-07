# Aether 包管理器 (apm)

## 概述
`apm` (Aether Package Manager) 是 Aether 语言的官方包管理工具，用于依赖管理、项目构建、测试运行和发布。

## 安装
```bash
curl -sSf https://aether-lang.org/install.sh | sh
```

## 核心命令

### 项目管理
- `apm new <name>` - 创建新项目
- `apm init` - 在当前目录初始化项目
- `apm build` - 编译项目（支持 debug/release）
- `apm run` - 运行项目
- `apm test` - 运行测试套件
- `apm check` - 检查代码（类型检查、lint）
- `apm fmt` - 格式化代码
- `apm doc` - 生成文档

### 依赖管理
- `apm add <crate>` - 添加依赖
- `apm remove <crate>` - 移除依赖
- `apm update` - 更新依赖
- `apm tree` - 显示依赖树
- `apm audit` - 安全审计

### 发布
- `apm publish` - 发布包到仓库
- `apm login` - 登录到仓库
- `apm owner` - 管理包所有者

## 配置文件 (Aether.toml)

```toml
[package]
name = "my-crate"
version = "0.1.0"
edition = "2024"
authors = ["Alice <alice@example.com>"]
description = "A sample Aether crate"
license = "MIT"
repository = "https://github.com/alice/my-crate"
keywords = ["web", "async"]
categories = ["network-programming"]

[dependencies]
aether-std = "1.0"
hyperion = { version = "2.1", features = ["full"] }
serde = "1.0"

[dev-dependencies]
aether-test = "1.0"

[build-dependencies]
codegen = "0.3"

[[bin]]
name = "my-app"
path = "src/main.ae"

[lib]
name = "my_crate"
path = "src/lib.ae"
crate-type = ["lib", "cdylib"]

[profile.dev]
opt-level = 0
debug = true

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"

[features]
default = ["std"]
std = []
full = ["std", "async", "network"]
```

## 依赖解析策略

1. **语义化版本**：遵循 SemVer 2.0.0
2. **最小版本选择**：默认选择满足条件的最低版本
3. **锁文件**：生成 `Aether.lock` 确保可重复构建
4. **冲突解决**：使用图算法解决版本冲突

## 仓库架构

```
registry/
├── crates/
│   └── hyperion/
│       ├── 2.0.0/
│       ├── 2.1.0/
│       └── 2.1.1/
├── index/
│   └── hy/pe/hyperion.json
└── config.toml
```

## API 集成

```ae
// 在代码中动态加载包
use apm::resolver;

fn main() {
    let mut resolver = Resolver::new();
    resolver.add_dependency("hyperion", "^2.0");
    
    match resolver.resolve() {
        Ok(graph) => {
            for crate in graph.crates() {
                println!("{} {}", crate.name, crate.version);
            }
        }
        Err(e) => eprintln!("Resolution failed: {}", e),
    }
}
```

## 安全特性

- **校验和验证**：所有包下载后验证 SHA-256
- **签名验证**：支持 GPG 签名验证发布者身份
- **沙箱构建**：在隔离环境中执行构建脚本
- **漏洞扫描**：自动检测已知 CVE

## 性能优化

- **并行下载**：同时下载多个依赖
- **增量编译**：仅重新编译变更的包
- **缓存策略**：本地缓存已下载的包
- **二进制缓存**：共享编译产物

## 扩展系统

```toml
# Aether.toml
[build-script]
path = "build.ae"

[plugins]
custom-linter = "1.0"
deploy-helper = "0.5"
```

## 环境变量

- `AETHER_HOME` - Aether 安装目录
- `AETHER_REGISTRY` - 自定义仓库地址
- `AETHER_CARGO_NET_OFFLINE` - 离线模式
- `RUST_BACKTRACE` - 启用回溯（调试用）

## 示例工作流

```bash
# 创建新项目
apm new my-webapp
cd my-webapp

# 添加依赖
apm add hyperion serde json

# 开发
apm run
apm test
apm fmt

# 发布
apm login
apm publish
```

## 故障排除

### 常见问题

1. **依赖冲突**
   ```bash
   apm tree --duplicates
   apm update --aggressive
   ```

2. **构建失败**
   ```bash
   apm clean
   apm build --verbose
   ```

3. **网络问题**
   ```bash
   export AETHER_REGISTRY=https://mirror.example.com
   apm fetch --offline
   ```

## 未来规划

- [ ] WebAssembly 支持
- [ ] 图形化界面
- [ ] 智能依赖建议
- [ ] 云构建服务
- [ ] 包分析仪表板
