#!/usr/bin/env python3
"""
Verify Chinese tokenization in mdBook search index.
This script analyzes the generated search index to verify that Chinese text
is properly tokenized using jieba.
"""

import json
import re
import sys
from pathlib import Path

def extract_json_from_js(js_content: str) -> dict:
    """Extract JSON content from the searchindex.js file."""
    # The format is: window.search = Object.assign(window.search, JSON.parse('...'));
    match = re.search(r"JSON\.parse\('(.+)'\)\)", js_content, re.DOTALL)
    if not match:
        raise ValueError("Could not find JSON content in searchindex.js")
    
    json_str = match.group(1)
    # Unescape the JSON string
    json_str = json_str.replace("\\'", "'")
    json_str = json_str.replace("\\\\", "\\")
    
    return json.loads(json_str)

def is_chinese(text: str) -> bool:
    """Check if text contains Chinese characters."""
    return bool(re.search(r'[\u4e00-\u9fff]', text))

def analyze_tokens(data: dict) -> dict:
    """Analyze the tokens in the search index."""
    # The index structure has invertedIndex which contains the actual tokens
    index = data.get('index', {})
    inverted_index = index.get('invertedIndex', {})
    
    all_tokens = set()
    chinese_tokens = set()
    english_tokens = set()
    
    # Analyze the inverted index to get actual indexed tokens
    for field_name, field_index in inverted_index.items():
        for token, doc_refs in field_index.items():
            all_tokens.add(token)
            if is_chinese(token):
                chinese_tokens.add(token)
            else:
                english_tokens.add(token)
    
    # Also check document store for reference
    docs = index.get('documentStore', {}).get('docs', {})
    
    return {
        'total_tokens': len(all_tokens),
        'chinese_tokens': sorted(chinese_tokens, key=len),
        'english_tokens': sorted(english_tokens),
        'chinese_token_count': len(chinese_tokens),
        'english_token_count': len(english_tokens),
        'all_tokens': sorted(all_tokens, key=len),
        'doc_count': len(docs),
    }

def check_tokenization_quality(analysis: dict) -> tuple[bool, list[str]]:
    """Check if the tokenization quality is good."""
    issues = []
    passed = True
    
    chinese_tokens = analysis['chinese_tokens']
    
    # Check 1: Should have Chinese tokens
    if analysis['chinese_token_count'] == 0:
        issues.append("❌ 没有检测到中文词汇 - 中文分词可能未生效")
        passed = False
    else:
        print(f"✅ 检测到 {analysis['chinese_token_count']} 个中文词汇")
    
    # Check 2: Tokens should not be too long (indicates poor segmentation)
    long_chinese_tokens = [t for t in chinese_tokens if len(t) > 20]
    if long_chinese_tokens:
        issues.append(f"⚠️ 发现 {len(long_chinese_tokens)} 个过长的中文 token（可能是整句未分词）")
        issues.extend(f"   - {t[:50]}..." for t in long_chinese_tokens[:5])
        passed = False
    else:
        print("✅ 中文词汇长度合理")
    
    # Check 3: Should have meaningful Chinese words (short tokens indicate proper segmentation)
    short_tokens = [t for t in chinese_tokens if 1 <= len(t) <= 4]
    if len(short_tokens) >= 10:
        print(f"✅ 检测到 {len(short_tokens)} 个短中文词汇（1-4 字），表明分词工作正常")
    else:
        issues.append(f"⚠️ 短中文词汇较少（{len(short_tokens)} 个），可能分词不够细致")
    
    # Check 4: Show sample tokens
    print("\n📋 中文词汇示例（按长度排序，前 50 个）:")
    for token in chinese_tokens[:50]:
        print(f"   - {token}")
    
    print("\n📋 短中文词汇示例（1-4 字）:")
    for token in short_tokens[:30]:
        print(f"   - {token}")
    
    return passed, issues

def main():
    # Find the search index file
    test_book_dir = Path('/home/cncsmonster/playground/mdbook-cn-playground/test-book/book')
    search_index_files = list(test_book_dir.glob('searchindex-*.js'))
    
    if not search_index_files:
        print("❌ 找不到搜索索引文件。请先构建测试书籍。")
        sys.exit(1)
    
    search_index_file = search_index_files[0]
    print(f"📖 正在分析搜索索引：{search_index_file.name}")
    print("=" * 60)
    
    # Read and parse the search index
    with open(search_index_file, 'r', encoding='utf-8') as f:
        content = f.read()
    
    try:
        data = extract_json_from_js(content)
    except Exception as e:
        print(f"❌ 解析搜索索引失败：{e}")
        sys.exit(1)
    
    # Analyze the tokens
    print("\n📊 索引统计:")
    
    analysis = analyze_tokens(data)
    print(f"   文档数量：{analysis['doc_count']}")
    print(f"   总 token 数：{analysis['total_tokens']}")
    print(f"   中文 token 数：{analysis['chinese_token_count']}")
    print(f"   英文 token 数：{analysis['english_token_count']}")
    
    # Check tokenization quality
    print("\n🔍 分词质量检查:")
    print("=" * 60)
    passed, issues = check_tokenization_quality(analysis)
    
    print("\n" + "=" * 60)
    if passed:
        print("✅ 中文分词验证通过！")
        print("\n💡 提示：可以在浏览器中打开 test-book/book/index.html 来测试搜索功能。")
        print("   尝试搜索以下关键词：")
        print("   - 中文")
        print("   - Rust")
        print("   - 分词")
        print("   - 内存安全")
    else:
        print("❌ 中文分词验证失败！")
        print("\n问题列表:")
        for issue in issues:
            print(issue)
        sys.exit(1)

if __name__ == '__main__':
    main()
