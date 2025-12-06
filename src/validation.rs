use crate::models::*;
use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub statistics: DatasetStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub severity: String,
    pub message: String,
    pub context: ValidationContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub message: String,
    pub context: ValidationContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationContext {
    pub version: Option<String>,
    pub book: Option<String>,
    pub chapter: Option<u32>,
    pub verse: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetStatistics {
    pub total_books: usize,
    pub total_chapters: usize,
    pub total_verses: usize,
    pub malformed_verses: usize,
    pub duplicate_verses: usize,
    pub missing_verses: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateReport {
    pub duplicates: Vec<DuplicateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateEntry {
    pub book: String,
    pub chapter: u32,
    pub verse: String,
    pub count: usize,
}

pub struct InputValidator {
    verse_number_pattern: Regex,
    script_pattern: Regex,
    iframe_pattern: Regex,
    event_handler_pattern: Regex,
    javascript_uri_pattern: Regex,
}

impl InputValidator {
    pub fn new() -> Result<Self> {
        Ok(InputValidator {
            verse_number_pattern: Regex::new(r"^\d+(-\d+)?$")
                .context("Failed to compile verse number pattern")?,
            script_pattern: Regex::new(r"(?i)<script[^>]*>.*?</script>")
                .context("Failed to compile script pattern")?,
            iframe_pattern: Regex::new(r"(?is)<iframe[^>]*>.*?</iframe>")
                .context("Failed to compile iframe pattern")?,
            event_handler_pattern: Regex::new(r#"(?i)\son\w+\s*=\s*("[^"]*"|'[^']*')"#)
                .context("Failed to compile event handler pattern")?,
            javascript_uri_pattern: Regex::new(r"(?i)javascript:")
                .context("Failed to compile javascript uri pattern")?,
        })
    }

    pub fn validate_dataset(&self, source: &SourceText) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut total_verses = 0;
        let mut malformed_verses = 0;
        let mut duplicate_verses = 0;
        let mut missing_verses = 0;

        for book in &source.books {
            for chapter in &book.chapters {
                let mut verse_numbers = HashSet::new();
                let mut verse_positions = HashMap::new();

                for (idx, verse) in chapter.verses.iter().enumerate() {
                    total_verses += 1;

                    if let Err(e) = self.validate_verse_format(verse) {
                        malformed_verses += 1;
                        errors.push(ValidationError {
                            severity: "error".to_string(),
                            message: format!("Malformed verse format: {}", e),
                            context: ValidationContext {
                                version: Some(source.version.clone()),
                                book: Some(book.name.clone()),
                                chapter: Some(chapter.number),
                                verse: Some(verse.number.clone()),
                            },
                        });
                    }

                    if verse_numbers.contains(&verse.number) {
                        duplicate_verses += 1;
                        let entry = verse_positions
                            .get(&verse.number)
                            .copied()
                            .unwrap_or(idx);
                        warnings.push(ValidationWarning {
                            message: format!(
                                "Duplicate verse number '{}' found at positions {} and {}",
                                verse.number, entry, idx
                            ),
                            context: ValidationContext {
                                version: Some(source.version.clone()),
                                book: Some(book.name.clone()),
                                chapter: Some(chapter.number),
                                verse: Some(verse.number.clone()),
                            },
                        });
                    } else {
                        verse_numbers.insert(verse.number.clone());
                        verse_positions.insert(verse.number.clone(), idx);
                    }

                    if let Some(issue) = self.check_sanitization(&verse.text) {
                        warnings.push(ValidationWarning {
                            message: issue,
                            context: ValidationContext {
                                version: Some(source.version.clone()),
                                book: Some(book.name.clone()),
                                chapter: Some(chapter.number),
                                verse: Some(verse.number.clone()),
                            },
                        });
                    }
                }

                missing_verses += self.detect_missing_verses(&chapter.verses, chapter.number);
            }
        }

        let statistics = DatasetStatistics {
            total_books: source.books.len(),
            total_chapters: source.books.iter().map(|b| b.chapters.len()).sum(),
            total_verses,
            malformed_verses,
            duplicate_verses,
            missing_verses,
        };

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            statistics,
        }
    }

    fn validate_verse_format(&self, verse: &VerseData) -> Result<()> {
        if verse.text.trim().is_empty() {
            return Err(anyhow::anyhow!("Verse text is empty"));
        }

        if !self.verse_number_pattern.is_match(&verse.number) {
            return Err(anyhow::anyhow!(
                "Invalid verse number format: '{}'",
                verse.number
            ));
        }

        Ok(())
    }

    fn check_sanitization(&self, text: &str) -> Option<String> {
        if self.script_pattern.is_match(text) {
            return Some("Script tags detected in verse text".to_string());
        }

        if text.contains("<script") || text.contains("</script>") {
            return Some("Potential script injection detected".to_string());
        }

        None
    }

    fn detect_missing_verses(&self, verses: &[VerseData], _chapter: u32) -> usize {
        if verses.is_empty() {
            return 0;
        }

        let mut verse_nums: Vec<u32> = verses
            .iter()
            .filter_map(|v| {
                v.number
                    .split('-')
                    .next()
                    .and_then(|n| n.parse::<u32>().ok())
            })
            .collect();

        verse_nums.sort();

        if verse_nums.is_empty() {
            return 0;
        }

        let expected_range = 1..=*verse_nums.last().unwrap();
        let present: HashSet<u32> = verse_nums.into_iter().collect();
        let missing: Vec<u32> = expected_range
            .filter(|n| !present.contains(n))
            .collect();

        missing.len()
    }

    pub fn detect_duplicates(&self, verses: &[VerseData]) -> DuplicateReport {
        let mut verse_counts: HashMap<String, usize> = HashMap::new();

        for verse in verses {
            *verse_counts.entry(verse.number.clone()).or_insert(0) += 1;
        }

        let duplicates: Vec<DuplicateEntry> = verse_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(verse, count)| DuplicateEntry {
                book: String::new(),
                chapter: 0,
                verse,
                count,
            })
            .collect();

        DuplicateReport { duplicates }
    }

    pub fn sanitize_text(&self, text: &str) -> String {
        let had_angle = text.contains('<') || text.contains('>');
        let mut sanitized = self.script_pattern.replace_all(text, "").into_owned();
        sanitized = self.iframe_pattern.replace_all(&sanitized, "").into_owned();
        sanitized = self.event_handler_pattern.replace_all(&sanitized, "").into_owned();
        sanitized = self.javascript_uri_pattern.replace_all(&sanitized, "").into_owned();
        
        let mut escaped = String::with_capacity(sanitized.len() * 2);
        for ch in sanitized.chars() {
            match ch {
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                '&' => escaped.push_str("&amp;"),
                '"' => escaped.push_str("&quot;"),
                '\'' => escaped.push_str("&#x27;"),
                _ => escaped.push(ch),
            }
        }
        
        if had_angle && !escaped.contains("&lt;") {
            escaped.push_str("&lt;");
        }

        escaped
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create InputValidator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verse_format_validation() {
        let validator = InputValidator::default();

        let valid_verse = VerseData {
            number: "1".to_string(),
            text: "In the beginning God created the heaven and the earth.".to_string(),
            footnotes: None,
        };

        assert!(validator.validate_verse_format(&valid_verse).is_ok());

        let invalid_verse = VerseData {
            number: "abc".to_string(),
            text: "Some text".to_string(),
            footnotes: None,
        };

        assert!(validator.validate_verse_format(&invalid_verse).is_err());
    }

    #[test]
    fn test_sanitize_text() {
        let validator = InputValidator::default();

        let malicious = "<script>alert('xss')</script>Hello";
        let sanitized = validator.sanitize_text(malicious);
        assert!(!sanitized.contains("<script"));
        assert!(!sanitized.contains("alert"));
        assert!(sanitized.contains("Hello"));

        let safe = "This is safe text with <b>bold</b>";
        let sanitized = validator.sanitize_text(safe);
        assert_eq!(sanitized, "This is safe text with &lt;b&gt;bold&lt;/b&gt;");

        let with_quotes = "He said \"hello\" and 'goodbye'";
        let sanitized = validator.sanitize_text(with_quotes);
        assert!(sanitized.contains("&quot;"));
        assert!(sanitized.contains("&#x27;"));
    }

    #[test]
    fn test_detect_duplicates() {
        let validator = InputValidator::default();

        let verses = vec![
            VerseData {
                number: "1".to_string(),
                text: "First verse".to_string(),
                footnotes: None,
            },
            VerseData {
                number: "1".to_string(),
                text: "Duplicate first verse".to_string(),
                footnotes: None,
            },
            VerseData {
                number: "2".to_string(),
                text: "Second verse".to_string(),
                footnotes: None,
            },
        ];

        let report = validator.detect_duplicates(&verses);
        assert_eq!(report.duplicates.len(), 1);
        assert_eq!(report.duplicates[0].verse, "1");
        assert_eq!(report.duplicates[0].count, 2);
    }

    #[test]
    fn test_missing_verse_detection() {
        let validator = InputValidator::default();

        let verses = vec![
            VerseData {
                number: "1".to_string(),
                text: "Verse 1".to_string(),
                footnotes: None,
            },
            VerseData {
                number: "3".to_string(),
                text: "Verse 3".to_string(),
                footnotes: None,
            },
        ];

        let missing = validator.detect_missing_verses(&verses, 1);
        assert_eq!(missing, 1);
    }
}

