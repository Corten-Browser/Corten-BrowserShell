# history Feature Component

## ⚠️ VERSION CONTROL RESTRICTIONS
**FORBIDDEN ACTIONS:**
- ❌ NEVER change project version to 1.0.0
- ❌ NEVER declare system "production ready"
- ❌ NEVER change lifecycle_state

**ALLOWED:**
- ✅ Report test coverage and quality metrics
- ✅ Complete your component work
- ✅ Suggest improvements

You are a specialized agent building ONLY the history feature component.

## Component Overview

**Purpose**: Browsing history tracking and management system

**Responsibilities**:
- Visit tracking (URL, title, timestamp)
- Search and filtering
- Visit frequency and recency scoring
- Privacy controls (clear history, incognito mode)
- SQLite-based persistence
- Async operations using Tokio

**Technology Stack**:
- Rust 2021 edition
- Tokio for async runtime
- rusqlite for SQLite database
- serde for serialization
- anyhow for error handling
- parking_lot for synchronization

**Dependencies**:
- shared_types (Level 0)
- user_data (Level 2) - for storage patterns

## MANDATORY: Test-Driven Development (TDD)

### TDD is Not Optional

**ALL code in this component MUST be developed using TDD.** This is a strict requirement, not a suggestion.

### TDD Workflow (Red-Green-Refactor)

**You MUST follow this cycle for EVERY feature:**

1. **RED**: Write a failing test
   - Write the test FIRST before any implementation code
   - Run the test and verify it FAILS
   - The test defines the behavior you want

2. **GREEN**: Make the test pass
   - Write the MINIMUM code needed to pass the test
   - Don't add extra features
   - Run the test and verify it PASSES

3. **REFACTOR**: Improve the code
   - Clean up duplication
   - Improve naming and structure
   - Maintain passing tests throughout

### TDD Commit Pattern

Your git history MUST show TDD practice:

```bash
# Commit sequence example
git commit -m "[history] test: Add failing test for visit recording"
git commit -m "[history] feat: Implement visit recording to pass test"
git commit -m "[history] refactor: Extract visit scoring logic"
```

## API Specification

### HistoryManager

```rust
use async_trait::async_trait;
use anyhow::Result;
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

/// Aggregated page statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// History manager interface
pub struct HistoryManager {
    storage: HistoryStorage,
}

impl HistoryManager {
    /// Create new history manager with database path
    pub async fn new(db_path: &str) -> Result<Self>;

    /// Record a visit
    pub async fn record_visit(&mut self, visit: HistoryVisit) -> Result<VisitId>;

    /// Get visit by ID
    pub async fn get_visit(&self, id: &VisitId) -> Result<Option<HistoryVisit>>;

    /// Delete visit
    pub async fn delete_visit(&mut self, id: &VisitId) -> Result<()>;

    /// Get all visits for a URL
    pub async fn get_visits_for_url(&self, url: &str) -> Result<Vec<HistoryVisit>>;

    /// Search history
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<HistoryVisit>>;

    /// Get most visited pages
    pub async fn get_most_visited(&self, limit: usize) -> Result<Vec<PageStats>>;

    /// Get recent visits
    pub async fn get_recent(&self, limit: usize) -> Result<Vec<HistoryVisit>>;

    /// Get frecency-scored pages (frequency + recency)
    pub async fn get_frecent(&self, limit: usize) -> Result<Vec<PageStats>>;

    /// Clear history older than timestamp
    pub async fn clear_older_than(&mut self, timestamp: i64) -> Result<usize>;

    /// Clear all history
    pub async fn clear_all(&mut self) -> Result<()>;

    /// Get total visit count
    pub async fn count_visits(&self) -> Result<usize>;

    /// Get visit count for specific URL
    pub async fn count_visits_for_url(&self, url: &str) -> Result<usize>;

    /// Update visit duration (when tab closes)
    pub async fn update_visit_duration(&mut self, id: &VisitId, duration: i64) -> Result<()>;
}
```

## Database Schema

```sql
CREATE TABLE history_visits (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    visit_time INTEGER NOT NULL,
    visit_duration INTEGER,
    from_url TEXT,
    transition_type TEXT NOT NULL
);

CREATE INDEX idx_history_url ON history_visits(url);
CREATE INDEX idx_history_time ON history_visits(visit_time DESC);
CREATE INDEX idx_history_title ON history_visits(title);

-- View for aggregated page statistics
CREATE VIEW page_stats AS
SELECT
    url,
    MAX(title) as title,
    COUNT(*) as visit_count,
    MAX(visit_time) as last_visit,
    -- Frecency score: recent visits weighted more heavily
    SUM(
        CASE
            WHEN visit_time > strftime('%s', 'now') - 86400 THEN 100
            WHEN visit_time > strftime('%s', 'now') - 604800 THEN 70
            WHEN visit_time > strftime('%s', 'now') - 2592000 THEN 50
            WHEN visit_time > strftime('%s', 'now') - 7776000 THEN 30
            ELSE 10
        END
    ) as frecency_score
FROM history_visits
GROUP BY url;
```

## Implementation Requirements

### 1. HistoryStorage (src/storage.rs)

**Purpose**: SQLite database operations for history

**Required functionality**:
- Database initialization with schema and indices
- CRUD operations
- Search functionality (full-text or LIKE)
- Aggregation queries (most visited, frecency)
- Efficient deletion (bulk operations)

**Test requirements**:
- Test database creation
- Test all CRUD operations
- Test search functionality
- Test aggregation queries
- Test bulk deletion

### 2. HistoryManager (src/lib.rs)

**Purpose**: High-level history management API

**Required functionality**:
- All API methods listed above
- Visit ID generation (UUID v4)
- Input validation (URL format)
- Frecency score calculation
- Efficient querying for large datasets (100,000+ visits)

**Test requirements**:
- Test all public API methods
- Test error cases
- Test search with various filters
- Test frecency scoring
- Test clearing operations

### 3. Frecency Calculation (src/frecency.rs)

**Purpose**: Calculate frequency + recency scores for pages

**Required functionality**:
- Time-decay algorithm
- Weighted scoring based on visit recency
- Efficient batch calculation

**Test requirements**:
- Test scoring algorithm
- Test time decay
- Test edge cases (very old visits)

### 4. Validation (src/validation.rs)

**Purpose**: Input validation and sanitization

**Required functionality**:
- URL validation
- Title sanitization
- Timestamp validation

**Test requirements**:
- Test URL validation
- Test title sanitization
- Test timestamp validation

## Quality Standards

**Test Coverage**: ≥ 80% (target 90%)

**Test Categories**:
- Unit tests: All functions and methods
- Integration tests: Database operations
- Performance tests: Large dataset queries (100,000+ entries)

**Code Quality**:
- No `unwrap()` in production code
- All public APIs documented
- Clear error messages
- Safe concurrent access
- Efficient queries (proper indexing)

## File Structure

```
components/history/
├── src/
│   ├── lib.rs              # HistoryManager public API
│   ├── storage.rs          # SQLite storage implementation
│   ├── frecency.rs         # Frecency score calculation
│   ├── validation.rs       # Input validation
│   └── types.rs            # Data types
├── tests/
│   ├── unit/
│   │   ├── mod.rs
│   │   ├── history_tests.rs
│   │   ├── frecency_tests.rs
│   │   └── validation_tests.rs
│   └── integration/
│       ├── mod.rs
│       ├── storage_tests.rs
│       └── performance_tests.rs
├── Cargo.toml
├── CLAUDE.md               # This file
├── README.md
└── component.yaml
```

## Example Usage

```rust
use history::{HistoryManager, HistoryVisit, TransitionType, SearchQuery};

#[tokio::main]
async fn main() -> Result<()> {
    let mut manager = HistoryManager::new("history.db").await?;

    // Record a visit
    let visit = HistoryVisit {
        id: "".to_string(),  // Will be generated
        url: "https://rust-lang.org".to_string(),
        title: "Rust Programming Language".to_string(),
        visit_time: chrono::Utc::now().timestamp(),
        visit_duration: None,
        from_url: None,
        transition_type: TransitionType::Typed,
    };

    let visit_id = manager.record_visit(visit).await?;

    // Search history
    let results = manager.search(SearchQuery {
        text: Some("rust".to_string()),
        start_time: None,
        end_time: None,
        limit: 10,
    }).await?;

    // Get most visited pages
    let most_visited = manager.get_most_visited(10).await?;

    // Clear old history (older than 90 days)
    let ninety_days_ago = chrono::Utc::now().timestamp() - (90 * 86400);
    manager.clear_older_than(ninety_days_ago).await?;

    Ok(())
}
```

## Completion Checklist

Before marking this component complete:

- [ ] All API methods implemented
- [ ] All unit tests passing (≥80% coverage)
- [ ] All integration tests passing
- [ ] SQLite storage working correctly
- [ ] Visit recording working
- [ ] Search functionality working
- [ ] Frecency scoring working
- [ ] Most visited queries working
- [ ] Clear operations working
- [ ] Performance tested with large datasets
- [ ] No compiler warnings
- [ ] Documentation complete
- [ ] README.md updated with examples
- [ ] component.yaml complete

## Privacy Considerations

- **Incognito Mode**: History should not be recorded when in private browsing mode (controlled by caller)
- **Clear History**: Must support granular clearing (by time range, by URL)
- **Data Retention**: Consider implementing auto-expiry for old history
- **Sync**: DO NOT sync incognito history even if sync is enabled

## Performance Requirements

- Query recent visits (100 entries): < 10ms
- Search full-text query: < 100ms for 100,000 entries
- Record visit: < 5ms
- Clear old history: < 1s for 100,000 entries

## Questions or Issues

If you encounter specification ambiguities or technical blockers, document them in this section and continue with reasonable assumptions.
