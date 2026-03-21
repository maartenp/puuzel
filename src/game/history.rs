use std::collections::VecDeque;

/// Tracks recently used word IDs to prevent puzzle repeats.
///
/// Maintains a sliding window of the last 200 word IDs used in puzzle generation.
/// The history resets on app restart (persistence is Phase 3).
pub struct WordHistory {
    recent: VecDeque<i64>,
}

const MAX_HISTORY: usize = 200;

impl WordHistory {
    /// Create a new empty word history.
    pub fn new() -> Self {
        WordHistory {
            recent: VecDeque::new(),
        }
    }

    /// Check if a word ID is in the recent history.
    pub fn contains(&self, word_id: i64) -> bool {
        self.recent.contains(&word_id)
    }

    /// Add a word ID to the history. If the history exceeds 200 entries, the oldest is evicted.
    pub fn add(&mut self, word_id: i64) {
        self.recent.push_back(word_id);
        if self.recent.len() > MAX_HISTORY {
            self.recent.pop_front();
        }
    }

    /// Iterate over all stored word IDs.
    pub fn recent_ids(&self) -> impl Iterator<Item = i64> + '_ {
        self.recent.iter().copied()
    }

    /// Add multiple word IDs to the history (e.g., after puzzle generation).
    pub fn add_all(&mut self, ids: impl IntoIterator<Item = i64>) {
        for id in ids {
            self.add(id);
        }
    }
}

impl Default for WordHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_contains() {
        let mut history = WordHistory::new();
        assert!(!history.contains(1));
        history.add(1);
        assert!(history.contains(1));
        assert!(!history.contains(2));
    }

    #[test]
    fn test_cap_at_200() {
        let mut history = WordHistory::new();

        // Add 201 items: ids 0..=200
        for i in 0i64..=200 {
            history.add(i);
        }

        // The first item (0) should have been evicted
        assert!(!history.contains(0), "First item should be evicted after 201 adds");

        // Item 1 should still be present (it's the oldest remaining)
        assert!(history.contains(1), "Item 1 should still be in history");

        // Item 200 should be present
        assert!(history.contains(200), "Last added item should be present");

        // Length should be exactly 200
        assert_eq!(
            history.recent.len(),
            200,
            "History should have exactly 200 entries"
        );
    }

    #[test]
    fn test_add_all() {
        let mut history = WordHistory::new();
        history.add_all(vec![10, 20, 30]);
        assert!(history.contains(10));
        assert!(history.contains(20));
        assert!(history.contains(30));
        assert!(!history.contains(40));
    }

    #[test]
    fn test_recent_ids_iter() {
        let mut history = WordHistory::new();
        history.add_all(vec![5, 10, 15]);

        let ids: Vec<i64> = history.recent_ids().collect();
        assert_eq!(ids, vec![5, 10, 15]);
    }

    #[test]
    fn test_eviction_order() {
        let mut history = WordHistory::new();
        // Add exactly 200 items
        for i in 0i64..200 {
            history.add(i);
        }
        // All 0..200 should be present
        assert!(history.contains(0));
        assert!(history.contains(199));

        // Add one more — 0 should be evicted
        history.add(200);
        assert!(!history.contains(0), "Item 0 should be evicted");
        assert!(history.contains(1), "Item 1 should remain");
        assert!(history.contains(200), "New item 200 should be present");
    }
}
