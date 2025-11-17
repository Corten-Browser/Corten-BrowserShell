//! Find in Page Component
//!
//! Provides text search functionality within page content.
//! Supports highlighting matches, navigation between matches,
//! and case sensitivity options.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum FindError {
    #[error("No active search")]
    NoActiveSearch,
    #[error("Pattern is empty")]
    EmptyPattern,
    #[error("Invalid regex pattern: {0}")]
    InvalidPattern(String),
    #[error("No matches found")]
    NoMatches,
}

pub type Result<T> = std::result::Result<T, FindError>;

/// A single match in the page content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Match {
    /// Start position in the content
    pub start: usize,
    /// End position in the content
    pub end: usize,
    /// The matched text
    pub text: String,
    /// Context around the match (for preview)
    pub context: String,
}

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindOptions {
    /// Case-sensitive search
    pub case_sensitive: bool,
    /// Use regular expression
    pub use_regex: bool,
    /// Whole word match only
    pub whole_word: bool,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            use_regex: false,
            whole_word: false,
        }
    }
}

/// Search state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindState {
    /// Current search query
    pub query: String,
    /// All matches found
    pub matches: Vec<Match>,
    /// Current match index (0-based)
    pub current_index: usize,
    /// Search options
    pub options: FindOptions,
}

/// Find in Page manager
pub struct FindInPage {
    /// Current content being searched
    content: Arc<RwLock<String>>,
    /// Current search state
    state: Arc<RwLock<Option<FindState>>>,
}

impl FindInPage {
    /// Create a new FindInPage instance
    pub fn new() -> Self {
        Self {
            content: Arc::new(RwLock::new(String::new())),
            state: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the content to search within
    pub async fn set_content(&self, content: String) {
        let mut current_content = self.content.write().await;
        *current_content = content;
        // Clear previous search state when content changes
        let mut state = self.state.write().await;
        *state = None;
    }

    /// Get the current content
    pub async fn get_content(&self) -> String {
        self.content.read().await.clone()
    }

    /// Start a new search
    pub async fn find(&self, query: String, options: FindOptions) -> Result<FindState> {
        if query.is_empty() {
            return Err(FindError::EmptyPattern);
        }

        let content = self.content.read().await;
        let matches = self.find_matches(&content, &query, &options)?;

        if matches.is_empty() {
            let state = FindState {
                query: query.clone(),
                matches: vec![],
                current_index: 0,
                options,
            };
            let mut current_state = self.state.write().await;
            *current_state = Some(state.clone());
            return Ok(state);
        }

        let state = FindState {
            query,
            matches,
            current_index: 0,
            options,
        };

        let mut current_state = self.state.write().await;
        *current_state = Some(state.clone());

        Ok(state)
    }

    fn find_matches(
        &self,
        content: &str,
        query: &str,
        options: &FindOptions,
    ) -> Result<Vec<Match>> {
        let mut matches = Vec::new();

        if options.use_regex {
            let pattern = if options.case_sensitive {
                query.to_string()
            } else {
                format!("(?i){}", query)
            };

            let re = regex::Regex::new(&pattern)
                .map_err(|e| FindError::InvalidPattern(e.to_string()))?;

            for mat in re.find_iter(content) {
                let context = self.extract_context(content, mat.start(), mat.end());
                matches.push(Match {
                    start: mat.start(),
                    end: mat.end(),
                    text: mat.as_str().to_string(),
                    context,
                });
            }
        } else {
            let search_content = if options.case_sensitive {
                content.to_string()
            } else {
                content.to_lowercase()
            };

            let search_query = if options.case_sensitive {
                query.to_string()
            } else {
                query.to_lowercase()
            };

            let mut start = 0;
            while let Some(pos) = search_content[start..].find(&search_query) {
                let abs_pos = start + pos;
                let end_pos = abs_pos + query.len();

                // Check whole word boundary if required
                if options.whole_word {
                    let is_word_start =
                        abs_pos == 0 || !content.chars().nth(abs_pos - 1).unwrap().is_alphanumeric();
                    let is_word_end = end_pos == content.len()
                        || !content.chars().nth(end_pos).unwrap().is_alphanumeric();

                    if !is_word_start || !is_word_end {
                        start = abs_pos + 1;
                        continue;
                    }
                }

                let matched_text = content[abs_pos..end_pos].to_string();
                let context = self.extract_context(content, abs_pos, end_pos);

                matches.push(Match {
                    start: abs_pos,
                    end: end_pos,
                    text: matched_text,
                    context,
                });

                start = end_pos;
            }
        }

        Ok(matches)
    }

    fn extract_context(&self, content: &str, start: usize, end: usize) -> String {
        let context_size = 30;
        let content_start = start.saturating_sub(context_size);
        let content_end = (end + context_size).min(content.len());

        let mut context = String::new();
        if content_start > 0 {
            context.push_str("...");
        }
        context.push_str(&content[content_start..content_end]);
        if content_end < content.len() {
            context.push_str("...");
        }

        context
    }

    /// Navigate to next match
    pub async fn find_next(&self) -> Result<Match> {
        let mut state = self.state.write().await;
        let state = state.as_mut().ok_or(FindError::NoActiveSearch)?;

        if state.matches.is_empty() {
            return Err(FindError::NoMatches);
        }

        state.current_index = (state.current_index + 1) % state.matches.len();
        Ok(state.matches[state.current_index].clone())
    }

    /// Navigate to previous match
    pub async fn find_previous(&self) -> Result<Match> {
        let mut state = self.state.write().await;
        let state = state.as_mut().ok_or(FindError::NoActiveSearch)?;

        if state.matches.is_empty() {
            return Err(FindError::NoMatches);
        }

        if state.current_index == 0 {
            state.current_index = state.matches.len() - 1;
        } else {
            state.current_index -= 1;
        }

        Ok(state.matches[state.current_index].clone())
    }

    /// Get current match
    pub async fn get_current_match(&self) -> Result<Match> {
        let state = self.state.read().await;
        let state = state.as_ref().ok_or(FindError::NoActiveSearch)?;

        if state.matches.is_empty() {
            return Err(FindError::NoMatches);
        }

        Ok(state.matches[state.current_index].clone())
    }

    /// Get current search state
    pub async fn get_state(&self) -> Option<FindState> {
        self.state.read().await.clone()
    }

    /// Get match count
    pub async fn get_match_count(&self) -> Result<usize> {
        let state = self.state.read().await;
        let state = state.as_ref().ok_or(FindError::NoActiveSearch)?;
        Ok(state.matches.len())
    }

    /// Clear current search
    pub async fn clear_search(&self) {
        let mut state = self.state.write().await;
        *state = None;
    }

    /// Replace current match with new text
    pub async fn replace_current(&self, replacement: &str) -> Result<String> {
        let state_guard = self.state.read().await;
        let state = state_guard.as_ref().ok_or(FindError::NoActiveSearch)?;

        if state.matches.is_empty() {
            return Err(FindError::NoMatches);
        }

        let current_match = &state.matches[state.current_index];
        let content = self.content.read().await;

        let mut new_content = String::new();
        new_content.push_str(&content[..current_match.start]);
        new_content.push_str(replacement);
        new_content.push_str(&content[current_match.end..]);

        drop(state_guard);
        drop(content);

        self.set_content(new_content.clone()).await;
        Ok(new_content)
    }

    /// Replace all matches with new text
    pub async fn replace_all(&self, replacement: &str) -> Result<(String, usize)> {
        let state_guard = self.state.read().await;
        let state = state_guard.as_ref().ok_or(FindError::NoActiveSearch)?;

        if state.matches.is_empty() {
            return Err(FindError::NoMatches);
        }

        let content = self.content.read().await;
        let mut new_content = content.clone();
        let count = state.matches.len();

        // Replace from end to start to preserve positions
        for mat in state.matches.iter().rev() {
            new_content.replace_range(mat.start..mat.end, replacement);
        }

        drop(state_guard);
        drop(content);

        self.set_content(new_content.clone()).await;
        Ok((new_content, count))
    }
}

impl Default for FindInPage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_find() {
        let finder = FindInPage::new();
        finder
            .set_content("Hello world, hello universe".to_string())
            .await;

        let state = finder
            .find("hello".to_string(), FindOptions::default())
            .await
            .unwrap();

        assert_eq!(state.matches.len(), 2);
        assert_eq!(state.matches[0].text, "Hello");
        assert_eq!(state.matches[1].text, "hello");
    }

    #[tokio::test]
    async fn test_case_sensitive_find() {
        let finder = FindInPage::new();
        finder
            .set_content("Hello world, hello universe".to_string())
            .await;

        let state = finder
            .find(
                "hello".to_string(),
                FindOptions {
                    case_sensitive: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(state.matches.len(), 1);
        assert_eq!(state.matches[0].text, "hello");
    }

    #[tokio::test]
    async fn test_whole_word_find() {
        let finder = FindInPage::new();
        finder
            .set_content("test testing tested test".to_string())
            .await;

        let state = finder
            .find(
                "test".to_string(),
                FindOptions {
                    whole_word: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(state.matches.len(), 2);
    }

    #[tokio::test]
    async fn test_regex_find() {
        let finder = FindInPage::new();
        finder
            .set_content("cat bat rat hat".to_string())
            .await;

        let state = finder
            .find(
                r"\b\w+at\b".to_string(),
                FindOptions {
                    use_regex: true,
                    case_sensitive: true,
                    ..Default::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(state.matches.len(), 4);
    }

    #[tokio::test]
    async fn test_find_next_previous() {
        let finder = FindInPage::new();
        finder
            .set_content("one two one three one".to_string())
            .await;

        finder
            .find("one".to_string(), FindOptions::default())
            .await
            .unwrap();

        let first = finder.get_current_match().await.unwrap();
        assert_eq!(first.start, 0);

        let second = finder.find_next().await.unwrap();
        assert_eq!(second.start, 8);

        let third = finder.find_next().await.unwrap();
        assert_eq!(third.start, 18);

        // Wrap around
        let back_to_first = finder.find_next().await.unwrap();
        assert_eq!(back_to_first.start, 0);

        // Go back
        let back_to_third = finder.find_previous().await.unwrap();
        assert_eq!(back_to_third.start, 18);
    }

    #[tokio::test]
    async fn test_no_matches() {
        let finder = FindInPage::new();
        finder
            .set_content("Hello world".to_string())
            .await;

        let state = finder
            .find("xyz".to_string(), FindOptions::default())
            .await
            .unwrap();

        assert_eq!(state.matches.len(), 0);
    }

    #[tokio::test]
    async fn test_empty_pattern_error() {
        let finder = FindInPage::new();
        finder.set_content("Hello world".to_string()).await;

        let result = finder.find("".to_string(), FindOptions::default()).await;
        assert!(matches!(result, Err(FindError::EmptyPattern)));
    }

    #[tokio::test]
    async fn test_replace_current() {
        let finder = FindInPage::new();
        finder.set_content("foo bar foo".to_string()).await;

        finder
            .find("foo".to_string(), FindOptions::default())
            .await
            .unwrap();

        let new_content = finder.replace_current("baz").await.unwrap();
        assert_eq!(new_content, "baz bar foo");
    }

    #[tokio::test]
    async fn test_replace_all() {
        let finder = FindInPage::new();
        finder.set_content("foo bar foo baz foo".to_string()).await;

        finder
            .find("foo".to_string(), FindOptions::default())
            .await
            .unwrap();

        let (new_content, count) = finder.replace_all("qux").await.unwrap();
        assert_eq!(new_content, "qux bar qux baz qux");
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_clear_search() {
        let finder = FindInPage::new();
        finder.set_content("Hello world".to_string()).await;

        finder
            .find("Hello".to_string(), FindOptions::default())
            .await
            .unwrap();

        finder.clear_search().await;

        let state = finder.get_state().await;
        assert!(state.is_none());
    }

    #[tokio::test]
    async fn test_match_context() {
        let finder = FindInPage::new();
        let long_text = "This is a very long string with the word target somewhere in the middle of it all";
        finder.set_content(long_text.to_string()).await;

        let state = finder
            .find("target".to_string(), FindOptions::default())
            .await
            .unwrap();

        assert_eq!(state.matches.len(), 1);
        assert!(state.matches[0].context.contains("target"));
        assert!(state.matches[0].context.contains("..."));
    }
}
