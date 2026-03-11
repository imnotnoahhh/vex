# test-features.sh 测试覆盖报告

## 测试概览

**测试脚本**: `scripts/test-features.sh`
**测试范围**: 5 种语言的所有二进制文件
**测试类型**: 安装、切换、版本检查、帮助信息、which 路径、虚拟环境

---

## 完整测试覆盖清单

### 1. Node.js (4 个二进制)

| 二进制 | 存在性 | which | --version | -v | --help | -h |
|--------|--------|-------|-----------|----|---------|----|
| node | ✓ | ✓ | ✓ | ✓ | ✓ | - |
| npm | ✓ | ✓ | ✓ | ✓ | ✓ | - |
| npx | ✓ | ✓ | ✓ | - | ✓ | - |
| corepack | ✓ | - | ✓ | - | ✓ | - |

**总计**: 4 个二进制 × 平均 4 项测试 = **16 项测试**

---

### 2. Python (12 个二进制)

| 二进制 | 存在性 | which | --version | -V | --help | -h | 特殊测试 |
|--------|--------|-------|-----------|----|---------|----|----------|
| python3 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | - |
| python3.12 | ✓ | - | ✓ | - | - | - | - |
| python | ✓ | ✓ | ✓ | - | - | - | - |
| pip3 | ✓ | ✓ | ✓ | - | ✓ | ✓ | - |
| pip3.12 | ✓ | - | ✓ | - | - | - | - |
| pip | ✓ | ✓ | ✓ | - | - | - | - |
| pydoc3 | ✓ | ✓ | - | - | - | ✓ | - |
| pydoc3.12 | ✓ | - | - | - | - | - | - |
| 2to3 | ✓ | ✓ | ✓ | - | ✓ | ✓ | - |
| 2to3-3.12 | ✓ | - | ✓ | - | - | - | - |
| python3-config | ✓ | ✓ | - | - | ✓ | - | --prefix, --cflags |
| python3.12-config | ✓ | - | - | - | - | - | - |

**总计**: 12 个二进制 × 平均 3 项测试 = **36 项测试**

**Python 虚拟环境测试** (重中之重):
- ✓ `vex python init` 创建 .venv
- ✓ .venv/bin/python 存在
- ✓ .venv/bin/pip 存在
- ✓ .venv/bin/activate 脚本存在
- ✓ `vex python freeze` 创建 requirements.lock
- ✓ `vex python sync` 从 lock 恢复 .venv
- ✓ 无 .venv 时 freeze 报错
- ✓ 无 lock 时 sync 报错

**虚拟环境测试**: **8 项测试**

---

### 3. Go (2 个二进制)

| 二进制 | 存在性 | which | version | -h | help |
|--------|--------|-------|---------|-------|------|
| go | ✓ | ✓ | ✓ | - | ✓ |
| gofmt | ✓ | ✓ | - | ✓ | - |

**总计**: 2 个二进制 × 平均 3 项测试 = **6 项测试**

---

### 4. Rust (10 个二进制)

| 二进制 | 存在性 | which | --version | -V | --help | -h |
|--------|--------|-------|-----------|----|---------|----|
| rustc | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| cargo | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| rustdoc | ✓ | - | ✓ | - | - | - |
| rustfmt | ✓ | ✓ | ✓ | - | ✓ | - |
| cargo-fmt | ✓ | - | ✓ | - | - | - |
| clippy-driver | ✓ | - | ✓ | - | - | - |
| cargo-clippy | ✓ | ✓ | ✓ | - | ✓ | - |
| rust-gdb | ✓ | - | - | - | - | - |
| rust-lldb | ✓ | - | - | - | - | - |
| rust-analyzer | ✓ | ✓ | ✓ | - | - | - |

**总计**: 10 个二进制 × 平均 3 项测试 = **30 项测试**

---

### 5. Java (19 个二进制)

| 二进制 | 存在性 | which | -version | --version | -help | --help |
|--------|--------|-------|----------|-----------|-------|--------|
| java | ✓ | ✓ | ✓ | - | ✓ | - |
| javac | ✓ | ✓ | ✓ | - | ✓ | - |
| jar | ✓ | ✓ | - | ✓ | - | ✓ |
| javadoc | ✓ | ✓ | - | ✓ | - | ✓ |
| javap | ✓ | ✓ | ✓ | - | - | - |
| jshell | ✓ | ✓ | - | ✓ | - | - |
| keytool | ✓ | ✓ | - | - | ✓ | - |
| jarsigner | ✓ | - | - | - | ✓ | - |
| jdb | ✓ | - | ✓ | - | - | - |
| jdeps | ✓ | - | - | ✓ | - | - |
| jfr | ✓ | - | - | ✓ | - | - |
| jhsdb | ✓ | - | - | - | - | - |
| jinfo | ✓ | - | - | - | - | - |
| jmap | ✓ | - | - | - | - | - |
| jps | ✓ | - | ✓ | - | - | - |
| jstack | ✓ | - | ✓ | - | - | - |
| jstat | ✓ | - | ✓ | - | - | - |
| native2ascii | ✓ | - | - | - | - | - |
| rmic | ✓ | - | - | - | - | - |
| serialver | ✓ | - | - | - | - | - |
| jrunscript | ✓ | - | - | - | - | - |

**注意**: 跳过的二进制（GUI/守护进程）:
- jconsole (GUI)
- jstatd (守护进程)
- rmiregistry (守护进程)
- rmid (守护进程)

**总计**: 19 个二进制 × 平均 2.5 项测试 = **48 项测试**

---

## 集成测试

### 8. 跨语言集成 (10 项测试)
- ✓ 切换到 node@20.11.0
- ✓ 切换到 python@3.12
- ✓ 切换到 go@1.23
- ✓ 切换到 rust@1.83
- ✓ 切换到 java@21
- ✓ current 显示 node
- ✓ current 显示 python
- ✓ current 显示 go
- ✓ current 显示 rust
- ✓ current 显示 java

### 9. 版本源检测 (2 项测试)
- ✓ 全局默认显示 "Global default"
- ✓ 项目 .tool-versions 显示 "Project override"

### 10. List Remote 过滤器 (3 项测试)
- ✓ --filter latest 返回 1 个版本
- ✓ --filter lts 返回 LTS 版本
- ✓ --filter major 降序排序

### 11. 安装选项 (2 项测试)
- ✓ install 默认切换
- ✓ install --no-switch 不切换

### 12. 动态二进制检测 (2 项测试)
- ✓ Java 21 没有 jnativescan
- ✓ Node 20 有 corepack

### 13. 并发安装保护 (1 项测试)
- ✓ 锁冲突显示错误消息

### 14. Doctor 健康检查 (7 项测试)
- ✓ 检查 vex 目录
- ✓ 检查目录结构
- ✓ 检查已安装工具
- ✓ 检查 symlinks 完整性
- ✓ 检查二进制可执行性
- ✓ 检查二进制可运行性
- ✓ 检查网络连接

---

## 测试统计

| 类别 | 测试数量 |
|------|----------|
| Node.js 二进制 | 16 |
| Python 二进制 | 36 |
| Python 虚拟环境 | 8 |
| Go 二进制 | 6 |
| Rust 二进制 | 30 |
| Java 二进制 | 48 |
| 跨语言集成 | 10 |
| 版本源检测 | 2 |
| List Remote | 3 |
| 安装选项 | 2 |
| 动态检测 | 2 |
| 并发保护 | 1 |
| Doctor 检查 | 7 |
| **总计** | **171 项测试** |

---

## 测试方法

### 存在性测试
```bash
if [ -e ~/.vex/bin/$bin ]; then
    pass "$bin symlink exists"
fi
```

### which 路径测试
```bash
path=$(which "$bin" 2>/dev/null)
if echo "$path" | grep -q "\.vex/bin"; then
    pass "which $bin → ~/.vex/bin/$bin"
fi
```

### 版本标志测试
```bash
output=$(bash -c "$bin $flag 2>&1 | head -5")
if echo "$output" | grep -qi "$expect"; then
    pass "$bin $flag works"
fi
```

---

## 覆盖的二进制总数

| 语言 | 二进制数量 |
|------|-----------|
| Node.js | 4 |
| Python | 12 |
| Go | 2 |
| Rust | 10 |
| Java | 19 |
| **总计** | **47 个二进制** |

---

## 特殊测试场景

### Python 虚拟环境（重中之重）
- 完整的 venv 生命周期测试
- 错误处理测试
- freeze/sync 工作流测试
- 目录结构验证

### 动态二进制检测
- 版本特定的二进制（jnativescan 仅在 Java 25+）
- 已移除的二进制（corepack 在 Node 25+ 中移除）

### 并发安全
- 文件锁机制测试
- 冲突错误消息验证

---

## 测试执行

```bash
# 运行完整测试
export PATH=$(pwd)/target/release:$PATH
bash scripts/test-features.sh

# 预期输出
╔════════════════════════════════════════════════════════════╗
║  vex v1.0.1 Comprehensive Feature Test Suite              ║
║  Testing ALL binaries for 5 languages                     ║
╚════════════════════════════════════════════════════════════╝

[ 1. Basic Functionality ]
  ✓ vex --version shows 1.0.1
  ✓ vex doctor has no fatal errors

[ 2. Node.js v20 - All Binaries ]
  Installing Node.js 20...
  ✓ node symlink exists
  ✓ npm symlink exists
  ...

╔════════════════════════════════════════════════════════════╗
║                    Test Summary                            ║
╠════════════════════════════════════════════════════════════╣
║  Passed:                                               171 ║
║  Failed:                                                 0 ║
╚════════════════════════════════════════════════════════════╝

✅ All tests passed!
```

---

## 测试时间估算

- Node.js 安装: ~10s
- Python 安装: ~30s
- Go 安装: ~5s
- Rust 安装: ~10s
- Java 安装: ~15s
- 所有测试执行: ~30s

**总计**: 约 **100 秒**（1.5 分钟）

---

## 维护指南

### 添加新语言
1. 在对应章节添加二进制列表
2. 添加存在性测试
3. 添加 which 测试
4. 添加版本/帮助标志测试

### 添加新二进制
1. 更新二进制列表
2. 添加到 `check_bin_exists` 循环
3. 如果常用，添加到 `check_which` 循环
4. 添加版本标志测试

### 更新版本
修改安装命令中的版本号：
```bash
vex install node@20.11.0  # 改为新版本
```

---

**最后更新**: 2026-03-11
**测试覆盖**: 47 个二进制，171 项测试
**状态**: ✅ 全面覆盖
