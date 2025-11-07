use crate::models::*;
use crate::validation::InputValidator;
use anyhow::{Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum BibleFormat {
    KJV,
    ASV,
    WEB,
    OEB,
    BSB,
}

impl BibleFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "kjv" => Ok(BibleFormat::KJV),
            "asv" => Ok(BibleFormat::ASV),
            "web" => Ok(BibleFormat::WEB),
            "oeb" => Ok(BibleFormat::OEB),
            "bsb" => Ok(BibleFormat::BSB),
            _ => Err(anyhow::anyhow!("Unknown Bible format: {}", s)),
        }
    }
}

pub struct TextParser {
    validator: InputValidator,
    verse_pattern: Regex,
    chapter_pattern: Regex,
    book_name_pattern: Regex,
}

impl TextParser {
    pub fn new() -> Result<Self> {
        Ok(TextParser {
            validator: InputValidator::new()
                .context("Failed to create InputValidator for parser")?,
            verse_pattern: Regex::new(r"^\s*(\d+(-\d+)?)\s+(.+)$")
                .context("Failed to compile verse pattern")?,
            chapter_pattern: Regex::new(r"^\s*(?:Chapter\s+)?(\d+)\s*$")
                .context("Failed to compile chapter pattern")?,
            book_name_pattern: Regex::new(r"^[A-Z][a-zA-Z\s]+$")
                .context("Failed to compile book name pattern")?,
        })
    }

    pub fn parse_source_text(
        &self,
        text: &str,
        format: BibleFormat,
        version: &str,
    ) -> Result<SourceText> {
        let mut books = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        let mut current_book: Option<BookData> = None;
        let mut current_chapter: Option<ChapterData> = None;

        for (idx, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(book_name) = self.extract_book_name(line) {
                // Handle malformed files where a book name is immediately followed by another book name
                // Example: "1 Samuel\n1 Kings\nChapter 1" - the chapters belong to "1 Samuel", not "1 Kings"
                // If current book has no chapters yet, check if we just saw a book name recently
                // If so, this might be a duplicate/malformed entry - skip it
                let current_has_chapters = current_book.as_ref().map_or(false, |b| !b.chapters.is_empty());
                let current_has_chapter_in_progress = current_chapter.is_some();
                
                // Check if previous non-empty line was also a book name
                let prev_was_book = if idx > 0 {
                    lines.iter()
                        .take(idx)
                        .rev()
                        .find(|l| !l.trim().is_empty())
                        .and_then(|l| self.extract_book_name(l.trim()))
                        .is_some()
                } else {
                    false
                };
                
                if !current_has_chapters && !current_has_chapter_in_progress && prev_was_book {
                    // We just saw a book name, and now we see another book name
                    // The previous book has no chapters yet, so skip this duplicate/malformed entry
                    // The chapters that follow will be attributed to the previous book
                    continue;
                }
                
                if let Some(chapter) = current_chapter.take() {
                    if let Some(book) = &mut current_book {
                        book.chapters.push(chapter);
                    }
                }

                if let Some(book) = current_book.take() {
                    if !book.chapters.is_empty() {
                        books.push(book);
                    }
                }

                let abbreviation = self.get_book_abbreviation(&book_name);
                current_book = Some(BookData {
                    name: book_name,
                    abbreviation,
                    chapters: Vec::new(),
                });

                if let Some(chapter_num) = self.extract_chapter_number(line) {
                    current_chapter = Some(ChapterData {
                        number: chapter_num,
                        verses: Vec::new(),
                    });
                }
                continue;
            }

            if let Some(chapter_num) = self.extract_chapter_number(line) {
                if let Some(chapter) = current_chapter.take() {
                    if let Some(book) = &mut current_book {
                        book.chapters.push(chapter);
                    }
                }

                if current_book.is_none() {
                    current_book = Some(BookData {
                        name: "Unknown".to_string(),
                        abbreviation: "Unknown".to_string(),
                        chapters: Vec::new(),
                    });
                }

                current_chapter = Some(ChapterData {
                    number: chapter_num,
                    verses: Vec::new(),
                });
                continue;
            }

            if let Some(verse_data) = self.parse_verse_line(line, &format)? {
                if let Some(ref mut chapter) = current_chapter {
                    chapter.verses.push(verse_data);
                }
            } else if let Some(ref mut chapter) = current_chapter {
                if let Some(last_verse) = chapter.verses.last_mut() {
                    if !line.trim().is_empty() && !self.extract_chapter_number(line).is_some() && self.extract_book_name(line).is_none() {
                        last_verse.text.push(' ');
                        last_verse.text.push_str(line.trim());
                    }
                }
            }
        }

        if let Some(chapter) = current_chapter.take() {
            if current_book.is_none() {
                current_book = Some(BookData {
                    name: "Unknown".to_string(),
                    abbreviation: "Unknown".to_string(),
                    chapters: Vec::new(),
                });
            }
            if let Some(book) = &mut current_book {
                book.chapters.push(chapter);
            }
        }

        if let Some(book) = current_book {
            if !book.chapters.is_empty() {
                books.push(book);
            }
        }

        Ok(SourceText {
            version: version.to_string(),
            books,
            metadata: SourceMetadata {
                description: None,
                language: Some("en".to_string()),
            },
        })
    }

    fn extract_chapter_number(&self, line: &str) -> Option<u32> {
        self.chapter_pattern
            .captures(line)
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse::<u32>().ok())
    }

    pub(crate) fn extract_book_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            return None;
        }

        let book_names = [
            "Genesis", "Exodus", "Leviticus", "Numbers", "Deuteronomy",
            "Joshua", "Judges", "Ruth", "1 Samuel", "2 Samuel", "1 Kings", "2 Kings",
            "1 Chronicles", "2 Chronicles", "Ezra", "Nehemiah", "Esther", "Job",
            "Psalm", "Psalms", "Proverbs", "Ecclesiastes", "Song of Solomon", "Song of Songs",
            "Isaiah", "Jeremiah", "Lamentations", "Ezekiel", "Daniel", "Hosea", "Joel",
            "Amos", "Obadiah", "Jonah", "Micah", "Nahum", "Habakkuk", "Zephaniah",
            "Haggai", "Zechariah", "Malachi",
            "Matthew", "Mark", "Luke", "John", "Acts", "Romans", "1 Corinthians",
            "2 Corinthians", "Galatians", "Ephesians", "Philippians", "Colossians",
            "1 Thessalonians", "2 Thessalonians", "1 Timothy", "2 Timothy", "Titus",
            "Philemon", "Hebrews", "James", "1 Peter", "2 Peter", "1 John", "2 John",
            "3 John", "Jude", "Revelation",
        ];

        for book in &book_names {
            if trimmed == *book {
                return Some(book.to_string());
            }
        }

        if self.book_name_pattern.is_match(trimmed) && trimmed.len() > 2 && !trimmed.contains("Chapter") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() <= 4 && parts.iter().all(|p| p.chars().next().map_or(false, |c| c.is_uppercase() || c.is_ascii_digit())) {
                return Some(trimmed.to_string());
            }
        }

        None
    }

    pub(crate) fn get_book_abbreviation(&self, book_name: &str) -> String {
        let abbreviations: std::collections::HashMap<&str, &str> = [
            ("Genesis", "Gen"), ("Exodus", "Exod"), ("Leviticus", "Lev"),
            ("Numbers", "Num"), ("Deuteronomy", "Deut"), ("Joshua", "Josh"),
            ("Judges", "Judg"), ("Ruth", "Ruth"), ("1 Samuel", "1Sam"),
            ("2 Samuel", "2Sam"), ("1 Kings", "1Kgs"), ("2 Kings", "2Kgs"),
            ("1 Chronicles", "1Chr"), ("2 Chronicles", "2Chr"), ("Ezra", "Ezra"),
            ("Nehemiah", "Neh"), ("Esther", "Esth"), ("Job", "Job"),
            ("Psalm", "Ps"), ("Psalms", "Ps"), ("Proverbs", "Prov"),
            ("Ecclesiastes", "Eccl"), ("Song of Solomon", "Song"), ("Song of Songs", "Song"),
            ("Isaiah", "Isa"), ("Jeremiah", "Jer"), ("Lamentations", "Lam"),
            ("Ezekiel", "Ezek"), ("Daniel", "Dan"), ("Hosea", "Hos"),
            ("Joel", "Joel"), ("Amos", "Amos"), ("Obadiah", "Obad"),
            ("Jonah", "Jonah"), ("Micah", "Mic"), ("Nahum", "Nah"),
            ("Habakkuk", "Hab"), ("Zephaniah", "Zeph"), ("Haggai", "Hag"),
            ("Zechariah", "Zech"), ("Malachi", "Mal"),
            ("Matthew", "Matt"), ("Mark", "Mark"), ("Luke", "Luke"),
            ("John", "John"), ("Acts", "Acts"), ("Romans", "Rom"),
            ("1 Corinthians", "1Cor"), ("2 Corinthians", "2Cor"), ("Galatians", "Gal"),
            ("Ephesians", "Eph"), ("Philippians", "Phil"), ("Colossians", "Col"),
            ("1 Thessalonians", "1Thess"), ("2 Thessalonians", "2Thess"),
            ("1 Timothy", "1Tim"), ("2 Timothy", "2Tim"), ("Titus", "Titus"),
            ("Philemon", "Phlm"), ("Hebrews", "Heb"), ("James", "Jas"),
            ("1 Peter", "1Pet"), ("2 Peter", "2Pet"), ("1 John", "1John"),
            ("2 John", "2John"), ("3 John", "3John"), ("Jude", "Jude"),
            ("Revelation", "Rev"),
        ].iter().cloned().collect();

        abbreviations.get(book_name).map(|s| s.to_string()).unwrap_or_else(|| {
            book_name.split_whitespace().take(3).collect::<Vec<&str>>().join(" ")
        })
    }

    fn parse_verse_line(&self, line: &str, format: &BibleFormat) -> Result<Option<VerseData>> {
        let captures = match self.verse_pattern.captures(line) {
            Some(caps) => caps,
            None => return Ok(None),
        };

        let verse_number = captures
            .get(1)
            .context("Verse number not found in pattern")?
            .as_str()
            .to_string();

        let verse_text = captures
            .get(3)
            .context("Verse text not found in pattern")?
            .as_str()
            .to_string();

        let sanitized_text = self.validator.sanitize_text(&verse_text);

        Ok(Some(VerseData {
            number: verse_number,
            text: sanitized_text,
            footnotes: self.extract_footnotes(&verse_text, &format)?,
        }))
    }

    fn extract_footnotes(&self, text: &str, _format: &BibleFormat) -> Result<Option<Vec<String>>> {
        let footnote_pattern = Regex::new(r"\[(\d+)\]")
            .context("Failed to compile footnote pattern")?;

        let footnotes: Vec<String> = footnote_pattern
            .captures_iter(text)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        if footnotes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(footnotes))
        }
    }

    pub fn parse_chapter(
        &self,
        chapter_text: &str,
        book: &str,
        chapter: u32,
        version: &str,
    ) -> Result<Chapter> {
        let format = BibleFormat::from_str(version)?;
        let verses = self.extract_verses(chapter_text, &format, chapter)?;

        let mut verse_map = HashMap::new();
        for verse_data in verses {
            let verse = self.create_verse(&verse_data, book, chapter, version)?;
            verse_map.insert(verse.number.clone(), verse);
        }

        let verse_count = verse_map.len() as u32;

        Ok(Chapter {
            book: book.to_string(),
            chapter,
            verses: verse_map,
            metadata: ChapterMetadata {
                verse_count,
                last_updated: Some(
                    chrono::Utc::now()
                        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                ),
            },
        })
    }

    fn extract_verses(&self, text: &str, format: &BibleFormat, chapter: u32) -> Result<Vec<VerseData>> {
        let mut verses = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        for line in lines {
            if let Some(verse) = self.parse_verse_line(line, format)? {
                let processed_verse = self.split_embedded_verses(verse, format, chapter)?;
                verses.extend(processed_verse);
            } else if let Some(last_verse) = verses.last_mut() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    last_verse.text.push(' ');
                    last_verse.text.push_str(trimmed);
                }
            }
        }

        Ok(verses)
    }

    fn split_embedded_verses(&self, verse: VerseData, format: &BibleFormat, chapter: u32) -> Result<Vec<VerseData>> {
        match format {
            BibleFormat::BSB => {
                Ok(vec![verse])
            },
            _ => {
                let embedded_verse_pattern = Regex::new(r"\s+(\d+):(\d+)(?:\s+|$)")
                    .context("Failed to compile embedded verse pattern")?;
                
                let mut result = Vec::new();
                let mut current_text = verse.text.clone();
                let mut current_number = verse.number.clone();
                
                loop {
                    if let Some(captures) = embedded_verse_pattern.captures(&current_text) {
                        let chapter_ref: u32 = captures.get(1).unwrap().as_str().parse().unwrap_or(0);
                        let verse_ref = captures.get(2).unwrap().as_str().to_string();
                        
                        if chapter_ref == chapter {
                            let full_match = captures.get(0).unwrap();
                            let split_pos = full_match.start();
                            let text_before = current_text[..split_pos].trim().to_string();
                            let text_after = current_text[full_match.end()..].trim().to_string();
                            
                            if !text_before.is_empty() {
                                result.push(VerseData {
                                    number: current_number.clone(),
                                    text: text_before,
                                    footnotes: self.extract_footnotes(&current_text[..split_pos], format)?,
                                });
                            }
                            
                            if !text_after.is_empty() {
                                current_number = verse_ref;
                                current_text = text_after;
                                continue;
                            } else {
                                break;
                            }
                        } else {
                            let text = current_text.clone();
                            result.push(VerseData {
                                number: current_number,
                                text: text.clone(),
                                footnotes: self.extract_footnotes(&text, format)?,
                            });
                            break;
                        }
                    } else {
                        if !current_text.trim().is_empty() {
                            result.push(VerseData {
                                number: current_number,
                                text: current_text.trim().to_string(),
                                footnotes: self.extract_footnotes(&current_text, format)?,
                            });
                        }
                        break;
                    }
                }
                
                if result.is_empty() {
                    Ok(vec![verse])
                } else {
                    Ok(result)
                }
            }
        }
    }

    fn create_verse(
        &self,
        verse_data: &VerseData,
        book: &str,
        chapter: u32,
        version: &str,
    ) -> Result<Verse> {
        let id = self.generate_deterministic_id(verse_data, book, chapter, version);
        let anchor = format!("#v{}", verse_data.number);
        let canonical_ref = format!("{}.{}.{}", book, chapter, verse_data.number);

        Ok(Verse {
            id,
            number: verse_data.number.clone(),
            text: verse_data.text.clone(),
            anchor,
            canonical_ref,
        })
    }

    pub fn generate_deterministic_id(
        &self,
        verse: &VerseData,
        book: &str,
        chapter: u32,
        version: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(version.as_bytes());
        hasher.update(book.as_bytes());
        hasher.update(chapter.to_string().as_bytes());
        hasher.update(verse.number.as_bytes());
        hasher.update(verse.text.as_bytes());

        let hash = hasher.finalize();
        format!("{:x}", hash)
    }
}

impl Default for TextParser {
    fn default() -> Self {
        Self::new().expect("Failed to create TextParser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_verse_line() {
        let parser = TextParser::default();

        let line = "1 In the beginning God created the heaven and the earth.";
        let verse = parser
            .parse_verse_line(line, &BibleFormat::KJV)
            .unwrap()
            .unwrap();

        assert_eq!(verse.number, "1");
        assert!(verse.text.contains("beginning"));
    }

    #[test]
    fn test_generate_deterministic_id() {
        let parser = TextParser::default();

        let verse1 = VerseData {
            number: "1".to_string(),
            text: "Test verse".to_string(),
            footnotes: None,
        };

        let id1 = parser.generate_deterministic_id(&verse1, "Genesis", 1, "kjv");
        let id2 = parser.generate_deterministic_id(&verse1, "Genesis", 1, "kjv");

        assert_eq!(id1, id2);

        let id3 = parser.generate_deterministic_id(&verse1, "Genesis", 2, "kjv");
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_extract_chapter_number() {
        let parser = TextParser::default();

        assert_eq!(parser.extract_chapter_number("Chapter 1"), Some(1));
        assert_eq!(parser.extract_chapter_number("1"), Some(1));
        assert_eq!(parser.extract_chapter_number("  42  "), Some(42));
        assert_eq!(parser.extract_chapter_number("Not a chapter"), None);
    }

    #[test]
    fn test_parse_chapter() {
        let parser = TextParser::default();

        let chapter_text = "1 In the beginning God created the heaven and the earth.\n2 And the earth was without form, and void;";
        let chapter = parser
            .parse_chapter(chapter_text, "Genesis", 1, "kjv")
            .unwrap();

        assert_eq!(chapter.book, "Genesis");
        assert_eq!(chapter.chapter, 1);
        assert_eq!(chapter.verses.len(), 2);
        assert!(chapter.verses.contains_key("1"));
        assert!(chapter.verses.contains_key("2"));
    }

    #[test]
    fn test_extract_book_name() {
        let parser = TextParser::default();

        assert_eq!(parser.extract_book_name("Genesis"), Some("Genesis".to_string()));
        assert_eq!(parser.extract_book_name("Genesis 1"), Some("Genesis".to_string()));
        assert_eq!(parser.extract_book_name("Chapter 1"), None);
        assert_eq!(parser.extract_book_name("1 In the beginning"), None);
        assert_eq!(parser.extract_book_name("Exodus"), Some("Exodus".to_string()));
    }

    #[test]
    fn test_parse_source_text_with_book_name() {
        let parser = TextParser::default();

        let text = "Genesis\nChapter 1\n1 In the beginning God created the heaven and the earth.\n2 And the earth was without form, and void;";
        let source = parser
            .parse_source_text(text, BibleFormat::KJV, "kjv")
            .unwrap();

        assert_eq!(source.books.len(), 1);
        assert_eq!(source.books[0].name, "Genesis");
        assert_eq!(source.books[0].abbreviation, "Gen");
        assert_eq!(source.books[0].chapters.len(), 1);
        assert_eq!(source.books[0].chapters[0].number, 1);
        assert_eq!(source.books[0].chapters[0].verses.len(), 2);
    }

    #[test]
    fn test_get_book_abbreviation() {
        let parser = TextParser::default();

        assert_eq!(parser.get_book_abbreviation("Genesis"), "Gen");
        assert_eq!(parser.get_book_abbreviation("1 Corinthians"), "1Cor");
        assert_eq!(parser.get_book_abbreviation("Revelation"), "Rev");
    }
}
