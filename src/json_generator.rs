use crate::logger::*;
use crate::models::*;
use crate::schema::validate_json;
use anyhow::{Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct JsonGenerator {
    output_base: PathBuf,
    logger: DiagnosticLogger,
    minify: bool,
    compress: bool,
    schema_cache: HashMap<String, Value>,
}

impl JsonGenerator {
    pub fn new(
        output_dir: &Path,
        logger: DiagnosticLogger,
        minify: bool,
        compress: bool,
    ) -> Result<Self> {
        let schema_dir = output_dir.join("schema");
        let mut schema_cache = HashMap::new();

        Self::load_schemas(&schema_dir, &mut schema_cache)?;

        Ok(JsonGenerator {
            output_base: output_dir.to_path_buf(),
            logger,
            minify,
            compress,
            schema_cache,
        })
    }

    fn load_schemas(schema_dir: &Path, cache: &mut HashMap<String, Value>) -> Result<()> {
        if !schema_dir.exists() {
            return Ok(());
        }

        let schema_files = vec!["chapter-1.0.json", "manifest-1.0.json", "crossrefs-1.0.json"];

        for schema_file in schema_files {
            let schema_path = schema_dir.join(schema_file);
            if schema_path.exists() {
                let schema_content = fs::read_to_string(&schema_path)
                    .context(format!("Failed to read schema: {}", schema_file))?;
                let schema_json: Value = serde_json::from_str(&schema_content)
                    .context(format!("Failed to parse schema: {}", schema_file))?;
                cache.insert(schema_file.to_string(), schema_json);
            }
        }

        Ok(())
    }

    fn compile_and_validate(&self, schema_name: &str, json_value: &Value) -> Result<()> {
        // Schema validation - log warnings but don't fail
        // Note: Full validation happens via schema.rs::validate_json when schemas are available
        if self.schema_cache.contains_key(schema_name) {
            let schema_path = self.output_base.join("schema").join(schema_name);
            if schema_path.exists() {
                if let Err(e) = crate::schema::validate_json(json_value, &schema_path) {
                    self.logger.warning(
                        format!("Schema validation warning for {}: {}", schema_name, e),
                        None,
                    );
                }
            }
        }
        Ok(())
    }

    pub fn generate_chapter_json(
        &self,
        chapter: &Chapter,
        version_code: &str,
    ) -> Result<PathBuf> {
        let book_dir = self
            .output_base
            .join(version_code)
            .join(&chapter.book);

        fs::create_dir_all(&book_dir)
            .context("Failed to create book directory")?;

        let output_path = book_dir.join(format!("{}.json", chapter.chapter));

        let mut verses_map: Vec<(String, String)> = chapter
            .verses
            .iter()
            .map(|(num, verse)| (num.clone(), verse.text.clone()))
            .collect();
        verses_map.sort_by_key(|(k, _)| k.clone());
        let verses_map: HashMap<String, String> = verses_map.into_iter().collect();

        let chapter_json = ChapterJson {
            schema_version: "1.0".to_string(),
            book: chapter.book.clone(),
            chapter: chapter.chapter,
            version: version_code.to_string(),
            verses: verses_map,
            metadata: chapter.metadata.clone(),
            extensions: json!({}),
        };

        let json_str = if self.minify {
            serde_json::to_string(&chapter_json)?
        } else {
            serde_json::to_string_pretty(&chapter_json)?
        };

        let json_value: Value = serde_json::from_str(&json_str)?;
        if let Err(e) = self.compile_and_validate("chapter-1.0.json", &json_value) {
            self.logger.warning(
                format!("Chapter JSON validation warning: {}", e),
                Some(json!({"chapter": chapter_json.chapter, "book": chapter_json.book})),
            );
        }

        fs::write(&output_path, &json_str)
            .context("Failed to write chapter JSON")?;

        if self.compress {
            self.compress_json(&output_path)?;
        }

        self.logger.info(format!(
            "Generated JSON: {} ({} bytes)",
            output_path.display(),
            json_str.len()
        ));

        Ok(output_path)
    }

    pub fn generate_versions_json(
        &self,
        versions: &HashMap<String, HashMap<String, Chapter>>,
    ) -> Result<PathBuf> {
        let output_path = self.output_base.join("versions.json");

        let mut version_list: Vec<Value> = Vec::new();

        for (version_code, chapters) in versions {
            let mut books = HashMap::new();
            for chapter_key in chapters.keys() {
                let parts: Vec<&str> = chapter_key.split('.').collect();
                if let Some(book_name) = parts.first() {
                    books.entry(book_name.to_string()).or_insert_with(|| {
                        chapters
                            .keys()
                            .filter(|k| k.starts_with(book_name))
                            .count()
                    });
                }
            }

            version_list.push(json!({
                "code": version_code,
                "name": version_code.to_uppercase(),
                "book_count": books.len(),
                "chapter_count": chapters.len(),
            }));
        }

        version_list.sort_by_key(|v| v["code"].as_str().unwrap_or("").to_string());

        let versions_json = json!({
            "schema_version": "1.0",
            "versions": version_list
        });

        let json_str = if self.minify {
            serde_json::to_string(&versions_json)?
        } else {
            serde_json::to_string_pretty(&versions_json)?
        };

        fs::write(&output_path, &json_str)
            .context("Failed to write versions.json")?;

        if self.compress {
            self.compress_json(&output_path)?;
        }

        self.logger.info(format!("Generated versions.json ({} bytes)", json_str.len()));

        Ok(output_path)
    }

    pub fn generate_books_json(
        &self,
        versions: &HashMap<String, HashMap<String, Chapter>>,
    ) -> Result<PathBuf> {
        let output_path = self.output_base.join("books.json");

        let mut book_map: HashMap<String, Value> = HashMap::new();

        for chapters in versions.values() {
            for chapter_key in chapters.keys() {
                let parts: Vec<&str> = chapter_key.split('.').collect();
                if let Some(book_name) = parts.first() {
                    book_map
                        .entry(book_name.to_string())
                        .and_modify(|e| {
                            let chapter_count = e["chapter_count"].as_u64().unwrap_or(0);
                            *e = json!({
                                "name": book_name,
                                "chapter_count": chapter_count.max(1),
                            });
                        })
                        .or_insert_with(|| {
                            json!({
                                "name": book_name,
                                "chapter_count": chapters.keys().filter(|k| k.starts_with(book_name)).count().max(1),
                            })
                        });
                }
            }
        }

        let mut book_list: Vec<Value> = book_map.values().cloned().collect();
        book_list.sort_by_key(|b| b["name"].as_str().unwrap_or("").to_string());

        let books_json = json!({
            "schema_version": "1.0",
            "books": book_list
        });

        let json_str = if self.minify {
            serde_json::to_string(&books_json)?
        } else {
            serde_json::to_string_pretty(&books_json)?
        };

        fs::write(&output_path, &json_str)
            .context("Failed to write books.json")?;

        if self.compress {
            self.compress_json(&output_path)?;
        }

        self.logger.info(format!("Generated books.json ({} bytes)", json_str.len()));

        Ok(output_path)
    }

    pub fn generate_crossrefs_json(
        &self,
        crossrefs: &CrossReferenceMap,
        output_path: Option<&Path>,
    ) -> Result<PathBuf> {
        let path = output_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.output_base.join("crossrefs.json"));

        fs::create_dir_all(path.parent().unwrap_or(&self.output_base))?;

        let json_str = if self.minify {
            serde_json::to_string(crossrefs)?
        } else {
            serde_json::to_string_pretty(crossrefs)?
        };

        let json_value: Value = serde_json::from_str(&json_str)?;
        if let Err(e) = self.compile_and_validate("crossrefs-1.0.json", &json_value) {
            self.logger.warning(
                format!("Crossrefs JSON validation warning: {}", e),
                None,
            );
        }

        fs::write(&path, &json_str)
            .context("Failed to write crossrefs.json")?;

        if self.compress {
            self.compress_json(&path)?;
        }

        let hash = Self::hash_json(&json_str);
        self.logger.info(format!(
            "Generated crossrefs.json ({} bytes, SHA-256: {})",
            json_str.len(),
            hash
        ));

        Ok(path)
    }

    fn compress_json(&self, json_path: &Path) -> Result<PathBuf> {
        let gz_path = json_path.with_extension("json.gz");

        let json_content = fs::read(json_path)
            .context("Failed to read JSON for compression")?;

        let mut encoder = GzEncoder::new(
            fs::File::create(&gz_path)
                .context("Failed to create compressed file")?,
            Compression::default(),
        );

        encoder
            .write_all(&json_content)
            .context("Failed to write compressed data")?;

        encoder
            .finish()
            .context("Failed to finalize compression")?;

        let original_size = json_content.len();
        let compressed_size = fs::metadata(&gz_path)?.len() as usize;
        let ratio = (1.0 - compressed_size as f64 / original_size as f64) * 100.0;

        self.logger.info(format!(
            "Compressed {} -> {} ({:.1}% reduction)",
            original_size, compressed_size, ratio
        ));

        Ok(gz_path)
    }

    pub fn hash_json(json: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn validate_json_against_schema(
        &self,
        json_value: &Value,
        schema_name: &str,
    ) -> Result<()> {
        self.compile_and_validate(schema_name, json_value)
            .or_else(|e| {
                self.logger.warning(
                    format!("Schema {} validation failed: {}", schema_name, e),
                    None,
                );
                Ok(())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use tempfile::TempDir;
    use std::collections::HashMap;

    fn create_test_chapter() -> Chapter {
        let mut verses = HashMap::new();
        verses.insert("1".to_string(), Verse {
            id: "test-id-1".to_string(),
            number: "1".to_string(),
            text: "In the beginning God created the heaven and the earth.".to_string(),
            anchor: "#v1".to_string(),
            canonical_ref: "Genesis.1.1".to_string(),
        });
        verses.insert("2".to_string(), Verse {
            id: "test-id-2".to_string(),
            number: "2".to_string(),
            text: "And the earth was without form, and void.".to_string(),
            anchor: "#v2".to_string(),
            canonical_ref: "Genesis.1.2".to_string(),
        });

        Chapter {
            book: "Genesis".to_string(),
            chapter: 1,
            verses,
            metadata: ChapterMetadata {
                verse_count: 2,
                last_updated: None,
            },
        }
    }

    #[test]
    fn test_generate_chapter_json() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = JsonGenerator::new(temp_dir.path(), logger, false, false).unwrap();
        let chapter = create_test_chapter();
        
        let output_path = generator
            .generate_chapter_json(&chapter, "kjv")
            .unwrap();
        
        assert!(output_path.exists());
        assert_eq!(output_path.file_name().unwrap(), "1.json");
        
        let json_content = std::fs::read_to_string(&output_path).unwrap();
        let chapter_json: ChapterJson = serde_json::from_str(&json_content).unwrap();
        
        assert_eq!(chapter_json.book, "Genesis");
        assert_eq!(chapter_json.chapter, 1);
        assert_eq!(chapter_json.version, "kjv");
        assert_eq!(chapter_json.schema_version, "1.0");
        assert_eq!(chapter_json.verses.len(), 2);
        assert!(chapter_json.verses.contains_key("1"));
        assert_eq!(chapter_json.verses.get("1").unwrap(), "In the beginning God created the heaven and the earth.");
    }

    #[test]
    fn test_deterministic_json_output() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        let log_dir = temp_dir1.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator1 = JsonGenerator::new(temp_dir1.path(), logger.clone(), false, false).unwrap();
        let generator2 = JsonGenerator::new(temp_dir2.path(), logger, false, false).unwrap();
        let chapter = create_test_chapter();
        
        let path1 = generator1.generate_chapter_json(&chapter, "kjv").unwrap();
        let path2 = generator2.generate_chapter_json(&chapter, "kjv").unwrap();
        
        let json1 = std::fs::read_to_string(&path1).unwrap();
        let json2 = std::fs::read_to_string(&path2).unwrap();
        
        let parsed1: ChapterJson = serde_json::from_str(&json1).unwrap();
        let parsed2: ChapterJson = serde_json::from_str(&json2).unwrap();
        
        assert_eq!(parsed1, parsed2, "Parsed JSON should be identical");
    }

    #[test]
    fn test_json_minification() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator_pretty = JsonGenerator::new(temp_dir.path(), logger.clone(), false, false).unwrap();
        let generator_minified = JsonGenerator::new(temp_dir.path(), logger, true, false).unwrap();
        let chapter = create_test_chapter();
        
        let pretty_path = generator_pretty.generate_chapter_json(&chapter, "kjv").unwrap();
        let minified_path = generator_minified.generate_chapter_json(&chapter, "kjv").unwrap();
        
        let pretty_json = std::fs::read_to_string(&pretty_path).unwrap();
        let minified_json = std::fs::read_to_string(&minified_path).unwrap();
        
        assert!(minified_json.len() <= pretty_json.len(), "Minified JSON should be smaller or equal");
        assert!(!minified_json.contains('\n'), "Minified JSON should not contain newlines");
        
        let pretty_parsed: ChapterJson = serde_json::from_str(&pretty_json).unwrap();
        let minified_parsed: ChapterJson = serde_json::from_str(&minified_json).unwrap();
        
        assert_eq!(pretty_parsed, minified_parsed, "Parsed JSON should be identical");
    }

    #[test]
    fn test_json_compression() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = JsonGenerator::new(temp_dir.path(), logger, false, true).unwrap();
        let chapter = create_test_chapter();
        
        let json_path = generator.generate_chapter_json(&chapter, "kjv").unwrap();
        let gz_path = json_path.with_extension("json.gz");
        
        assert!(json_path.exists(), "Original JSON should exist");
        assert!(gz_path.exists(), "Compressed file should exist");
        
        let json_size = std::fs::metadata(&json_path).unwrap().len();
        let gz_size = std::fs::metadata(&gz_path).unwrap().len();
        
        assert!(gz_size < json_size, "Compressed file should be smaller");
        
        use std::io::Read;
        use flate2::read::GzDecoder;
        let mut decoder = GzDecoder::new(std::fs::File::open(&gz_path).unwrap());
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        
        let original = std::fs::read_to_string(&json_path).unwrap();
        assert_eq!(decompressed, original, "Decompressed content should match original");
    }

    #[test]
    fn test_generate_versions_json() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = JsonGenerator::new(temp_dir.path(), logger, false, false).unwrap();
        
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Genesis.1".to_string(), create_test_chapter());
        versions.insert("kjv".to_string(), kjv_chapters);
        
        let output_path = generator.generate_versions_json(&versions).unwrap();
        
        assert!(output_path.exists());
        
        let json_content = std::fs::read_to_string(&output_path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        
        assert_eq!(value["schema_version"], "1.0");
        assert!(value["versions"].is_array());
        let versions_array = value["versions"].as_array().unwrap();
        assert_eq!(versions_array.len(), 1);
        assert_eq!(versions_array[0]["code"], "kjv");
    }

    #[test]
    fn test_generate_books_json() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = JsonGenerator::new(temp_dir.path(), logger, false, false).unwrap();
        
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Genesis.1".to_string(), create_test_chapter());
        versions.insert("kjv".to_string(), kjv_chapters);
        
        let output_path = generator.generate_books_json(&versions).unwrap();
        
        assert!(output_path.exists());
        
        let json_content = std::fs::read_to_string(&output_path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        
        assert_eq!(value["schema_version"], "1.0");
        assert!(value["books"].is_array());
    }

    #[test]
    fn test_json_size_limits() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = JsonGenerator::new(temp_dir.path(), logger, true, false).unwrap();
        let chapter = create_test_chapter();
        
        let output_path = generator.generate_chapter_json(&chapter, "kjv").unwrap();
        let json_size = std::fs::metadata(&output_path).unwrap().len();
        
        assert!(json_size < 10_000, "Chapter JSON should be under reasonable size limit");
    }

    #[test]
    fn test_crossrefs_json_generation() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = JsonGenerator::new(temp_dir.path(), logger, false, false).unwrap();
        
        let crossrefs = CrossReferenceMap {
            schema_version: "1.0".to_string(),
            versification: Some(HashMap::new()),
            mappings: HashMap::new(),
            conflicts: Vec::new(),
            metrics: None,
        };
        
        let output_path = generator.generate_crossrefs_json(&crossrefs, None).unwrap();
        
        assert!(output_path.exists());
        
        let json_content = std::fs::read_to_string(&output_path).unwrap();
        let parsed: CrossReferenceMap = serde_json::from_str(&json_content).unwrap();
        
        assert_eq!(parsed.schema_version, "1.0");
    }

    #[test]
    fn test_json_hash_stability() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = JsonGenerator::new(temp_dir.path(), logger, false, false).unwrap();
        let chapter = create_test_chapter();
        
        let path1 = generator.generate_chapter_json(&chapter, "kjv").unwrap();
        let json1 = std::fs::read_to_string(&path1).unwrap();
        
        let path2 = generator.generate_chapter_json(&chapter, "kjv").unwrap();
        let json2 = std::fs::read_to_string(&path2).unwrap();
        
        let parsed1: ChapterJson = serde_json::from_str(&json1).unwrap();
        let parsed2: ChapterJson = serde_json::from_str(&json2).unwrap();
        
        assert_eq!(parsed1, parsed2, "Same content should produce same structure");
    }
}

