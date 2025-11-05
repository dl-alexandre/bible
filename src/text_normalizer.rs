use regex::Regex;
use std::collections::HashSet;

static STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "been", "by", "for", "from",
    "has", "he", "in", "is", "it", "its", "of", "on", "that", "the", "to",
    "was", "were", "will", "with",
];

pub struct TextNormalizer {
    punctuation_pattern: Regex,
    verse_number_pattern: Regex,
    footnote_pattern: Regex,
    whitespace_pattern: Regex,
    stoplist: HashSet<String>,
}

impl TextNormalizer {
    pub fn new() -> Result<Self, regex::Error> {
        Ok(TextNormalizer {
            punctuation_pattern: Regex::new(r"[^\w\s]").unwrap_or_else(|_| {
                Regex::new(r".").unwrap()
            }),
            verse_number_pattern: Regex::new(r"\b\d+[a-z]?\b").unwrap_or_else(|_| {
                Regex::new(r".").unwrap()
            }),
            footnote_pattern: Regex::new(r"\[\d+\]").unwrap_or_else(|_| {
                Regex::new(r".").unwrap()
            }),
            whitespace_pattern: Regex::new(r"\s+").unwrap_or_else(|_| {
                Regex::new(r".").unwrap()
            }),
            stoplist: STOP_WORDS.iter().map(|s| s.to_string()).collect(),
        })
    }

    pub fn normalize(&self, text: &str) -> String {
        let mut normalized = text.to_lowercase();
        
        normalized = self.footnote_pattern.replace_all(&normalized, "").to_string();
        normalized = self.verse_number_pattern.replace_all(&normalized, "").to_string();
        normalized = self.punctuation_pattern.replace_all(&normalized, " ").to_string();
        normalized = self.whitespace_pattern.replace_all(&normalized, " ").to_string();
        
        normalized.trim().to_string()
    }

    pub fn normalize_tokens(&self, text: &str) -> HashSet<String> {
        let normalized = self.normalize(text);
        normalized
            .split_whitespace()
            .filter(|word| !self.stoplist.contains(*word))
            .filter(|word| !word.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let normalized = self.normalize(text);
        normalized
            .split_whitespace()
            .filter(|word| !self.stoplist.contains(*word))
            .filter(|word| !word.is_empty())
            .map(|s| s.to_string())
            .collect()
    }
}

impl Default for TextNormalizer {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        let normalizer = TextNormalizer::default();
        
        let text = "In the beginning God created [1] the heaven and the earth.";
        let normalized = normalizer.normalize(text);
        assert!(!normalized.contains("["));
        assert!(!normalized.contains("1"));
        assert_eq!(normalized.to_lowercase(), normalized);
    }

    #[test]
    fn test_stoplist_filtering() {
        let normalizer = TextNormalizer::default();
        
        let text = "In the beginning God created the heaven";
        let tokens = normalizer.normalize_tokens(text);
        assert!(!tokens.contains(&"the".to_string()));
        assert!(!tokens.contains(&"in".to_string()));
        assert!(tokens.contains(&"beginning".to_string()));
        assert!(tokens.contains(&"god".to_string()));
    }
}

