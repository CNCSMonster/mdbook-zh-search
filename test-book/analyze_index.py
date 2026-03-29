#!/usr/bin/env python3
"""Analyze the search index structure properly."""

import json
import re
import sys

def extract_json_from_js(js_content: str) -> dict:
    """Extract JSON content from the searchindex.js file."""
    match = re.search(r"JSON\.parse\('(.+)'\)\)", js_content, re.DOTALL)
    if not match:
        raise ValueError("Could not find JSON content")
    
    json_str = match.group(1)
    json_str = json_str.replace("\\'", "'")
    json_str = json_str.replace("\\\\", "\\")
    
    return json.loads(json_str)

def extract_tokens_from_trie(node, prefix='', tokens=None):
    """Recursively extract tokens from elasticlunr trie structure."""
    if tokens is None:
        tokens = set()
    
    if not isinstance(node, dict):
        return tokens
    
    # If this node has 'df' (document frequency), it's a terminal node
    if 'df' in node and node['df'] > 0:
        # Remove 'root' prefix from token
        clean_token = prefix[4:] if prefix.startswith('root') else prefix
        tokens.add(clean_token)
    
    for key, value in node.items():
        if key not in ['df', 'docs', '$']:
            extract_tokens_from_trie(value, prefix + key, tokens)
    
    return tokens

def main():
    import glob
    search_index_files = glob.glob('/home/cncsmonster/playground/mdbook-cn-playground/test-book/book/searchindex-*.js')
    if not search_index_files:
        print("❌ 找不到搜索索引文件")
        return
    search_index_file = search_index_files[0]
    
    with open(search_index_file, 'r', encoding='utf-8') as f:
        content = f.read()
    
    data = extract_json_from_js(content)
    
    print("=" * 60)
    print("搜索索引分析")
    print("=" * 60)
    
    # Document URLs
    doc_urls = data.get('doc_urls', [])
    print(f"\n文档数量：{len(doc_urls)}")
    print("文档列表:")
    for url in doc_urls[:10]:
        print(f"  - {url}")
    
    # Index structure
    index = data.get('index', {})
    fields = index.get('fields', [])
    print(f"\n索引字段：{fields}")
    
    # Extract tokens from each field's trie
    trie_index = index.get('index', {})
    all_tokens = set()
    
    for field_name in fields:
        field_trie = trie_index.get(field_name, {})
        field_tokens = extract_tokens_from_trie(field_trie)
        print(f"\n字段 '{field_name}' 的 token 数量：{len(field_tokens)}")
        all_tokens.update(field_tokens)
    
    # Analyze Chinese tokens
    chinese_tokens = [t for t in all_tokens if any('\u4e00' <= c <= '\u9fff' for c in t)]
    english_tokens = [t for t in all_tokens if t.isascii()]
    
    print("\n" + "=" * 60)
    print("Token 分析")
    print("=" * 60)
    print(f"\n总 token 数：{len(all_tokens)}")
    print(f"中文 token 数：{len(chinese_tokens)}")
    print(f"英文 token 数：{len(english_tokens)}")
    
    if chinese_tokens:
        print("\n中文 token 示例（按长度排序）:")
        for t in sorted(chinese_tokens, key=len)[:50]:
            print(f"  [{len(t):2d}] {t}")
    else:
        print("\n⚠️ 未检测到中文 token!")
        print("\n所有 token 列表:")
        for t in sorted(all_tokens, key=len)[:100]:
            print(f"  {t}")
    
    # Check document store
    docs = index.get('documentStore', {}).get('docs', {})
    print("\n" + "=" * 60)
    print("文档存储检查")
    print("=" * 60)
    print(f"存储的文档数：{len(docs)}")
    
    # Check if docs have Chinese content
    has_chinese = False
    for doc_id, doc in docs.items():
        body = doc.get('body', '')
        if any('\u4e00' <= c <= '\u9fff' for c in body):
            has_chinese = True
            print(f"\n文档 {doc_id} 包含中文内容:")
            print(f"  标题：{doc.get('title', 'N/A')}")
            print(f"  内容预览：{body[:100]}...")
            break
    
    if not has_chinese:
        print("⚠️ 文档存储中没有中文内容!")
    
    print("\n" + "=" * 60)
    if chinese_tokens and len(chinese_tokens) > 0:
        print("✅ 中文搜索索引验证成功!")
    else:
        print("❌ 中文搜索索引验证失败 - 没有中文 token 被索引")
        print("\n可能的原因:")
        print("1. 分词器没有正确识别中文")
        print("2. elasticlunr-rs 的 trie 结构存储方式特殊")
        print("3. 需要检查 tokenize 函数的输出")

if __name__ == '__main__':
    main()
