# CI 配置说明

## 当前 CI 工作流

### 1. Test CI (`.github/workflows/test.yml`)

**触发条件**:
- Push 到 `main` 分支
- Pull Request

**任务**:
| 任务 | 说明 |
|------|------|
| `test` | 构建项目并运行单元测试 |
| `install-test` | 测试 `cargo install --git` 安装方式 |

**配置详情**:
```yaml
# 测试 cargo install --git
cargo install --git https://github.com/CNCSMonster/mdbook-zh-search.git --locked
mdbook --version
```

---

### 2. Release CI (`.github/workflows/release.yml`)

**触发条件**:
- Push tag（如 `v0.1.0`）
- 手动触发（Workflow Dispatch）

**任务**:
| 任务 | 平台 | 产物 |
|------|------|------|
| `create-source-tarball` | Ubuntu | 源码压缩包 |
| `build` | Ubuntu Linux | `mdbook-zh-search-linux` |
| `build` | Ubuntu Linux (musl) | `mdbook-zh-search-linux-musl` |
| `build` | macOS Intel | `mdbook-zh-search-macos-intel` |
| `build` | macOS ARM | `mdbook-zh-search-macos-arm` |
| `build` | Windows | `mdbook-zh-search-windows.exe` |
| `create-release` | - | GitHub Release |

**使用方法**:

#### 自动发布（推荐）
```bash
# 打 tag 并推送
git tag v0.1.0
git push origin v0.1.0
```

CI 会自动：
1. 构建所有平台的二进制文件
2. 创建 GitHub Release
3. 上传所有产物和 SHA256 校验和

#### 手动发布
1. 访问 https://github.com/CNCSMonster/mdbook-zh-search/actions/workflows/release.yml
2. 点击 "Run workflow"
3. 输入版本号（如 `v0.1.0`）
4. 点击 "Run workflow"

---

## 安装方式

### ✅ 方式一：从 Git 安装（推荐）

```bash
cargo install --git https://github.com/CNCSMonster/mdbook-zh-search.git
mdbook --version
```

**优点**:
- 最简单
- 自动获取最新版本
- Cargo 自动处理依赖

**缺点**:
- 需要本地编译（约 30 秒）

---

### ✅ 方式二：从源码安装

```bash
git clone https://github.com/CNCSMonster/mdbook-zh-search.git
cd mdbook-zh-search
cargo install --path .
```

**优点**:
- 可以修改代码
- 适合开发者

**缺点**:
- 需要克隆整个仓库

---

### ⏳ 方式三：下载预编译二进制

从 [Releases](https://github.com/CNCSMonster/mdbook-zh-search/releases) 下载。

**优点**:
- 无需编译
- 立即使用

**缺点**:
- 需要等待首次 Release

---

## 验证安装

```bash
# 检查版本
mdbook --version
# 输出：mdbook v0.5.2

# 测试中文搜索
cd test-book
mdbook build
# 打开生成的 book/index.html
# 按 S 或 / 测试搜索功能
```

---

## 本地测试

### 运行单元测试
```bash
cargo test --package mdbook-html --features search
```

### 运行集成测试
```bash
cargo test
```

### 构建测试书籍
```bash
cd test-book
mdbook build
```

---

## CI 状态

[![Test](https://github.com/CNCSMonster/mdbook-zh-search/actions/workflows/test.yml/badge.svg)](https://github.com/CNCSMonster/mdbook-zh-search/actions/workflows/test.yml)
[![Release](https://github.com/CNCSMonster/mdbook-zh-search/actions/workflows/release.yml/badge.svg)](https://github.com/CNCSMonster/mdbook-zh-search/actions/workflows/release.yml)
