# mdbook-zh-search

[![Build Status](https://img.shields.io/github/actions/workflow/status/rust-lang/mdBook/main.yml)](https://github.com/rust-lang/mdBook/actions)
[![License: MPL-2.0](https://img.shields.io/badge/license-MPL--2.0-blue.svg)](LICENSE)

为 **mdBook** 添加完整的**中文搜索支持**，同时保持对英文和其他语言的兼容性。

## 🎯 项目目标

mdBook 原有的搜索功能基于 `elasticlunr-rs`，默认只支持英文分词。本项目通过集成 `jieba-rs` 中文分词库，实现了对中文搜索的完整支持。

## ✨ 特性

- 🔍 **中文搜索**：使用 `jieba-rs` 进行智能中文分词
- 🌏 **多语言支持**：同时支持英文、中英文混合内容
- 🚫 **停用词过滤**：自动过滤常见中文停用词（的、了、是等）
- 📚 **Bigram 增强**：添加字符二元语法，提高搜索召回率
- ✅ **完全兼容**：保持与原有英文搜索功能的完全兼容

## 📦 安装

### 方式一：从 Git 安装（推荐）

```bash
# 直接从 GitHub 安装
cargo install --git https://github.com/CNCSMonster/mdbook-zh-search.git

# 验证安装
mdbook --version
```

### 方式二：从源码安装

```bash
# 克隆仓库
git clone https://github.com/CNCSMonster/mdbook-zh-search.git
cd mdbook-zh-search

# 编译安装
cargo install --path .
```

### 方式三：下载预编译二进制

从 [Releases](https://github.com/CNCSMonster/mdbook-zh-search/releases) 下载对应平台的二进制文件：

- **Linux**: `mdbook-zh-search-linux`
- **Linux (musl)**: `mdbook-zh-search-linux-musl`
- **macOS (Intel)**: `mdbook-zh-search-macos-intel`
- **macOS (Apple Silicon)**: `mdbook-zh-search-macos-arm`
- **Windows**: `mdbook-zh-search-windows.exe`

下载后放到 PATH 目录即可：

```bash
# Linux/macOS
chmod +x mdbook-zh-search-linux
sudo mv mdbook-zh-search-linux /usr/local/bin/mdbook

# Windows (PowerShell)
mv mdbook-zh-search-windows.exe C:\Windows\mdbook.exe
```

## 🚀 快速开始

### 1. 创建 mdBook 项目

```bash
mdbook init my-chinese-book
cd my-chinese-book
```

### 2. 配置 `book.toml`

```toml
[book]
title = "我的中文书籍"
authors = ["Your Name"]
language = "zh"

[output.html]
default-theme = "light"

[output.html.search]
enable = true
limit-results = 30
```

### 3. 构建书籍

```bash
mdbook build
```

### 4. 预览

```bash
mdbook serve
```

然后在浏览器中打开 `http://localhost:3000`，按 `S` 或 `/` 即可使用搜索功能。

## 📖 示例

查看 [`test-book/`](test-book/) 目录中的完整示例：

```bash
cd test-book
mdbook serve
```

示例书籍包含：
- 纯中文内容
- 中英文混合内容
- 技术术语和代码示例

## 🔧 技术实现

### 核心修改

1. **添加依赖** (`crates/mdbook-html/Cargo.toml`)
   ```toml
   jieba-rs = "0.6"
   lazy_static = "1.4"
   ```

2. **自定义 CJK Language** (`crates/mdbook-html/src/html_handlebars/search.rs`)
   - 实现 `elasticlunr::lang::Language` trait
   - 使用 jieba 进行中文分词
   - 禁用英文 Pipeline 以保留中文字符

3. **智能分词函数**
   - 自动检测 CJK 文本
   - 中文使用 jieba 分词
   - 英文保持原有分词逻辑
   - 添加 bigram 提高召回率

### 分词示例

| 输入 | 输出 Tokens |
|------|-------------|
| "你好世界" | ["你好", "世界", "好世"] |
| "Rust 编程语言" | ["rust", "编程", "语言"] |
| "hello world" | ["hello", "world"] |

## ✅ 测试验证

### 单元测试

```bash
cargo test --package mdbook-html --features search
```

**结果**：30 个测试全部通过

### 索引统计

| 类型 | Token 数量 |
|------|-----------|
| 英文 | 130 |
| 中文 | 223 |
| 总计 | 374 |

### 关键词验证

**英文关键词**：✅ rust, programming, language, main, println, fn, let

**中文关键词**：✅ 中文，分词，搜索，中国，内存，安全，编程，语言

## 📚 文档

- [中文搜索支持方案](docs/中文搜索支持方案.md) - 详细设计方案
- [实现总结](docs/实现总结.md) - 实现过程总结
- [英文搜索验证报告](docs/英文搜索验证报告.md) - 兼容性验证

## 🔗 相关项目

- [mdBook](https://github.com/rust-lang/mdBook) - 官方 mdBook 项目
- [jieba-rs](https://github.com/messense/jieba-rs) - Rust 版中文分词库
- [elasticlunr-rs](https://crates.io/crates/elasticlunr-rs) - mdBook 使用的搜索库

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

### 开发环境设置

```bash
# 需要 Rust 1.88.0 或更高版本
rustup update

# 克隆后开发
git clone https://github.com/CNCSMonster/mdbook-zh-search.git
cd mdbook-zh-search
cargo build
```

## 📄 许可证

本项目遵循 [MPL-2.0](LICENSE) 许可证（与 mdBook 保持一致）。

## 🙏 致谢

- mdBook 团队开发的优秀工具
- jieba-rs 提供的中文分词能力
- 所有为中文搜索支持做出贡献的社区成员

---

**注意**：本项目是 mdBook 的一个分支，专注于添加中文搜索支持。建议定期同步官方 mdBook 的更新。
