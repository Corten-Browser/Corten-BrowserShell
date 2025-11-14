// @implements: REQ-003
//! Navigation history management
//!
//! Manages back/forward navigation stacks for tabs.

use shared_types::Url;
use std::collections::VecDeque;

/// Navigation history for a single tab
#[derive(Debug, Clone)]
pub struct NavigationHistory {
    back_stack: VecDeque<Url>,
    forward_stack: VecDeque<Url>,
    current: Option<Url>,
}

impl NavigationHistory {
    /// Create a new navigation history
    pub fn new() -> Self {
        Self {
            back_stack: VecDeque::new(),
            forward_stack: VecDeque::new(),
            current: None,
        }
    }

    /// Navigate to a new URL
    pub fn navigate(&mut self, url: Url) {
        // Move current to back stack
        if let Some(current_url) = self.current.take() {
            self.back_stack.push_back(current_url);
        }

        // Clear forward stack (new navigation invalidates forward history)
        self.forward_stack.clear();

        // Set new current URL
        self.current = Some(url);
    }

    /// Go back in history
    pub fn go_back(&mut self) -> Option<Url> {
        if let Some(prev_url) = self.back_stack.pop_back() {
            // Move current to forward stack
            if let Some(current_url) = self.current.replace(prev_url.clone()) {
                self.forward_stack.push_front(current_url);
            }
            Some(prev_url)
        } else {
            None
        }
    }

    /// Go forward in history
    pub fn go_forward(&mut self) -> Option<Url> {
        if let Some(next_url) = self.forward_stack.pop_front() {
            // Move current to back stack
            if let Some(current_url) = self.current.replace(next_url.clone()) {
                self.back_stack.push_back(current_url);
            }
            Some(next_url)
        } else {
            None
        }
    }

    /// Check if can go back
    pub fn can_go_back(&self) -> bool {
        !self.back_stack.is_empty()
    }

    /// Check if can go forward
    pub fn can_go_forward(&self) -> bool {
        !self.forward_stack.is_empty()
    }

    /// Get current URL
    pub fn current_url(&self) -> Option<&Url> {
        self.current.as_ref()
    }
}

impl Default for NavigationHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history_has_no_navigation() {
        let history = NavigationHistory::new();
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
        assert!(history.current_url().is_none());
    }

    #[test]
    fn test_navigate_sets_current_url() {
        let mut history = NavigationHistory::new();
        let url = Url::parse("https://example.com").unwrap();
        history.navigate(url);

        assert_eq!(history.current_url().map(|u| u.as_str()), Some("https://example.com"));
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());
    }

    #[test]
    fn test_second_navigate_enables_back() {
        let mut history = NavigationHistory::new();
        let url1 = Url::parse("https://example.com").unwrap();
        let url2 = Url::parse("https://example.com/page2").unwrap();

        history.navigate(url1);
        history.navigate(url2);

        assert!(history.can_go_back());
        assert!(!history.can_go_forward());
        assert_eq!(history.current_url().map(|u| u.as_str()), Some("https://example.com/page2"));
    }

    #[test]
    fn test_go_back_returns_previous_url() {
        let mut history = NavigationHistory::new();
        let url1 = Url::parse("https://example.com").unwrap();
        let url2 = Url::parse("https://example.com/page2").unwrap();

        history.navigate(url1.clone());
        history.navigate(url2);

        let back_url = history.go_back().unwrap();
        assert_eq!(back_url.as_str(), "https://example.com");
        assert_eq!(history.current_url().map(|u| u.as_str()), Some("https://example.com"));
    }

    #[test]
    fn test_go_back_enables_forward() {
        let mut history = NavigationHistory::new();
        let url1 = Url::parse("https://example.com").unwrap();
        let url2 = Url::parse("https://example.com/page2").unwrap();

        history.navigate(url1);
        history.navigate(url2);
        history.go_back();

        assert!(history.can_go_forward());
    }

    #[test]
    fn test_navigate_clears_forward_stack() {
        let mut history = NavigationHistory::new();
        let url1 = Url::parse("https://example.com").unwrap();
        let url2 = Url::parse("https://example.com/page2").unwrap();
        let url3 = Url::parse("https://example.com/page3").unwrap();

        history.navigate(url1);
        history.navigate(url2);
        history.go_back();

        assert!(history.can_go_forward());

        history.navigate(url3);

        assert!(!history.can_go_forward());
    }
}
