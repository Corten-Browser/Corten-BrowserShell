use history::frecency::{calculate_visit_score, calculate_frecency_score};

#[test]
fn test_calculate_visit_score_today() {
    let now = chrono::Utc::now().timestamp();
    let score = calculate_visit_score(now, now);
    assert_eq!(score, 100.0);
}

#[test]
fn test_calculate_visit_score_yesterday() {
    let now = chrono::Utc::now().timestamp();
    let yesterday = now - 86400; // 1 day ago
    let score = calculate_visit_score(yesterday, now);
    assert_eq!(score, 100.0);
}

#[test]
fn test_calculate_visit_score_one_week_ago() {
    let now = chrono::Utc::now().timestamp();
    let one_week_ago = now - (7 * 86400);
    let score = calculate_visit_score(one_week_ago, now);
    assert_eq!(score, 70.0);
}

#[test]
fn test_calculate_visit_score_one_month_ago() {
    let now = chrono::Utc::now().timestamp();
    let one_month_ago = now - (30 * 86400);
    let score = calculate_visit_score(one_month_ago, now);
    assert_eq!(score, 50.0);
}

#[test]
fn test_calculate_visit_score_three_months_ago() {
    let now = chrono::Utc::now().timestamp();
    let three_months_ago = now - (90 * 86400);
    let score = calculate_visit_score(three_months_ago, now);
    assert_eq!(score, 30.0);
}

#[test]
fn test_calculate_visit_score_very_old() {
    let now = chrono::Utc::now().timestamp();
    let very_old = now - (365 * 86400);
    let score = calculate_visit_score(very_old, now);
    assert_eq!(score, 10.0);
}

#[test]
fn test_calculate_frecency_score_single_recent_visit() {
    let now = chrono::Utc::now().timestamp();
    let visits = vec![now];
    let score = calculate_frecency_score(&visits, now);
    assert_eq!(score, 100.0);
}

#[test]
fn test_calculate_frecency_score_multiple_recent_visits() {
    let now = chrono::Utc::now().timestamp();
    let visits = vec![now, now - 3600, now - 7200]; // 3 visits within last day
    let score = calculate_frecency_score(&visits, now);
    assert_eq!(score, 300.0); // 100 * 3
}

#[test]
fn test_calculate_frecency_score_mixed_visits() {
    let now = chrono::Utc::now().timestamp();
    let visits = vec![
        now,                      // 100
        now - (7 * 86400),        // 70
        now - (30 * 86400),       // 50
        now - (90 * 86400),       // 30
        now - (365 * 86400),      // 10
    ];
    let score = calculate_frecency_score(&visits, now);
    assert_eq!(score, 260.0); // 100 + 70 + 50 + 30 + 10
}

#[test]
fn test_calculate_frecency_score_empty() {
    let now = chrono::Utc::now().timestamp();
    let visits = vec![];
    let score = calculate_frecency_score(&visits, now);
    assert_eq!(score, 0.0);
}
