use super::static_files::StaticFiles;
use crate::html::{ChapterTree, Node};
use crate::theme::searcher;
use crate::utils::ToUrlPath;
use anyhow::{Result, bail};
use ego_tree::iter::Edge;
use elasticlunr::{Index, IndexBuilder, Pipeline};
use jieba_rs::Jieba;
use lazy_static::lazy_static;
use mdbook_core::book::Chapter;
use mdbook_core::config::{Search, SearchChapterSettings};
use mdbook_core::static_regex;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

const MAX_WORD_LENGTH_TO_INDEX: usize = 80;

// Chinese stop words for search
const CHINESE_STOP_WORDS: &[&str] = &[
    "的", "了", "是", "在", "就", "都", "而", "及", "与", "着",
    "或", "一个", "没有", "我们", "你们", "他们", "它", "她", "他",
    "这", "那", "之", "用", "以", "为", "对", "将", "从", "到",
    "和", "或", "等", "个", "也", "不", "你", "我", "他", "她",
    "它", "其", "此", "该", "该", "此", "这些", "那些", "什么",
    "怎么", "如何", "为什么", "吗", "呢", "吧", "啊", "呀", "哦",
];

lazy_static! {
    static ref JIEBA: Jieba = Jieba::new();
}

/// Custom language implementation for CJK support.
/// Uses jieba for tokenization and has no pipeline (no stemming/trimming).
struct CjkLanguage;

impl elasticlunr::lang::Language for CjkLanguage {
    fn name(&self) -> String {
        "CJK".to_string()
    }

    fn code(&self) -> String {
        "cjk".to_string()
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        tokenize(text)
    }

    fn make_pipeline(&self) -> Pipeline {
        // Return an empty pipeline to preserve Chinese characters
        // The default English pipeline includes a stemmer that removes non-ASCII chars
        Pipeline { queue: vec![] }
    }
}

/// Detects if text contains CJK (Chinese, Japanese, Korean) characters.
fn is_cjk_text(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c,
            '\u{4e00}'..='\u{9fff}' |  // CJK Unified Ideographs
            '\u{3400}'..='\u{4dbf}' |  // CJK Extension A
            '\u{f900}'..='\u{faff}' |  // CJK Compatibility Ideographs
            '\u{3040}'..='\u{30ff}' |  // Hiragana/Katakana
            '\u{ac00}'..='\u{d7af}' |  // Hangul Syllables
            '\u{3100}'..='\u{312f}' |  // Bopomofo
            '\u{31a0}'..='\u{31bf}'    // Bopomofo Extended
        )
    })
}

/// Tokenizes text with support for both CJK and Latin languages.
/// For CJK text, uses jieba for word segmentation with multiple modes for better coverage.
/// For Latin text, splits on whitespace and hyphens.
fn tokenize(text: &str) -> Vec<String> {
    if is_cjk_text(text) {
        let mut tokens = Vec::new();
        
        // Use jieba cut_for_search for better search coverage
        let jieba_tokens: Vec<String> = JIEBA
            .cut_for_search(text, false)
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty() && s.len() <= MAX_WORD_LENGTH_TO_INDEX && !is_stop_word(s))
            .collect();
        
        // Also add character-level bigrams for better search recall
        let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
        for i in 0..chars.len().saturating_sub(1) {
            let bigram: String = chars[i..i+2].iter().collect();
            if bigram.len() == 2 && !is_stop_word(&bigram) && !tokens.contains(&bigram) {
                tokens.push(bigram.to_lowercase());
            }
        }
        
        // Combine both tokenization methods
        tokens.extend(jieba_tokens);
        tokens.sort();
        tokens.dedup();
        tokens
    } else {
        // Tokenization for Latin-based languages with punctuation stripping
        // Since we disabled the default pipeline, we need to handle trimming ourselves
        text.split(|c: char| c.is_whitespace() || c == '-')
            .filter(|s| !s.is_empty())
            .map(|s| {
                // Strip leading and trailing punctuation while preserving internal punctuation
                // like in programming language constructs (e.g., println!, fn(), etc.)
                s.trim_matches(|c: char| c.is_ascii_punctuation() && c != '_' && c != '!')
                  .to_lowercase()
            })
            .filter(|s| !s.is_empty() && s.len() <= MAX_WORD_LENGTH_TO_INDEX)
            .collect()
    }
}

/// Checks if a word is a Chinese stop word.
fn is_stop_word(word: &str) -> bool {
    CHINESE_STOP_WORDS.contains(&word)
}

/// Creates all files required for search.
pub(super) fn create_files(
    search_config: &Search,
    static_files: &mut StaticFiles,
    chapter_trees: &[ChapterTree<'_>],
) -> Result<()> {
    // Build index with custom CJK language support
    // This uses jieba for tokenization and has no pipeline to preserve Chinese characters
    let mut index = IndexBuilder::with_language(Box::new(CjkLanguage))
        .add_field_with_tokenizer("title", Box::new(&tokenize))
        .add_field_with_tokenizer("body", Box::new(&tokenize))
        .add_field_with_tokenizer("breadcrumbs", Box::new(&tokenize))
        .build();

    // These are links to all of the headings in all of the chapters.
    let mut doc_urls = Vec::new();

    let chapter_configs = sort_search_config(&search_config.chapter);
    validate_chapter_config(&chapter_configs, chapter_trees)?;

    for ct in chapter_trees {
        let path = settings_path(ct.chapter);
        let chapter_settings = get_chapter_settings(&chapter_configs, path);
        if !chapter_settings.enable.unwrap_or(true) {
            continue;
        }
        index_chapter(&mut index, search_config, &mut doc_urls, ct)?;
    }

    let index = write_to_json(index, search_config, doc_urls)?;
    debug!("Writing search index ✓");
    if index.len() > 10_000_000 {
        warn!("search index is very large ({} bytes)", index.len());
    }

    if search_config.copy_js {
        static_files.add_builtin(
            "searchindex.js",
            // To reduce the size of the generated JSON by preventing all `"` characters to be
            // escaped, we instead surround the string with much less common `'` character.
            format!(
                "window.search = Object.assign(window.search, JSON.parse('{}'));",
                index.replace("\\", "\\\\").replace("'", "\\'")
            )
            .as_bytes(),
        );
        static_files.add_builtin("searcher.js", searcher::JS);
        static_files.add_builtin("mark.min.js", searcher::MARK_JS);
        static_files.add_builtin("elasticlunr.min.js", searcher::ELASTICLUNR_JS);
        debug!("Copying search files ✓");
    }

    Ok(())
}

/// Uses the given arguments to construct a search document, then inserts it to the given index.
fn add_doc(
    index: &mut Index,
    doc_urls: &mut Vec<String>,
    anchor_base: &str,
    heading_id: &str,
    items: &[&str],
) {
    let mut url = anchor_base.to_string();
    if !heading_id.is_empty() {
        url.push('#');
        url.push_str(heading_id);
    }

    let doc_ref = doc_urls.len().to_string();
    doc_urls.push(url);

    let items = items.iter().map(|&x| collapse_whitespace(x.trim()));
    index.add_doc(&doc_ref, items);
}

/// Adds the chapter to the search index.
fn index_chapter(
    index: &mut Index,
    search_config: &Search,
    doc_urls: &mut Vec<String>,
    chapter_tree: &ChapterTree<'_>,
) -> Result<()> {
    let anchor_base = chapter_tree.html_path.to_url_path();

    let mut in_heading = false;
    let max_section_depth = search_config.heading_split_level;
    let mut section_id = None;
    let mut heading = String::new();
    let mut body = String::new();
    let mut breadcrumbs = chapter_tree.chapter.parent_names.clone();

    breadcrumbs.push(chapter_tree.chapter.name.clone());

    let mut traverse = chapter_tree.tree.root().traverse();

    while let Some(edge) = traverse.next() {
        match edge {
            Edge::Open(node) => match node.value() {
                Node::Element(el) => {
                    if let Some(level) = el.heading_level()
                        && level <= max_section_depth
                        && let Some(heading_id) = el.attr("id")
                    {
                        if !heading.is_empty() {
                            // Section finished, the next heading is following now
                            // Write the data to the index, and clear it for the next section
                            add_doc(
                                index,
                                doc_urls,
                                &anchor_base,
                                section_id.unwrap(),
                                &[&heading, &body, &breadcrumbs.join(" » ")],
                            );
                            heading.clear();
                            body.clear();
                            breadcrumbs.pop();
                        }
                        section_id = Some(heading_id);
                        in_heading = true;
                    } else if matches!(el.name(), "script" | "style") {
                        // Skip this node.
                        while let Some(edge) = traverse.next() {
                            if let Edge::Close(close) = edge
                                && close == node
                            {
                                break;
                            }
                        }
                    // Insert spaces where HTML output would usually separate text
                    // to ensure words don't get merged together
                    } else if in_heading {
                        heading.push(' ');
                    } else {
                        body.push(' ');
                    }
                }
                Node::Text(text) => {
                    if in_heading {
                        heading.push_str(text);
                    } else {
                        body.push_str(text);
                    }
                }
                Node::Comment(_) => {}
                Node::Fragment => {}
                Node::RawData(_) => {}
            },
            Edge::Close(node) => match node.value() {
                Node::Element(el) => {
                    if let Some(level) = el.heading_level()
                        && level <= max_section_depth
                    {
                        in_heading = false;
                        breadcrumbs.push(heading.clone());
                    }
                }
                _ => {}
            },
        }
    }

    if !body.is_empty() || !heading.is_empty() {
        // Make sure the last section is added to the index
        let title = if heading.is_empty() {
            if let Some(chapter) = breadcrumbs.first() {
                chapter
            } else {
                ""
            }
        } else {
            &heading
        };
        add_doc(
            index,
            doc_urls,
            &anchor_base,
            section_id.unwrap_or_default(),
            &[title, &body, &breadcrumbs.join(" » ")],
        );
    }

    Ok(())
}

fn write_to_json(index: Index, search_config: &Search, doc_urls: Vec<String>) -> Result<String> {
    use elasticlunr::config::{SearchBool, SearchOptions, SearchOptionsField};
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    struct ResultsOptions {
        limit_results: u32,
        teaser_word_count: u32,
    }

    #[derive(Serialize)]
    struct SearchindexJson {
        /// The options used for displaying search results
        results_options: ResultsOptions,
        /// The searchoptions for elasticlunr.js
        search_options: SearchOptions,
        /// Used to lookup a document's URL from an integer document ref.
        doc_urls: Vec<String>,
        /// The index for elasticlunr.js
        index: elasticlunr::Index,
    }

    let mut fields = BTreeMap::new();
    let mut opt = SearchOptionsField::default();
    let mut insert_boost = |key: &str, boost| {
        opt.boost = Some(boost);
        fields.insert(key.into(), opt);
    };
    insert_boost("title", search_config.boost_title);
    insert_boost("body", search_config.boost_paragraph);
    insert_boost("breadcrumbs", search_config.boost_hierarchy);

    let search_options = SearchOptions {
        bool: if search_config.use_boolean_and {
            SearchBool::And
        } else {
            SearchBool::Or
        },
        expand: search_config.expand,
        fields,
    };

    let results_options = ResultsOptions {
        limit_results: search_config.limit_results,
        teaser_word_count: search_config.teaser_word_count,
    };

    let json_contents = SearchindexJson {
        results_options,
        search_options,
        doc_urls,
        index,
    };

    // By converting to serde_json::Value as an intermediary, we use a
    // BTreeMap internally and can force a stable ordering of map keys.
    let json_contents = serde_json::to_value(&json_contents)?;
    let json_contents = serde_json::to_string(&json_contents)?;

    Ok(json_contents)
}

fn settings_path(ch: &Chapter) -> &Path {
    ch.source_path
        .as_deref()
        .unwrap_or_else(|| ch.path.as_deref().unwrap())
}

fn validate_chapter_config(
    chapter_configs: &[(PathBuf, SearchChapterSettings)],
    chapter_trees: &[ChapterTree<'_>],
) -> Result<()> {
    for (path, _) in chapter_configs {
        let found = chapter_trees
            .iter()
            .any(|ct| settings_path(ct.chapter).starts_with(path));
        if !found {
            bail!(
                "[output.html.search.chapter] key `{}` does not match any chapter paths",
                path.display()
            );
        }
    }
    Ok(())
}

fn sort_search_config(
    map: &HashMap<String, SearchChapterSettings>,
) -> Vec<(PathBuf, SearchChapterSettings)> {
    let mut settings: Vec<_> = map
        .iter()
        .map(|(key, value)| (PathBuf::from(key), value.clone()))
        .collect();
    // Note: This is case-sensitive, and assumes the author uses the same case
    // as the actual filename.
    settings.sort_by(|a, b| a.0.cmp(&b.0));
    settings
}

fn get_chapter_settings(
    chapter_configs: &[(PathBuf, SearchChapterSettings)],
    source_path: &Path,
) -> SearchChapterSettings {
    let mut result = SearchChapterSettings::default();
    for (path, config) in chapter_configs {
        if source_path.starts_with(path) {
            result.enable = config.enable.or(result.enable);
        }
    }
    result
}

/// Replaces multiple consecutive whitespace characters with a single space character.
fn collapse_whitespace(text: &str) -> Cow<'_, str> {
    static_regex!(WS, r"\s\s+");
    WS.replace_all(text, " ")
}

#[test]
fn chapter_settings_priority() {
    let cfg = r#"
        [output.html.search.chapter]
        "cli/watch.md" = { enable = true }
        "cli" = { enable = false }
        "cli/inner/foo.md" = { enable = false }
        "cli/inner" = { enable = true }
        "foo" = {} # Just to make sure empty table is allowed.
    "#;
    let cfg: mdbook_core::config::Config = toml::from_str(cfg).unwrap();
    let html = cfg.html_config().unwrap();
    let chapter_configs = sort_search_config(&html.search.unwrap().chapter);
    for (path, enable) in [
        ("foo.md", None),
        ("cli/watch.md", Some(true)),
        ("cli/index.md", Some(false)),
        ("cli/inner/index.md", Some(true)),
        ("cli/inner/foo.md", Some(false)),
    ] {
        let mut settings = SearchChapterSettings::default();
        settings.enable = enable;
        assert_eq!(
            get_chapter_settings(&chapter_configs, Path::new(path)),
            settings
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        assert_eq!(tokenize("hello world"), vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_with_hyphens() {
        assert_eq!(
            tokenize("hello-world test-case"),
            vec!["hello", "world", "test", "case"]
        );
    }

    #[test]
    fn test_tokenize_mixed_whitespace() {
        assert_eq!(
            tokenize("hello\tworld\ntest\r\ncase"),
            vec!["hello", "world", "test", "case"]
        );
    }

    #[test]
    fn test_tokenize_empty_string() {
        assert_eq!(tokenize(""), Vec::<String>::new());
    }

    #[test]
    fn test_tokenize_only_whitespace() {
        assert_eq!(tokenize("   \t\n  "), Vec::<String>::new());
    }

    #[test]
    fn test_tokenize_case_normalization() {
        assert_eq!(tokenize("Hello WORLD Test"), vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_tokenize_trim_whitespace() {
        assert_eq!(tokenize("  hello   world  "), vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_long_words_filtered() {
        let long_word = "a".repeat(MAX_WORD_LENGTH_TO_INDEX + 1);
        let short_word = "a".repeat(MAX_WORD_LENGTH_TO_INDEX);
        let input = format!("{} hello {}", long_word, short_word);
        assert_eq!(tokenize(&input), vec!["hello", &short_word]);
    }

    #[test]
    fn test_tokenize_max_length_word() {
        let max_word = "a".repeat(MAX_WORD_LENGTH_TO_INDEX);
        assert_eq!(tokenize(&max_word), vec![max_word]);
    }

    #[test]
    fn test_tokenize_special_characters() {
        // Punctuation at the start/end is stripped, but internal punctuation is preserved
        // This matches the behavior of the original pipeline trimmer
        assert_eq!(
            tokenize("hello,world.test!case?"),
            vec!["hello,world.test!case"]
        );
    }

    #[test]
    fn test_tokenize_unicode() {
        assert_eq!(
            tokenize("café naïve résumé"),
            vec!["café", "naïve", "résumé"]
        );
    }

    #[test]
    fn test_tokenize_unicode_rtl_hebre() {
        assert_eq!(tokenize("שלום עולם"), vec!["שלום", "עולם"]);
    }

    #[test]
    fn test_tokenize_numbers() {
        assert_eq!(
            tokenize("test123 456-789 hello"),
            vec!["test123", "456", "789", "hello"]
        );
    }

    #[test]
    fn test_tokenize_chinese_basic() {
        // Basic Chinese text should be tokenized by jieba
        let result = tokenize("你好世界");
        assert!(!result.is_empty());
        // Should contain at least some meaningful tokens
        assert!(result.iter().any(|s| s.contains("你好") || s.contains("世界") || s.contains("好世")));
    }

    #[test]
    fn test_tokenize_chinese_sentence() {
        // Chinese sentence should be properly segmented
        let result = tokenize("中华人民共和国是中国");
        assert!(!result.is_empty());
        // Should contain key terms
        assert!(result.iter().any(|s| s.contains("中国")));
    }

    #[test]
    fn test_tokenize_chinese_with_stop_words() {
        // Stop words should be filtered out
        let result = tokenize("这是一个测试");
        // "是" and "一个" are stop words, should be filtered
        assert!(!result.contains(&"是".to_string()));
        assert!(!result.contains(&"一个".to_string()));
    }

    #[test]
    fn test_tokenize_mixed_chinese_english() {
        // Mixed Chinese and English should be handled correctly
        let result = tokenize("你好 world 世界");
        assert!(!result.is_empty());
        // Should contain both Chinese and English tokens
        assert!(result.iter().any(|s| s == "world"));
    }

    #[test]
    fn test_tokenize_chinese_technical_terms() {
        // Technical terms with English should work
        let result = tokenize("Rust 编程语言");
        assert!(!result.is_empty());
        // Should contain "Rust" or Chinese terms
        assert!(result.iter().any(|s| s.to_lowercase() == "rust" || s.contains("编程") || s.contains("语言")));
    }

    #[test]
    fn test_is_cjk_text() {
        assert!(is_cjk_text("你好"));
        assert!(is_cjk_text("こんにちは"));
        assert!(is_cjk_text("안녕하세요"));
        assert!(!is_cjk_text("hello"));
        assert!(!is_cjk_text("café"));
    }

    #[test]
    fn test_is_stop_word() {
        assert!(is_stop_word("的"));
        assert!(is_stop_word("了"));
        assert!(is_stop_word("是"));
        assert!(!is_stop_word("中国"));
        assert!(!is_stop_word("hello"));
    }
}
