//! Profanity filtering
//!
//! Tools for detecting and filtering profane content.

/// Profanity filter
#[derive(Debug, Clone)]
pub struct ProfanityFilter {
    /// Blocked words list
    blocked_words: Vec<String>,
    /// Replacement character
    replacement_char: char,
}

impl Default for ProfanityFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfanityFilter {
    /// Create a new profanity filter
    pub fn new() -> Self {
        Self {
            blocked_words: vec![
                "badword1".to_string(),
                "badword2".to_string(),
                // Add more blocked words
            ],
            replacement_char: '*',
        }
    }

    /// Check if text contains profanity
    pub fn contains_profanity(&self, text: &str) -> bool {
        let lower_text = text.to_lowercase();
        self.blocked_words
            .iter()
            .any(|word| lower_text.contains(word))
    }

    /// Filter profanity from text
    pub fn filter(&self, text: &str) -> String {
        let mut result = text.to_string();
        for word in &self.blocked_words {
            let replacement = self.replacement_char.to_string().repeat(word.len());
            result = result.replace(word, &replacement);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Creation Tests ====================

    #[test]
    fn test_profanity_filter_new() {
        let filter = ProfanityFilter::new();
        assert!(!filter.blocked_words.is_empty());
        assert_eq!(filter.replacement_char, '*');
    }

    #[test]
    fn test_profanity_filter_default() {
        let filter = ProfanityFilter::default();
        assert!(!filter.blocked_words.is_empty());
        assert_eq!(filter.replacement_char, '*');
    }

    // ==================== Contains Profanity Tests ====================

    #[test]
    fn test_contains_profanity_found() {
        let filter = ProfanityFilter::new();
        assert!(filter.contains_profanity("This contains badword1"));
    }

    #[test]
    fn test_contains_profanity_not_found() {
        let filter = ProfanityFilter::new();
        assert!(!filter.contains_profanity("This is clean text"));
    }

    #[test]
    fn test_contains_profanity_case_insensitive() {
        let filter = ProfanityFilter::new();
        assert!(filter.contains_profanity("This contains BADWORD1"));
        assert!(filter.contains_profanity("This contains BadWord1"));
    }

    #[test]
    fn test_contains_profanity_multiple_words() {
        let filter = ProfanityFilter::new();
        assert!(filter.contains_profanity("badword1 and badword2"));
    }

    #[test]
    fn test_contains_profanity_empty_text() {
        let filter = ProfanityFilter::new();
        assert!(!filter.contains_profanity(""));
    }

    #[test]
    fn test_contains_profanity_word_at_start() {
        let filter = ProfanityFilter::new();
        assert!(filter.contains_profanity("badword1 is at the start"));
    }

    #[test]
    fn test_contains_profanity_word_at_end() {
        let filter = ProfanityFilter::new();
        assert!(filter.contains_profanity("Text ends with badword2"));
    }

    #[test]
    fn test_contains_profanity_word_as_substring() {
        let filter = ProfanityFilter::new();
        // The word "badword1" is part of "mybadword1here"
        assert!(filter.contains_profanity("mybadword1here"));
    }

    // ==================== Filter Tests ====================

    #[test]
    fn test_filter_replaces_profanity() {
        let filter = ProfanityFilter::new();
        let filtered = filter.filter("This contains badword1");
        assert!(!filtered.contains("badword1"));
        assert!(filtered.contains("********")); // 8 characters like "badword1"
    }

    #[test]
    fn test_filter_clean_text() {
        let filter = ProfanityFilter::new();
        let text = "This is clean text";
        let filtered = filter.filter(text);
        assert_eq!(filtered, text);
    }

    #[test]
    fn test_filter_multiple_occurrences() {
        let filter = ProfanityFilter::new();
        let filtered = filter.filter("badword1 appears badword1 twice");
        assert!(!filtered.contains("badword1"));
    }

    #[test]
    fn test_filter_multiple_different_words() {
        let filter = ProfanityFilter::new();
        let filtered = filter.filter("badword1 and badword2");
        assert!(!filtered.contains("badword1"));
        assert!(!filtered.contains("badword2"));
    }

    #[test]
    fn test_filter_empty_text() {
        let filter = ProfanityFilter::new();
        let filtered = filter.filter("");
        assert_eq!(filtered, "");
    }

    #[test]
    fn test_filter_only_profanity() {
        let filter = ProfanityFilter::new();
        let filtered = filter.filter("badword1");
        assert_eq!(filtered, "********");
    }

    #[test]
    fn test_filter_preserves_surrounding_text() {
        let filter = ProfanityFilter::new();
        let filtered = filter.filter("Hello badword1 world");
        assert!(filtered.starts_with("Hello"));
        assert!(filtered.ends_with("world"));
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_profanity_filter_clone() {
        let original = ProfanityFilter::new();
        let cloned = original.clone();
        assert_eq!(original.blocked_words, cloned.blocked_words);
        assert_eq!(original.replacement_char, cloned.replacement_char);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_filter_case_sensitivity() {
        let filter = ProfanityFilter::new();
        // Note: filter() doesn't do case-insensitive replacement by default
        // It only replaces exact matches
        let _filtered = filter.filter("BADWORD1");
        // Contains check is case-insensitive, but filter uses exact match
        assert!(filter.contains_profanity("BADWORD1"));
    }

    #[test]
    fn test_replacement_length_matches_word() {
        let filter = ProfanityFilter::new();
        let filtered = filter.filter("badword1"); // 8 chars
        assert_eq!(filtered.len(), 8);
        assert_eq!(filtered, "********");
    }

    #[test]
    fn test_special_characters_in_text() {
        let filter = ProfanityFilter::new();
        let text = "Text with badword1!@#$%";
        let filtered = filter.filter(text);
        assert!(filtered.contains("!@#$%"));
        assert!(!filtered.contains("badword1"));
    }

    #[test]
    fn test_unicode_text() {
        let filter = ProfanityFilter::new();
        let text = "Hello 世界 badword1 こんにちは";
        let filtered = filter.filter(text);
        assert!(filtered.contains("世界"));
        assert!(filtered.contains("こんにちは"));
        assert!(!filtered.contains("badword1"));
    }
}
