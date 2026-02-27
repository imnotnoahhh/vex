# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

vex 是一个用 Rust 编写的 macOS 二进制版本管理器，用于管理 Node.js、Go、Java (Eclipse Temurin JDK)、Rust 等语言的官方二进制发行版。通过符号链接 + PATH 前置（非 shim）实现快速版本切换。

## 开发命令

```bash
# 构建项目
cargo build

# 构建 release 版本
cargo build --release

# 运行测试
cargo test

# 代码检查
cargo clippy

# 代码格式化
cargo fmt

# 格式检查（CI 用）
cargo fmt --check
```

## 核心架构

### 文件系统布局
- `~/.vex/config.toml` - 全局配置
- `~/.vex/locks/` - 操作锁文件（防止并发冲突）
- `~/.vex/cache/` - 下载缓存
- `~/.vex/toolchains/<tool>/<version>/` - 已安装版本
- `~/.vex/current/<tool>` - 当前激活版本的符号链接
- `~/.vex/bin/` - PATH 入口，包含指向 current/*/bin 的符号链接

### 模块结构
- `main.rs` - CLI 入口，使用 clap 定义命令，包含交互式安装逻辑
- `error.rs` - 统一错误处理（VexError 枚举）
- `downloader.rs` - HTTP 下载 + 进度条 + SHA256 校验 + 自动重试（3 次）
- `installer.rs` - 解压 tar.gz + 移动到 toolchains 目录
- `switcher.rs` - 版本切换（原子更新 current/ 和 bin/ 符号链接）
- `resolver.rs` - 版本文件解析（.tool-versions、.node-version 等），向上遍历目录树
- `shell.rs` - Shell Hook 脚本生成（zsh chpwd / bash PROMPT_COMMAND）
- `tools/` - 每个语言的适配层
  - `mod.rs` - Tool trait 定义、Arch 枚举、get_tool() 分发
  - `node.rs` - Node.js（nodejs.org API）
  - `go.rs` - Go（go.dev JSON API）
  - `java.rs` - Java / Eclipse Temurin JDK（Adoptium API）
  - `rust.rs` - Rust（channel-rust-stable.toml）

### Tool Trait
所有语言工具必须实现 `Tool` trait：
- `name()` - 工具名称
- `list_remote()` - 查询可用版本
- `download_url()` - 构造下载 URL（根据架构）
- `checksum_url()` - 构造校验和 URL（可选）
- `bin_names()` - 可执行文件名列表
- `bin_subpath()` - 解压后 bin 目录的相对路径
- `bin_paths()` - 返回 (bin_name, subpath) 对，当二进制文件在不同子目录时覆写（如 Rust 的 rustc/bin 和 cargo/bin）
- `get_checksum()` - 获取 SHA256 校验和（默认返回 None，各工具可覆写）

架构检测使用 `Arch` 枚举：
- `Arch::Arm64` - Apple Silicon
- `Arch::X86_64` - Intel
- `Arch::detect()` - 自动检测当前架构

### 安装流程
1. 检查是否已安装（已安装则提示并跳过）
2. 构造下载 URL（根据工具和架构）
3. 下载到 `~/.vex/cache/`（支持自动重试 3 次，4xx 错误不重试）
4. 验证 SHA256 校验和（通过 `get_checksum()` 获取期望值）
5. 解压 tar.gz 到临时目录
6. 移动到 `toolchains/<tool>/<version>/`
7. 自动切换到新安装的版本（更新符号链接）
8. 清理缓存文件（失败时由 CleanupGuard 自动清理临时文件）

### 版本切换流程
1. 检查版本是否已安装
2. 原子更新 `current/<tool>` 符号链接（通过临时链接 + rename）
3. 更新 `bin/` 下的可执行文件链接（使用 `bin_paths()` 获取路径映射）

### 官方下载源
- Node: `nodejs.org/dist/index.json` + SHASUMS256.txt
- Go: `go.dev/dl/?mode=json`（JSON 中包含 sha256）
- Java: `api.adoptium.net/v3/assets/`（仅 JDK + HotSpot）
- Rust: `static.rust-lang.org/dist/channel-rust-stable.toml`（仅 rustc + cargo）

### 版本文件解析（resolver）
- 支持 `.tool-versions`（asdf 格式）、`.node-version`、`.nvmrc`、`.go-version`、`.java-version`、`.rust-toolchain`
- `.tool-versions` 优先级高于语言专用文件
- 从当前目录向上遍历，直到根目录
- `resolve_versions()` 返回所有工具的版本映射，`resolve_version()` 查询单个工具

### Shell Hook 自动切换
- `vex env zsh` / `vex env bash` 输出 shell 集成脚本
- zsh: 使用 `add-zsh-hook chpwd` 注册目录切换回调
- bash: 使用 `PROMPT_COMMAND` + `__VEX_PREV_DIR` 跟踪目录变化
- `vex use --auto` 静默读取版本文件并切换已安装的版本

### 下一步计划
- Python: 方案待定，候选：python-build-standalone、uv、conda-forge、本地编译 CPython

## 实现阶段

已完成：
1. ✅ 项目骨架 + CLI 框架
2. ✅ Tool trait + Node.js 实现
3. ✅ Go 实现
4. ✅ Java (Eclipse Temurin JDK) + Rust 实现
5. ✅ 辅助命令（current、list、uninstall）+ 交互式安装
6. ✅ SHA256 校验和验证
7. ✅ 安装后自动切换 + 重复安装检测 + 错误清理
8. ✅ .tool-versions 文件支持 + Shell Hook 自动切换
9. ✅ 单元测试 + CLI 集成测试（92 个测试，零警告）

## 关键设计决策

- 仅支持官方二进制，不编译源码
- 使用符号链接 + PATH 前置而非 shim，避免性能开销
- 下载到 cache 临时目录，解压后 rename 到 toolchains，防止半成品
- 自动重试：网络下载失败自动重试 3 次，4xx 客户端错误（如 404）不重试
- SHA256 校验和验证：通过 Tool trait 的 `get_checksum()` 方法获取期望值
- 安装后自动切换版本（自动 use），无需手动执行 `vex use`
- 重复安装检测：已安装版本跳过下载，提示用 `vex use` 切换
- CleanupGuard：安装失败时自动清理临时下载和解压文件（Ctrl+C 中断除外）
- 错误输出使用 Display 格式（如 `Error: Tool not found: python`）
- 架构检测：自动识别 Apple Silicon (arm64) 和 Intel (x86_64)
- MVP 仅支持 macOS (arm64 + x86_64)
- .tool-versions 优先级高于语言专用版本文件（与 asdf/mise 一致）
- Shell Hook 在 cd 时自动检测版本文件并切换，仅切换已安装的版本（未安装则静默跳过）

## 重要实现细节

### Java (Eclipse Temurin) 特殊说明
- 使用 Adoptium API v3，仅 JDK + HotSpot 组合
- macOS JDK 目录结构特殊：`Contents/Home/bin/`
- 版本号使用 major version（8, 11, 17, 21, 25），API 返回完整 semver

### Rust 特殊说明
- 解析 `channel-rust-stable.toml`，仅支持稳定版
- 只安装 rustc + cargo，不安装 clippy、rustfmt 等
- rustc 和 cargo 在不同子目录（`rustc/bin/` 和 `cargo/bin/`），通过覆写 `bin_paths()` 处理
