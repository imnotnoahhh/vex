<h1 align="center">vex</h1>

<p align="center">
  <a href="README.md">English</a> · <a href="README.zh-CN.md">简体中文</a>
</p>

<p align="center">
  <strong>面向 macOS 的高速多语言版本管理器</strong>
</p>

<p align="center">
  符号链接切换 · Node.js / Go / Java / Rust / Python · .tool-versions · cd 自动切换
</p>

<p align="center">
  <a href="https://github.com/imnotnoahhh/vex/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/imnotnoahhh/vex/ci.yml?style=flat-square&label=CI" alt="CI">
  </a>
  <a href="https://github.com/imnotnoahhh/vex/releases">
    <img src="https://img.shields.io/github/v/release/imnotnoahhh/vex?style=flat-square" alt="Release">
  </a>
  <a href="https://github.com/imnotnoahhh/vex/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/imnotnoahhh/vex?style=flat-square" alt="License">
  </a>
</p>

<p align="center">
  <a href="#快速开始">快速开始</a> ·
  <a href="#常用命令">常用命令</a> ·
  <a href="#python-工作流">Python 工作流</a> ·
  <a href="#开源与贡献">开源与贡献</a>
</p>

<p align="center">
  <img src="./docs/demo/vex-install.gif" alt="vex install demo" width="980" />
</p>

## 这是什么

vex 是一个 macOS 原生的多语言版本管理器。它用直接符号链接切换版本，没有 shim 启动开销，同时把工具链、缓存、全局 CLI 和语言用户态目录尽量收进 `~/.vex`。

它适合这些场景：

- 同时管理 Node.js、Go、Java、Rust、Python。
- 在项目里使用 `.tool-versions`，进入目录时自动切换版本。
- 用 Python 的 per-version base 环境安装全局 CLI，比如 `kaggle`，同时保持项目 `.venv` 隔离。
- 在 CI 里用官方 GitHub Action 恢复和激活缓存好的工具链。
- 用 `vex doctor` 和 `vex globals` 检查 PATH、shell hook、全局 CLI、Maven/Gradle 状态和常见冲突。

## 功能亮点

- **无 shim 切换**：`~/.vex/bin/node` 直接指向真实工具链二进制。
- **多语言支持**：Node.js、Go、Java(Eclipse Temurin)、Rust、Python。
- **Python base + venv**：`vex python base pip install kaggle` 安装用户级 Python CLI，项目 `.venv` 激活后不会泄漏 base 包。
- **项目本地优先**：Node 项目里 `node_modules/.bin` 优先于 npm 全局 bin。
- **全局 CLI 清单**：`vex globals` 显示 npm、Python base、Go、Cargo、Maven、Gradle 来源和状态。
- **稳定 Python latest**：`vex list-remote python --filter latest` 优先 bugfix/security 版本，避免把 feature/prerelease 当默认 latest。
- **自动 shell 集成**：支持 zsh、bash、fish、nushell。
- **项目模板**：`vex init --template python-venv --add-only` 可安全补齐项目模板文件。
- **锁文件和离线模式**：`.tool-versions.lock`、`--frozen`、`--offline` 支持可复现和缓存优先的安装。
- **发布验证**：CI 覆盖 Rustfmt、Clippy、三平台测试、安全审计、macOS feature smoke、Strict macOS、发布 postflight。

## 快速开始

### 安装

推荐使用一行安装脚本：

```bash
# 最新版本
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash

# 指定版本
curl -fsSL https://raw.githubusercontent.com/imnotnoahhh/vex/main/scripts/install-release.sh | bash -s -- --version v1.7.0
```

如果你已经使用 Homebrew，也可以使用官方 tap：

```bash
brew install imnotnoahhh/homebrew-vex/vex
```

从源码构建：

```bash
git clone https://github.com/imnotnoahhh/vex.git
cd vex
cargo build --release
cp target/release/vex ~/.local/bin/vex
```

验证安装：

```bash
vex --version
```

### 初始化 shell

```bash
vex init --shell auto
```

手动配置示例：

```bash
# zsh
echo 'eval "$(vex env zsh)"' >> ~/.zshrc
source ~/.zshrc

# bash
echo 'eval "$(vex env bash)"' >> ~/.bashrc
source ~/.bashrc

# fish
echo 'vex env fish | source' >> ~/.config/fish/config.fish

# nushell
vex env nu | save -f ~/.config/nushell/vex.nu
echo 'source ~/.config/nushell/vex.nu' >> ~/.config/nushell/config.nu
```

## 常用命令

```bash
# 安装并切换版本
vex install node@20
vex install go@latest
vex install java@lts
vex install rust@stable
vex install python@latest

# 只安装，不切换当前版本
vex install node@20 --no-switch

# 项目版本 pin
vex local node@20.11.0
vex local python@3.14

# 安装当前项目 .tool-versions 中的所有工具
vex install

# 切换到当前项目解析出的版本
vex use --auto

# 查看当前版本和全局 CLI
vex current
vex globals
vex globals python --json

# 健康检查
vex doctor
vex doctor --verbose

# 临时在 vex 环境里运行命令，不改变全局版本
vex exec -- node -v
vex exec -- python -V

# 运行 .vex.toml 中的项目任务
vex run test
```

完整命令请看 [命令参考](docs/guides/command-reference.md)。

## Python 工作流

vex 的 Python 来自 [python-build-standalone](https://github.com/astral-sh/python-build-standalone) 的标准 `install_only` CPython 包。

```bash
# 1. 安装 Python
vex install python@3.14
vex use python@3.14

# 2. 安装用户级 CLI 到当前 Python base 环境
vex python base
vex python base pip install kaggle
kaggle --version

# 3. 进入项目后创建隔离 venv
cd my-project
vex python init
pip install requests flask
vex python freeze

# 4. 新机器恢复项目环境
vex install
vex python sync
```

Python 有两个依赖作用域：

- `~/.vex/python/base/<version>`：每个 Python 版本一个 base 环境，适合 `kaggle`、`black` 等用户级 CLI。
- `project/.venv`：项目环境。shell hook 激活 `.venv` 后会隐藏 Python base `bin`，避免全局包污染项目。

## .tool-versions 工作流

```bash
vex local node@20.11.0
vex local go@1.24
vex local python@3.14

# 生成 .tool-versions 后提交
git add .tool-versions
git commit -m "Pin project toolchains"

# 队友克隆后
vex install
```

启用 shell hook 后，进入项目目录会自动执行 `vex use --auto`。

## GitHub Actions

仓库根目录提供 macOS-only composite action：

```yaml
- uses: imnotnoahhh/vex@v1
  with:
    tools: node@20 go@1.24 python@3.14
```

或者根据项目文件自动安装：

```yaml
- uses: imnotnoahhh/vex@v1
  with:
    auto-install: true
```

## 文档

- 英文 README：[README.md](README.md)
- 用户文档索引：[docs/README.md](docs/README.md)
- 安装指南：[docs/guides/installation.md](docs/guides/installation.md)
- 命令参考：[docs/guides/command-reference.md](docs/guides/command-reference.md)
- 配置指南：[docs/guides/configuration.md](docs/guides/configuration.md)
- 迁移对比：[docs/guides/migration-comparison.md](docs/guides/migration-comparison.md)
- 最佳实践：[docs/guides/best-practices.md](docs/guides/best-practices.md)
- 故障排查：[docs/guides/troubleshooting.md](docs/guides/troubleshooting.md)

## 开发

```bash
git clone https://github.com/imnotnoahhh/vex.git
cd vex
cargo build
cargo test

# CI 对齐检查
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features -- --test-threads=1
VEX_BIN="$(pwd)/target/debug/vex" bash scripts/test-management-features.sh
```

## 开源与贡献

这个仓库包含：

- [MIT License](LICENSE)
- [贡献指南](CONTRIBUTING.md)
- [安全策略](SECURITY.md)
- [支持说明](SUPPORT.md)
- [行为准则](CODE_OF_CONDUCT.md)
- Issue 模板和 PR 模板
- CI、发布、postflight 和 Homebrew tap 自动化

欢迎提交 issue 和 PR。贡献前请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

## License

[MIT](LICENSE) © 2026 Noah Qin
