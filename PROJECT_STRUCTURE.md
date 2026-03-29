# mdbook-zh-search 项目结构

## 📁 项目结构概览

```
mdbook-zh-search/
├── 📄 核心文件
│   ├── README.md                  # 项目说明和安装指南 ⭐
│   ├── Cargo.toml                 # Rust 项目配置
│   ├── Cargo.lock                 # 依赖锁定文件
│   ├── LICENSE                    # MPL-2.0 许可证
│   └── .gitignore                 # Git 忽略规则
│
├── 🔧 CI/CD
│   └── .github/workflows/
│       ├── test.yml               # 测试工作流
│       └── release.yml            # 发布工作流
│
├── 📚 文档
│   └── docs/
│       ├── 中文搜索支持方案.md     # 设计方案 ⭐
│       ├── 实现总结.md            # 实现总结 ⭐
│       ├── 英文搜索验证报告.md     # 兼容性验证 ⭐
│       └── 要求.md                # 原始需求
│
├── 🦀 源代码
│   ├── src/
│   │   └── main.rs                # 主入口（含版本信息）
│   └── crates/                    # 子 crates（来自 mdBook）
│       ├── mdbook-core/           # 核心功能
│       ├── mdbook-driver/         # 驱动器
│       ├── mdbook-html/           # HTML 渲染器 ⭐ (中文搜索修改处)
│       ├── mdbook-markdown/       # Markdown 解析
│       └── ...
│
├── 🧪 测试
│   ├── test-book/                 # 中文搜索测试书籍 ⭐
│   │   ├── book.toml
│   │   ├── src/
│   │   │   ├── SUMMARY.md
│   │   │   ├── 01-intro.md        # 中文内容
│   │   │   ├── 02-rust.md         # 中英文混合
│   │   │   └── 03-segmentation.md # 分词技术
│   │   ├── analyze_index.py       # 索引分析脚本
│   │   └── verify_tokenization.py # 分词验证脚本
│   └── tests/                     # mdBook 原测试套件
│
├── 📖 指南 (来自 mdBook)
│   └── guide/                     # mdBook 用户指南源码
│
└── 🛠️ 工具
    ├── ci/                        # CI 脚本
    ├── examples/                  # 示例
    └── tests/gui/                 # GUI 测试
```

---

## 📂 核心目录说明

### 1. `src/` - 主程序入口
```
src/
└── main.rs                        # 添加了中文搜索版本标识
```

**修改内容**:
- 添加 `VERSION_WITH_FEATURES` 显示 "with Chinese search support"
- 帮助信息底部添加 🇨🇳 中文搜索说明

---

### 2. `crates/mdbook-html/` - HTML 渲染器 ⭐ 核心修改
```
crates/mdbook-html/
├── Cargo.toml                     # 添加了 jieba-rs 依赖
└── src/
    └── html_handlebars/
        └── search.rs              # 中文搜索核心实现 ⭐⭐⭐
```

**修改内容**:
- 添加 `jieba-rs` 和 `lazy_static` 依赖
- 实现 `CjkLanguage` 结构体
- 改进 `tokenize()` 函数支持中文分词
- 添加中文停用词过滤
- 添加 CJK 文本检测

---

### 3. `test-book/` - 测试书籍
```
test-book/
├── book.toml                      # 测试书籍配置
├── src/
│   ├── SUMMARY.md                 # 目录
│   ├── 01-intro.md                # 简介（纯中文）
│   ├── 02-rust.md                 # Rust（中英文混合）
│   └── 03-segmentation.md         # 分词技术（中文）
├── analyze_index.py               # 分析搜索索引
└── verify_tokenization.py         # 验证分词效果
```

**用途**: 验证中文搜索功能是否正常工作

---

### 4. `docs/` - 项目文档
```
docs/
├── 中文搜索支持方案.md             # 详细设计方案
├── 实现总结.md                    # 实现过程总结
├── 英文搜索验证报告.md             # 兼容性验证报告
└── 要求.md                        # 原始需求文档
```

---

### 5. `.github/workflows/` - CI/CD
```
.github/workflows/
├── test.yml                       # 测试 + cargo install 验证
└── release.yml                    # 多平台构建 + GitHub Release
```

---

## 🗂️ 文件分类

### 📌 重要文件（不要删除）
| 文件 | 说明 |
|------|------|
| `README.md` | 项目说明和安装指南 |
| `src/main.rs` | 主程序入口 |
| `crates/mdbook-html/Cargo.toml` | 依赖配置 |
| `crates/mdbook-html/src/html_handlebars/search.rs` | 中文搜索核心 |
| `test-book/` | 测试书籍 |
| `docs/*.md` | 项目文档 |
| `.github/workflows/*.yml` | CI 配置 |

### 📦 mdBook 原生文件（保持原样）
- `crates/` 下其他子 crate
- `tests/` 测试套件
- `guide/` 用户指南
- `examples/` 示例

### 🧹 可清理文件（可选）
| 文件/目录 | 说明 | 建议 |
|----------|------|------|
| `target/` | 编译产物 | ✅ 应被 .gitignore 忽略 |
| `guide/` | mdBook 官方指南 | 可删除（与中文搜索无关） |
| `examples/` | 示例 | 可删除（与中文搜索无关） |
| `tests/gui/` | GUI 测试 | 可删除（与中文搜索无关） |
| `tests/testsuite/` | 集成测试 | 可保留（验证兼容性） |
| `CHANGELOG.md` | mdBook 变更日志 | 可删除 |
| `CODE_OF_CONDUCT.md` | 行为准则 | 可保留 |
| `CONTRIBUTING.md` | 贡献指南 | 可保留 |
| `ci/` | CI 脚本 | 可删除（已有 GitHub Actions） |
| `eslint.config.mjs` | ESLint 配置 | 可删除（前端相关） |
| `package.json` | npm 配置 | 可删除（前端相关） |
| `triagebot.toml` | Rust bot 配置 | 可删除（官方仓库用） |

---

## 🎯 核心修改总结

### 修改的文件 (2 个)
1. **`crates/mdbook-html/Cargo.toml`**
   ```toml
   [dependencies]
   jieba-rs = "0.6"
   lazy_static = "1.4"
   ```

2. **`crates/mdbook-html/src/html_handlebars/search.rs`**
   - 添加 `CjkLanguage` 实现
   - 改进 `tokenize()` 函数
   - 添加中文停用词
   - 禁用英文 Pipeline

3. **`src/main.rs`**
   - 更新版本信息显示中文搜索支持

### 新增的文件
- `docs/` 目录（4 个文档）
- `test-book/` 目录（测试书籍）
- `.github/workflows/` (2 个 CI 配置)

---

## 📊 代码统计

| 类别 | 文件数 | 代码行数 |
|------|--------|----------|
| 核心修改 | 3 | ~200 行 |
| 新增文档 | 4 | ~800 行 |
| 测试代码 | 5 | ~400 行 |
| CI 配置 | 2 | ~200 行 |
| **总计** | **14** | **~1600 行** |

---

## 🔍 快速定位

### 想看中文搜索实现？
👉 `crates/mdbook-html/src/html_handlebars/search.rs`

### 想测试中文搜索？
👉 `cd test-book && mdbook build`

### 想看设计思路？
👉 `docs/中文搜索支持方案.md`

### 想验证兼容性？
👉 `docs/英文搜索验证报告.md`

### 想了解如何安装？
👉 `README.md` 的 "安装" 章节
