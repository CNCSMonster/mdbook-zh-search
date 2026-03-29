# jieba-rs 使用架构澄清

## 📋 核心问题

**"我们不是已经用上 jieba-rs 的分词能力了吗？为什么还要编译成 WASM？"**

---

## 🔍 当前架构分析

### 当前 jieba-rs 的使用位置

```
┌─────────────────────────────────────────────────────────┐
│  构建时 (Build Time) - Rust 程序                         │
│                                                         │
│  mdbook-zh-search (本地运行)                             │
│  ├── 使用 jieba-rs 分词 ✅                               │
│  ├── 使用 elasticlunr-rs 建索引                          │
│  └── 输出 searchindex.js                                │
└─────────────────────────────────────────────────────────┘
                        ↓
                  生成文件
                        ↓
┌─────────────────────────────────────────────────────────┐
│  运行时 (Runtime) - 浏览器                               │
│                                                         │
│  浏览器加载 searchindex.js                               │
│  ├── 使用 elasticlunr.js 搜索                            │
│  └── ❌ 没有分词功能！                                   │
└─────────────────────────────────────────────────────────┘
```

---

## ⚠️ 关键问题：分词不一致

### 构建时（Rust）

```rust
// crates/mdbook-html/src/html_handlebars/search.rs

use jieba_rs::Jieba;
use lazy_static::lazy_static;

lazy_static! {
    static ref JIEBA: Jieba = Jieba::new();
}

fn tokenize(text: &str) -> Vec<String> {
    // ✅ 使用 jieba-rs 分词
    JIEBA.cut_for_search(text, false)
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

// 示例：
// 输入："一句话总结"
// 输出：["一句", "句话", "总结", "一句话"]
```

### 运行时（JavaScript）- 问题所在！

```javascript
// searcher.js (当前实现)

function search(query) {
    // ❌ 用户输入的搜索词没有分词！
    // 直接传递给 elasticlunr.js
    
    const results = index.search(query, options);
    
    // 问题：
    // 1. 构建时："一句话总结" → ["一句", "句话", "总结", "一句话"]
    // 2. 搜索时："一句话总结" → 直接搜索（未分词）
    // 3. 结果：可能找不到！
}
```

---

## 🐛 实际问题演示

### 场景 1：搜索"一句话总结"

```
构建时索引:
  文档内容："这是一句话总结"
  分词后：["这是", "一句", "句话", "总结", "一句话"]
  索引：{ "一句": [doc1], "句话": [doc1], "总结": [doc1], "一句话": [doc1] }

搜索时:
  用户输入："一句话总结"
  
  当前方式（无分词）:
    搜索词："一句话总结"
    匹配：❌ 索引中没有完整的"一句话总结"
    结果：找不到！
  
  理想方式（有分词）:
    搜索词："一句话总结" → ["一句", "句话", "总结", "一句话"]
    匹配：✅ "一句" 匹配，"句话" 匹配，"总结" 匹配
    结果：找到 doc1
```

---

### 场景 2：搜索"内存安全"

```
构建时索引:
  文档内容："Rust 的内存安全特性"
  分词后：["Rust", "的", "内存", "安全", "特性"]
  索引：{ "内存": [doc1], "安全": [doc1], "特性": [doc1] }

搜索时:
  用户输入："内存安全"
  
  当前方式（无分词）:
    搜索词："内存安全"
    匹配：❌ 索引中没有"内存安全"（只有分开的"内存"和"安全"）
    结果：找不到！
  
  理想方式（有分词）:
    搜索词："内存安全" → ["内存", "安全"]
    匹配：✅ "内存" 匹配，"安全" 匹配
    结果：找到 doc1
```

---

## ✅ 当前解决方案（不完美）

### 方案：elasticlunr 的 expand 选项

```javascript
// searcher.js
const search_options = {
    bool: 'OR',
    expand: true,  // ✅ 启用扩展搜索
    fields: {
        title: {boost: 2},
        body: {boost: 1},
    },
};

// expand: true 的作用:
// 搜索"内存安全"时，会尝试匹配"内存"和"安全"
// 但这不是真正的分词，只是子串匹配
```

**问题**：
- ❌ 不是真正的中文分词
- ❌ 准确率不高
- ❌ 无法处理复杂情况

---

## 🎯 为什么需要 jieba-rs WASM？

### 目标：构建时和运行时分词一致

```
┌─────────────────────────────────────────────────────────┐
│  构建时 (Rust)                                           │
│                                                         │
│  jieba-rs 分词                                          │
│  "一句话总结" → ["一句", "句话", "总结", "一句话"]        │
│  ↓                                                      │
│  生成索引                                                │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  运行时 (浏览器)                                         │
│                                                         │
│  jieba-rs WASM 分词 (新增)                               │
│  "一句话总结" → ["一句", "句话", "总结", "一句话"]        │
│  ↓                                                      │
│  搜索索引                                                │
└─────────────────────────────────────────────────────────┘

✅ 分词逻辑完全一致！
```

---

## 📊 方案对比

### 方案 A：当前方式（无运行时分析）

```
构建时：jieba-rs (Rust)
运行时：无分词，直接搜索

优点:
✅ 简单
✅ 无需 WASM

缺点:
❌ 分词不一致
❌ 搜索准确率低
❌ 用户体验差
```

### 方案 B：JavaScript 分词（替代方案）

```
构建时：jieba-rs (Rust)
运行时：node-segmentit / node-jieba (JavaScript)

优点:
✅ 无需 WASM
✅ 分词一致

缺点:
❌ JavaScript 分词库质量参差不齐
❌ 与 Rust 版本可能有差异
❌ 维护两套依赖
```

### 方案 C：jieba-rs WASM（推荐）✅

```
构建时：jieba-rs (Rust)
运行时：jieba-rs WASM

优点:
✅ 分词完全一致
✅ 代码复用
✅ 质量可靠
✅ 维护简单

缺点:
⚠️ 需要加载 WASM (1-2MB)
⚠️ 首次搜索有延迟
```

---

## 🔧 实现细节

### 当前代码（问题代码）

```rust
// crates/mdbook-html/src/html_handlebars/search.rs

// 构建时使用 jieba-rs
fn tokenize(text: &str) -> Vec<String> {
    if is_cjk_text(text) {
        // ✅ Rust 分词
        JIEBA.cut_for_search(text, false)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        // 英文分词
        text.split_whitespace()
            .map(|s| s.to_lowercase())
            .collect()
    }
}
```

```javascript
// crates/mdbook-html/front-end/searcher/searcher.js

// 运行时搜索
function search(query) {
    // ❌ 没有分词！
    const results = searchindex.search(query, search_options);
    return results;
}
```

---

### 改进后代码（使用 WASM）

```rust
// crates/jieba-wasm/src/lib.rs

// 编译为 WASM 供浏览器使用
use jieba_rs::Jieba;
use wasm_bindgen::prelude::*;

lazy_static! {
    static ref JIEBA: Jieba = Jieba::new();
}

#[wasm_bindgen]
pub fn cut_for_search(text: &str) -> Vec<String> {
    JIEBA.cut_for_search(text, false)
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}

#[wasm_bindgen]
pub fn cut(text: &str) -> Vec<String> {
    JIEBA.cut(text, false)
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}
```

```javascript
// searcher.js (改进后)

let jiebaWasm = null;

// 按需加载 jieba WASM
async function loadJiebaWasm() {
    if (jiebaWasm) {
        return jiebaWasm;
    }
    
    const module = await import('./jieba_wasm.js');
    jiebaWasm = module;
    return module;
}

// 搜索时分词
async function search(query) {
    // ✅ 运行时也使用 jieba 分词
    if (isChinese(query)) {
        const jieba = await loadJiebaWasm();
        const tokens = jieba.cut_for_search(query);
        
        // 用分词后的结果搜索
        const results = searchindex.search(tokens.join(' '), search_options);
        return results;
    }
    
    // 英文直接搜索
    return searchindex.search(query, search_options);
}
```

---

## 📈 效果对比

### 测试用例

```
文档内容："Rust 的内存安全特性非常出色"
分词：["Rust", "内存", "安全", "特性", "非常", "出色"]
```

| 搜索词 | 当前方式 | 使用 WASM 后 |
|--------|---------|-------------|
| "内存" | ✅ 找到 | ✅ 找到 |
| "安全" | ✅ 找到 | ✅ 找到 |
| "内存安全" | ⚠️ 可能找到 | ✅ 找到 |
| "内存安全特性" | ❌ 找不到 | ✅ 找到 |
| "Rust 内存" | ⚠️ 可能找到 | ✅ 找到 |

---

## 💡 关键澄清

### 问题："之前不是通过编译成 wasm 提供功能的吗？"

**答案：不是！**

```
之前的架构:
┌────────────────────────────────────┐
│  构建时 (Rust 程序)                 │
│  ├── jieba-rs (原生 Rust 库) ✅     │
│  └── 生成 searchindex.js           │
└────────────────────────────────────┘

┌────────────────────────────────────┐
│  运行时 (浏览器)                    │
│  ├── elasticlunr.js (JavaScript)   │
│  └── ❌ 没有 WASM！                 │
└────────────────────────────────────┘

关键：jieba-rs 只在构建时使用，运行时没有分词！
```

---

### 新的架构（使用 WASM）

```
┌────────────────────────────────────┐
│  构建时 (Rust 程序)                 │
│  ├── jieba-rs (原生 Rust 库) ✅     │
│  └── 生成 searchindex.js           │
└────────────────────────────────────┘
              ↓
         同一套分词逻辑
              ↓
┌────────────────────────────────────┐
│  运行时 (浏览器)                    │
│  ├── jieba-rs WASM ✅              │
│  ├── elasticlunr.js                │
│  └── 分词后搜索                    │
└────────────────────────────────────┘

关键：构建时和运行时使用相同的分词逻辑！
```

---

## 🎯 总结

### 为什么需要 jieba-rs WASM？

| 原因 | 说明 |
|------|------|
| **分词一致性** | 构建时和运行时用同一套逻辑 |
| **搜索准确率** | 用户输入也能正确分词 |
| **代码复用** | Rust 代码，一处编写，多处使用 |
| **质量保证** | jieba-rs 是成熟的中文分词库 |

### 当前问题

- ❌ 构建时分词，运行时不分词
- ❌ 搜索词直接传递给索引，未分词
- ❌ 导致搜索准确率低

### 解决方案

- ✅ 添加 jieba-rs WASM
- ✅ 运行时也对搜索词分词
- ✅ 分词一致性，准确率提升

---

## 📝 下一步

如果你想改进搜索准确率，我可以帮你：

1. **创建 jieba-wasm crate**
2. **修改 searcher.js 支持运行时分析**
3. **测试搜索准确率提升**
4. **性能优化（按需加载）**

**你想开始吗？**
