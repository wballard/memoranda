use chrono::{DateTime, Utc};
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write;
use tracing::warn;

use super::models::{Memo, MemoId};
use crate::config::Settings;

// These constants are now configurable - see Settings struct
// Kept as fallback defaults if Settings is not available
const FALLBACK_RECENCY_BOOST_DAYS: f64 = 365.0;
const FALLBACK_SNIPPET_LENGTH: usize = 100;
const FALLBACK_SNIPPET_CONTEXT_PADDING: usize = 2;

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub recency_boost_days: f64,
    pub snippet_length: usize,
    pub snippet_context_padding: usize,
}

impl From<&Settings> for SearchConfig {
    fn from(settings: &Settings) -> Self {
        Self {
            recency_boost_days: settings.search_recency_boost_days,
            snippet_length: settings.search_snippet_length,
            snippet_context_padding: settings.search_snippet_context_padding,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SearchOperator {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone)]
pub enum SearchTerm {
    Word(String),
    Phrase(String),
    Wildcard(String),
    Boolean {
        left: Box<SearchTerm>,
        operator: SearchOperator,
        right: Box<SearchTerm>,
    },
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub terms: Vec<String>,
    pub phrase: Option<String>,
    pub tags: Vec<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub regex: Option<String>,
    pub title_only: bool,
    pub content_only: bool,
    pub boolean_query: Option<SearchTerm>,
}

impl SearchQuery {
    #[must_use]
    pub fn new() -> Self {
        Self {
            terms: Vec::new(),
            phrase: None,
            tags: Vec::new(),
            date_from: None,
            date_to: None,
            regex: None,
            title_only: false,
            content_only: false,
            boolean_query: None,
        }
    }

    #[must_use]
    pub fn with_terms(terms: Vec<String>) -> Self {
        Self {
            terms,
            phrase: None,
            tags: Vec::new(),
            date_from: None,
            date_to: None,
            regex: None,
            title_only: false,
            content_only: false,
            boolean_query: None,
        }
    }

    #[must_use]
    pub fn with_phrase(phrase: String) -> Self {
        Self {
            terms: Vec::new(),
            phrase: Some(phrase),
            tags: Vec::new(),
            date_from: None,
            date_to: None,
            regex: None,
            title_only: false,
            content_only: false,
            boolean_query: None,
        }
    }

    #[must_use]
    pub fn with_tags(tags: Vec<String>) -> Self {
        Self {
            terms: Vec::new(),
            phrase: None,
            tags,
            date_from: None,
            date_to: None,
            regex: None,
            title_only: false,
            content_only: false,
            boolean_query: None,
        }
    }

    #[must_use]
    pub fn with_boolean_query(boolean_query: SearchTerm) -> Self {
        Self {
            terms: Vec::new(),
            phrase: None,
            tags: Vec::new(),
            date_from: None,
            date_to: None,
            regex: None,
            title_only: false,
            content_only: false,
            boolean_query: Some(boolean_query),
        }
    }

    pub fn parse_query(query: &str) -> Self {
        let mut search_query = SearchQuery::new();

        // Simple parser for basic query formats
        if query.contains(" AND ") || query.contains(" OR ") || query.contains(" NOT ") {
            if let Some(boolean_query) = Self::parse_boolean_query(query) {
                search_query.boolean_query = Some(boolean_query);
            }
        } else if query.starts_with('"') && query.ends_with('"') {
            // Phrase query
            let phrase = query.trim_matches('"').to_string();
            search_query.phrase = Some(phrase);
        } else if query.contains('*') || query.contains('?') {
            // Wildcard query
            let wildcard_term = SearchTerm::Wildcard(query.to_string());
            search_query.boolean_query = Some(wildcard_term);
        } else {
            // Simple term search
            search_query.terms = query.split_whitespace().map(|s| s.to_string()).collect();
        }

        search_query
    }

    fn parse_boolean_query(query: &str) -> Option<SearchTerm> {
        // Simple boolean query parser
        // This is a basic implementation and could be improved with proper parsing
        if let Some(and_pos) = query.find(" AND ") {
            let left = query[..and_pos].trim();
            let right = query[and_pos + 5..].trim();

            return Some(SearchTerm::Boolean {
                left: Box::new(Self::parse_term(left)),
                operator: SearchOperator::And,
                right: Box::new(Self::parse_term(right)),
            });
        }

        if let Some(or_pos) = query.find(" OR ") {
            let left = query[..or_pos].trim();
            let right = query[or_pos + 4..].trim();

            return Some(SearchTerm::Boolean {
                left: Box::new(Self::parse_term(left)),
                operator: SearchOperator::Or,
                right: Box::new(Self::parse_term(right)),
            });
        }

        if let Some(not_pos) = query.find(" NOT ") {
            let left = query[..not_pos].trim();
            let right = query[not_pos + 5..].trim();

            return Some(SearchTerm::Boolean {
                left: Box::new(Self::parse_term(left)),
                operator: SearchOperator::Not,
                right: Box::new(Self::parse_term(right)),
            });
        }

        None
    }

    fn parse_term(term: &str) -> SearchTerm {
        let term = term.trim();

        if term.starts_with('"') && term.ends_with('"') {
            SearchTerm::Phrase(term.trim_matches('"').to_string())
        } else if term.contains('*') || term.contains('?') {
            SearchTerm::Wildcard(term.to_string())
        } else {
            SearchTerm::Word(term.to_string())
        }
    }
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub memo: Memo,
    pub score: f64,
    pub snippets: Vec<String>,
    pub title_matches: Vec<String>,
    pub content_matches: Vec<String>,
}

impl SearchResult {
    pub fn new(memo: Memo, score: f64) -> Self {
        Self {
            memo,
            score,
            snippets: Vec::new(),
            title_matches: Vec::new(),
            content_matches: Vec::new(),
        }
    }
}

impl PartialEq for SearchResult {
    fn eq(&self, other: &Self) -> bool {
        self.memo.id == other.memo.id
    }
}

impl Eq for SearchResult {}

impl PartialOrd for SearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by score descending, then by created_at descending
        match other.score.partial_cmp(&self.score) {
            Some(Ordering::Equal) => other.memo.created_at.cmp(&self.memo.created_at),
            Some(ordering) => ordering,
            None => Ordering::Equal,
        }
    }
}

#[derive(Debug)]
pub struct MemoSearcher {
    index: HashMap<String, Vec<MemoId>>,
}

impl MemoSearcher {
    #[must_use]
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    pub fn index_memo(&mut self, memo: &Memo) {
        let tokens = self.tokenize_text(&format!("{} {}", memo.title, memo.content));

        for token in tokens {
            self.index
                .entry(token.to_lowercase())
                .or_default()
                .push(memo.id);
        }
    }

    pub fn search(&self, query: &SearchQuery, memos: &[Memo]) -> Vec<SearchResult> {
        // Use fallback constants for backward compatibility
        let config = SearchConfig {
            recency_boost_days: FALLBACK_RECENCY_BOOST_DAYS,
            snippet_length: FALLBACK_SNIPPET_LENGTH,
            snippet_context_padding: FALLBACK_SNIPPET_CONTEXT_PADDING,
        };
        self.search_with_config(query, memos, &config)
    }

    pub fn search_with_config(
        &self,
        query: &SearchQuery,
        memos: &[Memo],
        config: &SearchConfig,
    ) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for memo in memos {
            if let Some(score) = self.score_memo_with_config(memo, query, config) {
                let mut result = SearchResult::new(memo.clone(), score);
                self.add_snippets_with_config(&mut result, query, config);
                results.push(result);
            }
        }

        results.sort();
        results
    }

    pub fn get_all_context(&self, memos: &[Memo]) -> String {
        let mut context = String::new();

        for memo in memos {
            let _ = write!(
                context,
                "# {}\n\n**Created:** {}\n**Updated:** {}\n**Tags:** {}\n\n{}\n\n---\n\n",
                memo.title,
                memo.created_at.format("%Y-%m-%d %H:%M:%S"),
                memo.updated_at.format("%Y-%m-%d %H:%M:%S"),
                memo.tags.join(", "),
                memo.content
            );
        }

        context
    }

    fn tokenize_text(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    #[allow(dead_code)]
    fn score_memo(&self, memo: &Memo, query: &SearchQuery) -> Option<f64> {
        let config = SearchConfig {
            recency_boost_days: FALLBACK_RECENCY_BOOST_DAYS,
            snippet_length: FALLBACK_SNIPPET_LENGTH,
            snippet_context_padding: FALLBACK_SNIPPET_CONTEXT_PADDING,
        };
        self.score_memo_with_config(memo, query, &config)
    }

    fn score_memo_with_config(
        &self,
        memo: &Memo,
        query: &SearchQuery,
        config: &SearchConfig,
    ) -> Option<f64> {
        let mut score = 0.0;
        let mut matches = false;

        // Term matching
        if !query.terms.is_empty() {
            for term in &query.terms {
                let (term_score, term_matches) = self.score_term_match(memo, term, 2.0, 1.0);
                score += term_score;
                if term_matches {
                    matches = true;
                }
            }
        }

        // Phrase matching
        if let Some(phrase) = &query.phrase {
            let (phrase_score, phrase_matches) = self.score_term_match(memo, phrase, 3.0, 1.5);
            score += phrase_score;
            if phrase_matches {
                matches = true;
            }
        }

        // Tag matching
        if !query.tags.is_empty() {
            for tag in &query.tags {
                if memo.tags.contains(tag) {
                    score += 1.0;
                    matches = true;
                }
            }
        }

        // Date filtering
        if let Some(date_from) = query.date_from {
            if memo.created_at < date_from {
                return None;
            }
        }

        if let Some(date_to) = query.date_to {
            if memo.created_at > date_to {
                return None;
            }
        }

        // Regex matching
        if let Some(regex_pattern) = &query.regex {
            match Regex::new(regex_pattern) {
                Ok(regex) => {
                    let search_text = format!("{} {}", memo.title, memo.content);
                    if regex.is_match(&search_text) {
                        score += 1.0;
                        matches = true;
                    }
                }
                Err(e) => {
                    warn!("Failed to compile regex pattern '{}': {}", regex_pattern, e);
                }
            }
        }

        // Boolean query matching
        if let Some(boolean_query) = &query.boolean_query {
            if let Some(boolean_score) = self.evaluate_boolean_term(memo, boolean_query) {
                score += boolean_score;
                matches = true;
            }
        }

        // Apply recency boost
        if matches {
            let days_since_creation = (Utc::now() - memo.created_at).num_days();
            let recency_boost =
                1.0 / (1.0 + days_since_creation as f64 / config.recency_boost_days);
            score *= 1.0 + recency_boost;

            Some(score)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn add_snippets(&self, result: &mut SearchResult, query: &SearchQuery) {
        let config = SearchConfig {
            recency_boost_days: FALLBACK_RECENCY_BOOST_DAYS,
            snippet_length: FALLBACK_SNIPPET_LENGTH,
            snippet_context_padding: FALLBACK_SNIPPET_CONTEXT_PADDING,
        };
        self.add_snippets_with_config(result, query, &config)
    }

    fn add_snippets_with_config(
        &self,
        result: &mut SearchResult,
        query: &SearchQuery,
        config: &SearchConfig,
    ) {
        if !query.terms.is_empty() {
            for term in &query.terms {
                if let Some(snippet) = self.extract_snippet_with_config(
                    &result.memo.content,
                    term,
                    config.snippet_length,
                    config.snippet_context_padding,
                ) {
                    result.snippets.push(snippet);
                }
            }
        }

        if let Some(phrase) = &query.phrase {
            if let Some(snippet) = self.extract_snippet_with_config(
                &result.memo.content,
                phrase,
                config.snippet_length,
                config.snippet_context_padding,
            ) {
                result.snippets.push(snippet);
            }
        }

        // Remove duplicates
        result.snippets.sort();
        result.snippets.dedup();
    }

    #[allow(dead_code)]
    fn extract_snippet(&self, content: &str, term: &str, max_length: usize) -> Option<String> {
        self.extract_snippet_with_config(
            content,
            term,
            max_length,
            FALLBACK_SNIPPET_CONTEXT_PADDING,
        )
    }

    fn extract_snippet_with_config(
        &self,
        content: &str,
        term: &str,
        max_length: usize,
        context_padding: usize,
    ) -> Option<String> {
        let term_lower = term.to_lowercase();
        let content_lower = content.to_lowercase();

        if let Some(pos) = content_lower.find(&term_lower) {
            let start = pos.saturating_sub(max_length / context_padding);
            let end = std::cmp::min(
                content.len(),
                pos + term.len() + max_length / context_padding,
            );

            let snippet = &content[start..end];
            Some(format!("...{snippet}..."))
        } else {
            None
        }
    }

    fn evaluate_boolean_term(&self, memo: &Memo, term: &SearchTerm) -> Option<f64> {
        match term {
            SearchTerm::Word(word) => self.score_term_match_optional(memo, word, 2.0, 1.0),
            SearchTerm::Phrase(phrase) => self.score_term_match_optional(memo, phrase, 3.0, 1.5),
            SearchTerm::Wildcard(pattern) => {
                let regex_pattern = self.wildcard_to_regex(pattern);
                match Regex::new(&regex_pattern) {
                    Ok(regex) => {
                        let search_text = format!("{} {}", memo.title, memo.content);
                        if regex.is_match(&search_text) {
                            Some(1.0)
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to compile wildcard pattern '{}' as regex '{}': {}",
                            pattern, regex_pattern, e
                        );
                        None
                    }
                }
            }
            SearchTerm::Boolean {
                left,
                operator,
                right,
            } => {
                let left_score = self.evaluate_boolean_term(memo, left);
                let right_score = self.evaluate_boolean_term(memo, right);

                match operator {
                    SearchOperator::And => match (left_score, right_score) {
                        (Some(l), Some(r)) => Some(l + r),
                        _ => None,
                    },
                    SearchOperator::Or => match (left_score, right_score) {
                        (Some(l), Some(r)) => Some(l + r),
                        (Some(l), None) => Some(l),
                        (None, Some(r)) => Some(r),
                        (None, None) => None,
                    },
                    SearchOperator::Not => {
                        if left_score.is_some() && right_score.is_none() {
                            left_score
                        } else {
                            None
                        }
                    }
                }
            }
        }
    }

    fn wildcard_to_regex(&self, pattern: &str) -> String {
        let mut regex = String::new();
        regex.push_str("(?i)"); // Case insensitive

        for ch in pattern.chars() {
            match ch {
                '*' => regex.push_str(".*"),
                '?' => regex.push('.'),
                c if c.is_alphanumeric() => regex.push(c),
                c => {
                    regex.push('\\');
                    regex.push(c);
                }
            }
        }

        regex
    }

    /// Helper method to score a term match against a memo
    fn score_term_match(
        &self,
        memo: &Memo,
        term: &str,
        title_score: f64,
        content_score: f64,
    ) -> (f64, bool) {
        let term_lower = term.to_lowercase();
        let title_lower = memo.title.to_lowercase();
        let content_lower = memo.content.to_lowercase();

        let mut score = 0.0;
        let mut matches = false;

        if title_lower.contains(&term_lower) {
            score += title_score;
            matches = true;
        }

        if content_lower.contains(&term_lower) {
            score += content_score;
            matches = true;
        }

        (score, matches)
    }

    /// Helper method to score a term match and return Option<f64> for boolean evaluation
    fn score_term_match_optional(
        &self,
        memo: &Memo,
        term: &str,
        title_score: f64,
        content_score: f64,
    ) -> Option<f64> {
        let term_lower = term.to_lowercase();
        let title_lower = memo.title.to_lowercase();
        let content_lower = memo.content.to_lowercase();

        if title_lower.contains(&term_lower) {
            Some(title_score)
        } else if content_lower.contains(&term_lower) {
            Some(content_score)
        } else {
            None
        }
    }
}

impl Default for MemoSearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_memo(title: &str, content: &str) -> Memo {
        Memo::new(title.to_string(), content.to_string()).unwrap()
    }

    fn create_test_memo_with_tags(title: &str, content: &str, tags: Vec<String>) -> Memo {
        let mut memo = create_test_memo(title, content);
        for tag in tags {
            memo.add_tag(tag);
        }
        memo
    }

    #[test]
    fn test_search_query_creation() {
        let query = SearchQuery::new();
        assert!(query.terms.is_empty());
        assert!(query.phrase.is_none());
        assert!(query.tags.is_empty());
        assert!(query.date_from.is_none());
        assert!(query.date_to.is_none());
        assert!(query.regex.is_none());
        assert!(!query.title_only);
        assert!(!query.content_only);
    }

    #[test]
    fn test_search_query_with_terms() {
        let terms = vec!["rust".to_string(), "programming".to_string()];
        let query = SearchQuery::with_terms(terms.clone());
        assert_eq!(query.terms, terms);
    }

    #[test]
    fn test_search_query_with_phrase() {
        let phrase = "hello world".to_string();
        let query = SearchQuery::with_phrase(phrase.clone());
        assert_eq!(query.phrase, Some(phrase));
    }

    #[test]
    fn test_search_query_with_tags() {
        let tags = vec!["rust".to_string(), "programming".to_string()];
        let query = SearchQuery::with_tags(tags.clone());
        assert_eq!(query.tags, tags);
    }

    #[test]
    fn test_search_result_creation() {
        let memo = create_test_memo("Test", "Content");
        let result = SearchResult::new(memo.clone(), 1.0);
        assert_eq!(result.memo.id, memo.id);
        assert_eq!(result.score, 1.0);
        assert!(result.snippets.is_empty());
    }

    #[test]
    fn test_search_result_ordering() {
        let memo1 = create_test_memo("Test1", "Content1");
        let memo2 = create_test_memo("Test2", "Content2");

        let result1 = SearchResult::new(memo1, 1.0);
        let result2 = SearchResult::new(memo2, 2.0);

        let mut results = vec![result1, result2];
        results.sort();

        assert_eq!(results[0].score, 2.0);
        assert_eq!(results[1].score, 1.0);
    }

    #[test]
    fn test_memo_searcher_creation() {
        let searcher = MemoSearcher::new();
        assert!(searcher.index.is_empty());
    }

    #[test]
    fn test_memo_searcher_index_memo() {
        let mut searcher = MemoSearcher::new();
        let memo = create_test_memo("Test Title", "Test content");

        searcher.index_memo(&memo);

        assert!(searcher.index.contains_key("test"));
        assert!(searcher.index.contains_key("title"));
        assert!(searcher.index.contains_key("content"));
    }

    #[test]
    fn test_memo_searcher_search_terms() {
        let mut searcher = MemoSearcher::new();
        let memo1 = create_test_memo("Rust Programming", "Learning Rust language");
        let memo2 = create_test_memo("Python Guide", "Python programming tutorial");

        searcher.index_memo(&memo1);
        searcher.index_memo(&memo2);

        let memos = vec![memo1.clone(), memo2.clone()];
        let query = SearchQuery::with_terms(vec!["rust".to_string()]);
        let results = searcher.search(&query, &memos);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memo.id, memo1.id);
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn test_memo_searcher_search_phrase() {
        let mut searcher = MemoSearcher::new();
        let memo1 = create_test_memo("Test", "hello world example");
        let memo2 = create_test_memo("Test", "hello there world");

        searcher.index_memo(&memo1);
        searcher.index_memo(&memo2);

        let memos = vec![memo1.clone(), memo2.clone()];
        let query = SearchQuery::with_phrase("hello world".to_string());
        let results = searcher.search(&query, &memos);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memo.id, memo1.id);
    }

    #[test]
    fn test_memo_searcher_search_tags() {
        let mut searcher = MemoSearcher::new();
        let memo1 = create_test_memo_with_tags("Test1", "Content1", vec!["rust".to_string()]);
        let memo2 = create_test_memo_with_tags("Test2", "Content2", vec!["python".to_string()]);

        searcher.index_memo(&memo1);
        searcher.index_memo(&memo2);

        let memos = vec![memo1.clone(), memo2.clone()];
        let query = SearchQuery::with_tags(vec!["rust".to_string()]);
        let results = searcher.search(&query, &memos);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memo.id, memo1.id);
    }

    #[test]
    fn test_memo_searcher_get_all_context() {
        let searcher = MemoSearcher::new();
        let memo1 =
            create_test_memo_with_tags("First Memo", "First content", vec!["tag1".to_string()]);
        let memo2 =
            create_test_memo_with_tags("Second Memo", "Second content", vec!["tag2".to_string()]);

        let memos = vec![memo1, memo2];
        let context = searcher.get_all_context(&memos);

        assert!(context.contains("# First Memo"));
        assert!(context.contains("# Second Memo"));
        assert!(context.contains("First content"));
        assert!(context.contains("Second content"));
        assert!(context.contains("tag1"));
        assert!(context.contains("tag2"));
        assert!(context.contains("Created:"));
        assert!(context.contains("Updated:"));
        assert!(context.contains("Tags:"));
    }

    #[test]
    fn test_tokenize_text() {
        let searcher = MemoSearcher::new();
        let tokens = searcher.tokenize_text("Hello, world! This is a test.");

        assert!(tokens.contains(&"Hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"This".to_string()));
        assert!(tokens.contains(&"is".to_string()));
        assert!(tokens.contains(&"a".to_string()));
        assert!(tokens.contains(&"test".to_string()));
    }

    #[test]
    fn test_extract_snippet() {
        let searcher = MemoSearcher::new();
        let content =
            "This is a long piece of content that contains the word rust in the middle of it.";
        let snippet = searcher.extract_snippet(content, "rust", 20);

        assert!(snippet.is_some());
        let snippet = snippet.unwrap();
        assert!(snippet.contains("rust"));
        assert!(snippet.starts_with("..."));
        assert!(snippet.ends_with("..."));
    }

    #[test]
    fn test_score_memo_with_title_match() {
        let searcher = MemoSearcher::new();
        let memo = create_test_memo("Rust Programming", "Learning languages");
        let query = SearchQuery::with_terms(vec!["rust".to_string()]);

        let score = searcher.score_memo(&memo, &query);
        assert!(score.is_some());
        assert!(score.unwrap() > 0.0);
    }

    #[test]
    fn test_score_memo_with_content_match() {
        let searcher = MemoSearcher::new();
        let memo = create_test_memo("Programming", "Learning rust language");
        let query = SearchQuery::with_terms(vec!["rust".to_string()]);

        let score = searcher.score_memo(&memo, &query);
        assert!(score.is_some());
        assert!(score.unwrap() > 0.0);
    }

    #[test]
    fn test_score_memo_no_match() {
        let searcher = MemoSearcher::new();
        let memo = create_test_memo("Python", "Learning python language");
        let query = SearchQuery::with_terms(vec!["rust".to_string()]);

        let score = searcher.score_memo(&memo, &query);
        assert!(score.is_none());
    }

    #[test]
    fn test_search_query_parse_simple() {
        let query = SearchQuery::parse_query("rust programming");
        assert_eq!(
            query.terms,
            vec!["rust".to_string(), "programming".to_string()]
        );
        assert!(query.phrase.is_none());
        assert!(query.boolean_query.is_none());
    }

    #[test]
    fn test_search_query_parse_phrase() {
        let query = SearchQuery::parse_query("\"hello world\"");
        assert_eq!(query.phrase, Some("hello world".to_string()));
        assert!(query.terms.is_empty());
    }

    #[test]
    fn test_search_query_parse_wildcard() {
        let query = SearchQuery::parse_query("rust*");
        assert!(query.boolean_query.is_some());
        match query.boolean_query.unwrap() {
            SearchTerm::Wildcard(pattern) => assert_eq!(pattern, "rust*"),
            _ => panic!("Expected wildcard term"),
        }
    }

    #[test]
    fn test_search_query_parse_boolean_and() {
        let query = SearchQuery::parse_query("rust AND programming");
        assert!(query.boolean_query.is_some());
        match query.boolean_query.unwrap() {
            SearchTerm::Boolean {
                left,
                operator,
                right,
            } => {
                match operator {
                    SearchOperator::And => {}
                    _ => panic!("Expected AND operator"),
                }
                match (*left, *right) {
                    (SearchTerm::Word(l), SearchTerm::Word(r)) => {
                        assert_eq!(l, "rust");
                        assert_eq!(r, "programming");
                    }
                    _ => panic!("Expected word terms"),
                }
            }
            _ => panic!("Expected boolean term"),
        }
    }

    #[test]
    fn test_search_query_parse_boolean_or() {
        let query = SearchQuery::parse_query("rust OR python");
        assert!(query.boolean_query.is_some());
        match query.boolean_query.unwrap() {
            SearchTerm::Boolean {
                left,
                operator,
                right,
            } => {
                match operator {
                    SearchOperator::Or => {}
                    _ => panic!("Expected OR operator"),
                }
                match (*left, *right) {
                    (SearchTerm::Word(l), SearchTerm::Word(r)) => {
                        assert_eq!(l, "rust");
                        assert_eq!(r, "python");
                    }
                    _ => panic!("Expected word terms"),
                }
            }
            _ => panic!("Expected boolean term"),
        }
    }

    #[test]
    fn test_memo_searcher_boolean_and_search() {
        let searcher = MemoSearcher::new();
        let memo1 = create_test_memo("Rust Programming", "Learning rust language");
        let memo2 = create_test_memo("Python", "Learning python language");

        let memos = vec![memo1.clone(), memo2.clone()];
        let query = SearchQuery::parse_query("rust AND programming");
        let results = searcher.search(&query, &memos);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memo.id, memo1.id);
    }

    #[test]
    fn test_memo_searcher_boolean_or_search() {
        let searcher = MemoSearcher::new();
        let memo1 = create_test_memo("Rust Programming", "Learning rust language");
        let memo2 = create_test_memo("Python", "Learning python language");
        let memo3 = create_test_memo("JavaScript", "Learning javascript");

        let memos = vec![memo1.clone(), memo2.clone(), memo3.clone()];
        let query = SearchQuery::parse_query("rust OR python");
        let results = searcher.search(&query, &memos);

        assert_eq!(results.len(), 2);
        // Results should contain both memo1 and memo2
        let result_ids: Vec<_> = results.iter().map(|r| r.memo.id).collect();
        assert!(result_ids.contains(&memo1.id));
        assert!(result_ids.contains(&memo2.id));
    }

    #[test]
    fn test_memo_searcher_wildcard_search() {
        let searcher = MemoSearcher::new();
        let memo1 = create_test_memo("Rust Programming", "Learning rust language");
        let memo2 = create_test_memo("Python", "Learning python language");

        let memos = vec![memo1.clone(), memo2.clone()];
        let query = SearchQuery::parse_query("rust*");
        let results = searcher.search(&query, &memos);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].memo.id, memo1.id);
    }

    #[test]
    fn test_wildcard_to_regex() {
        let searcher = MemoSearcher::new();

        assert_eq!(searcher.wildcard_to_regex("rust*"), "(?i)rust.*");
        assert_eq!(searcher.wildcard_to_regex("rust?"), "(?i)rust.");
        assert_eq!(searcher.wildcard_to_regex("rust*ing"), "(?i)rust.*ing");
        assert_eq!(searcher.wildcard_to_regex("*rust"), "(?i).*rust");
    }

    #[test]
    fn test_evaluate_boolean_term_word() {
        let searcher = MemoSearcher::new();
        let memo = create_test_memo("Rust Programming", "Learning rust language");
        let term = SearchTerm::Word("rust".to_string());

        let score = searcher.evaluate_boolean_term(&memo, &term);
        assert!(score.is_some());
        assert_eq!(score.unwrap(), 2.0); // Title match
    }

    #[test]
    fn test_evaluate_boolean_term_phrase() {
        let searcher = MemoSearcher::new();
        let memo = create_test_memo("Hello World", "This is a hello world example");
        let term = SearchTerm::Phrase("hello world".to_string());

        let score = searcher.evaluate_boolean_term(&memo, &term);
        assert!(score.is_some());
        assert_eq!(score.unwrap(), 3.0); // Title phrase match
    }

    #[test]
    fn test_evaluate_boolean_term_wildcard() {
        let searcher = MemoSearcher::new();
        let memo = create_test_memo("Rust Programming", "Learning rust language");
        let term = SearchTerm::Wildcard("rust*".to_string());

        let score = searcher.evaluate_boolean_term(&memo, &term);
        assert!(score.is_some());
        assert_eq!(score.unwrap(), 1.0); // Wildcard match
    }
}
