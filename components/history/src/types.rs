use serde::{Deserialize, Serialize};

/// Unique identifier for history entries
pub type VisitId = String;

/// History visit entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryVisit {
    pub id: VisitId,
    pub url: String,
    pub title: String,
    pub visit_time: i64,  // Unix timestamp
    pub visit_duration: Option<i64>,  // Seconds
    pub from_url: Option<String>,  // Referrer
    pub transition_type: TransitionType,
}

/// Page transition type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransitionType {
    Link,           // Clicked a link
    Typed,          // Typed in address bar
    Reload,         // Reloaded page
    Bookmark,       // From bookmark
    Redirect,       // HTTP redirect
    FormSubmit,     // Submitted a form
}

impl TransitionType {
    /// Convert to string representation for database storage
    pub fn to_str(&self) -> &'static str {
        match self {
            TransitionType::Link => "link",
            TransitionType::Typed => "typed",
            TransitionType::Reload => "reload",
            TransitionType::Bookmark => "bookmark",
            TransitionType::Redirect => "redirect",
            TransitionType::FormSubmit => "form_submit",
        }
    }

    /// Parse from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "link" => Some(TransitionType::Link),
            "typed" => Some(TransitionType::Typed),
            "reload" => Some(TransitionType::Reload),
            "bookmark" => Some(TransitionType::Bookmark),
            "redirect" => Some(TransitionType::Redirect),
            "form_submit" => Some(TransitionType::FormSubmit),
            _ => None,
        }
    }
}

/// Aggregated page statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageStats {
    pub url: String,
    pub title: String,
    pub visit_count: usize,
    pub last_visit: i64,
    pub frecency_score: f64,  // Frequency + Recency score
}

/// History search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub limit: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: None,
            start_time: None,
            end_time: None,
            limit: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_type_to_str() {
        assert_eq!(TransitionType::Link.to_str(), "link");
        assert_eq!(TransitionType::Typed.to_str(), "typed");
        assert_eq!(TransitionType::Reload.to_str(), "reload");
        assert_eq!(TransitionType::Bookmark.to_str(), "bookmark");
        assert_eq!(TransitionType::Redirect.to_str(), "redirect");
        assert_eq!(TransitionType::FormSubmit.to_str(), "form_submit");
    }

    #[test]
    fn test_transition_type_from_str() {
        assert_eq!(TransitionType::from_str("link"), Some(TransitionType::Link));
        assert_eq!(TransitionType::from_str("typed"), Some(TransitionType::Typed));
        assert_eq!(TransitionType::from_str("reload"), Some(TransitionType::Reload));
        assert_eq!(TransitionType::from_str("bookmark"), Some(TransitionType::Bookmark));
        assert_eq!(TransitionType::from_str("redirect"), Some(TransitionType::Redirect));
        assert_eq!(TransitionType::from_str("form_submit"), Some(TransitionType::FormSubmit));
        assert_eq!(TransitionType::from_str("invalid"), None);
    }

    #[test]
    fn test_search_query_default() {
        let query = SearchQuery::default();
        assert_eq!(query.text, None);
        assert_eq!(query.start_time, None);
        assert_eq!(query.end_time, None);
        assert_eq!(query.limit, 100);
    }
}
