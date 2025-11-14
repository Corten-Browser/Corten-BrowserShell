/// Calculate score for a single visit based on recency
/// Returns a score based on how recent the visit was:
/// - Last 24 hours: 100
/// - Last week: 70
/// - Last month: 50
/// - Last 3 months: 30
/// - Older: 10
pub fn calculate_visit_score(visit_time: i64, current_time: i64) -> f64 {
    let age = current_time - visit_time;

    // Define time periods in seconds
    const DAY: i64 = 86400;
    const WEEK: i64 = 7 * DAY;
    const MONTH: i64 = 30 * DAY;
    const THREE_MONTHS: i64 = 90 * DAY;

    if age <= DAY {
        100.0
    } else if age <= WEEK {
        70.0
    } else if age <= MONTH {
        50.0
    } else if age <= THREE_MONTHS {
        30.0
    } else {
        10.0
    }
}

/// Calculate frecency score for a list of visit timestamps
/// Frecency = sum of all visit scores (frequency + recency)
pub fn calculate_frecency_score(visit_times: &[i64], current_time: i64) -> f64 {
    visit_times
        .iter()
        .map(|&visit_time| calculate_visit_score(visit_time, current_time))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visit_score_boundaries() {
        let now = 1000000i64;

        // Exactly at boundaries
        assert_eq!(calculate_visit_score(now, now), 100.0);
        assert_eq!(calculate_visit_score(now - 86400, now), 100.0);
        assert_eq!(calculate_visit_score(now - 86401, now), 70.0);
    }

    #[test]
    fn test_frecency_with_duplicates() {
        let now = chrono::Utc::now().timestamp();
        let visits = vec![now, now, now]; // 3 visits at same time
        let score = calculate_frecency_score(&visits, now);
        assert_eq!(score, 300.0);
    }
}
