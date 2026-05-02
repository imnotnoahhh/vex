# vex 验证覆盖说明

本文档描述仓库当前用于验证 vex 的主要测试入口，以及它们覆盖到的能力范围。

当前这套覆盖已经额外纳入：

- `vex init --template` / `--list-templates` / `--add-only`
- 安全 team config 来源解析（本地文件、HTTPS、Git）
- 安装失败后的清理与切换失败后的回滚
- 仓库内 `imnotnoahhh/vex` GitHub Action 所依赖的脚本和 CI smoke path
- `vex repair migrate-home`、captured user-state 导出、以及 Rust 官方 targets/components live smoke

## 验证入口

### 1. 快速功能回归

脚本：`scripts/test-features.sh`

用途：
- 快速验证核心 CLI 功能
- 检查常见二进制是否存在、能否调用
- 覆盖 Python `.venv` 基础工作流
- 适合本地开发中的快速 smoke test

### 2. 管理类功能回归

脚本：`scripts/test-management-features.sh`

用途：
- 验证 `current / list / list-remote / doctor --json`
- 验证 `outdated`、`upgrade --all`、`prune` / `gc`
- 验证 `.vex.toml`、`vex exec`、`vex run` 的真实行为
- 适合在 CI 中做一条更聚焦的新功能 smoke test

### 3. 官方 release 严格验证

脚本：`scripts/test_vex_release_strict.py`

用途：
- fresh 下载最新 GitHub release 二进制
- 在隔离 HOME 中做全量 macOS 验证
- 适合 release 前确认“线上发布物”是否健康

说明：
- 仓库里的 `Strict macOS` workflow 会在 PR 和 `main` push 上验证当前分支的 local build
- “已发布 release 二进制”的这条严格校验改为显式触发，避免在未发布分支上拿最新已发布版本和新分支预期做错误对比
- 已发布 release 的自动校验仍由 `release-postflight` 负责

### 4. 本地构建严格验证

脚本：`scripts/test_vex_release_strict.py`（通过环境变量切到本地构建）

用途：
- 使用本地 `target/debug/vex`
- 复用和 release 严格版相同的验证逻辑与报告格式
- 适合验证修复后的本地代码是否已经通过端到端检查

推荐环境变量：

```bash
VEX_TEST_HOME=/tmp/vex-audit-home \
VEX_STRICT_TMP_ROOT=/tmp/strict-local-build \
VEX_STRICT_USE_LOCAL_BUILD=1 \
VEX_STRICT_VEX_BIN="$(pwd)/target/debug/vex"
```

### 5. Rust 官方扩展 live smoke

脚本：`scripts/test-rust-extensions-live.sh`

用途：
- 真实下载 Rust 官方 stable toolchain
- 真实下载并安装 `aarch64-apple-ios`、`aarch64-apple-ios-sim`、`rust-src`
- 验证 `.vex-metadata.json`、sysroot 链接、移除后的清理行为
- 适合在 CI 和手动验收时确认 Rust 扩展链路没有只停留在 fixture 级别

## 当前严格验证覆盖面

严格验证脚本当前覆盖以下能力：

- 顶层 CLI：`--version`、`-V`、`--help`、`help <command>`
- shell 集成：`vex env zsh/bash/fish/nu`
- 初始化流程：`vex init --dry-run`、`vex init --shell zsh`、`vex init --shell auto`
- 上游版本解析：Node.js、Go、Java、Rust、Python
- fresh install：5 种语言都在隔离 HOME 中重新安装
- 官方归档比对：将本地安装结果与官方 macOS 归档中的二进制清单做比对
- symlink 校验：`~/.vex/current/*` 与 `~/.vex/bin/*`
- 可执行性校验：按工具特征探测 `--version` / `--help` / `-version` 等
- Python 工作流：`vex python init / freeze / sync`，以及 Python base 环境路径隔离
- 多版本切换：手动切到备用版本，再切回目标版本
- 项目与全局切换：`.tool-versions`、`vex global`、shell hook `cd` 自动切换
- Python 自动激活：进入项目自动激活 `.venv`，离开项目自动退出，并隐藏 Python base `bin`
- 健康检查：`vex doctor`
- 全局 CLI 盘点：`vex globals` 覆盖 Go、Rust、Python、Node 与 Java Maven/Gradle 状态
- Rust 官方扩展：`vex rust target add/remove`、`vex rust component add/remove`
- 模板与 team-config 相关 CLI：由单元测试与 CLI integration tests 覆盖

## 当前发现并验证的二进制数量

以下数字来自 2026-03-12 的严格 macOS 验证结果：

| 语言 | 二进制数量 |
|------|-----------|
| Node.js | 4 |
| Go | 2 |
| Java | 30 |
| Rust | 11 |
| Python | 12 |
| **总计** | **59 个二进制** |

## 推荐执行方式

### 快速回归

```bash
bash scripts/test-features.sh
bash scripts/test-management-features.sh
VEX_BIN="$(pwd)/target/debug/vex" bash scripts/test-shell-hooks.sh
VEX_BIN="$(pwd)/target/debug/vex" bash scripts/test-rust-extensions-live.sh
```

### 验证最新发布的 release

```bash
python3 scripts/test_vex_release_strict.py
```

### 验证本地修复后的构建

```bash
VEX_TEST_HOME=/tmp/vex-audit-home \
VEX_STRICT_TMP_ROOT=/tmp/strict-local-build \
VEX_STRICT_USE_LOCAL_BUILD=1 \
VEX_STRICT_VEX_BIN="$(pwd)/target/debug/vex" \
python3 scripts/test_vex_release_strict.py
```

也可以指定隔离 HOME：

```bash
VEX_TEST_HOME=/tmp/vex-audit-home \
VEX_STRICT_TMP_ROOT=/tmp/strict-local-build \
VEX_STRICT_USE_LOCAL_BUILD=1 \
VEX_STRICT_VEX_BIN="$(pwd)/target/debug/vex" \
python3 scripts/test_vex_release_strict.py
```

## 如何解读 warning

严格验证脚本里的 `Warned` 通常表示：

- 上游 API 一次请求返回不完整内容
- 或 `urllib` 失败后脚本改用 `curl` 重试

只要最后 `Failed: 0`，这类 warning 一般表示网络抖动被脚本兜住了，而不是 vex 功能错误。

## 何时更新本文档

以下情况应同步更新本文档：

- 新增或移除某种语言支持
- 某种工具新增或移除可链接二进制
- 严格验证脚本新增新的测试阶段
- 快速功能脚本的覆盖目标发生明显变化
- 新增模板、team config 语义、或 GitHub Action 行为变化

**Last Updated**: 2026-04-04
**Status**: ✅ Comprehensive coverage
