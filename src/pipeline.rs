use crate::html_generator::HtmlGenerator;
use crate::json_generator::JsonGenerator;
use crate::logger::*;
use crate::manifest_generator::ManifestGenerator;
use crate::mapper::*;
use crate::models::*;
use crate::parser::*;
use crate::site_generator::SiteGenerator;
use crate::validation::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ProcessingPipeline {
    pub parser: TextParser,
    pub validator: InputValidator,
    pub mapper: CrossVersionMapper,
    pub logger: DiagnosticLogger,
}

impl ProcessingPipeline {
    pub fn new(log_dir: &Path) -> Result<Self> {
        Ok(ProcessingPipeline {
            parser: TextParser::new()
                .context("Failed to create TextParser")?,
            validator: InputValidator::new()
                .context("Failed to create InputValidator")?,
            mapper: CrossVersionMapper::new()
                .context("Failed to create CrossVersionMapper")?,
            logger: DiagnosticLogger::new(log_dir)
                .context("Failed to create DiagnosticLogger")?,
        })
    }

    pub fn process_version(
        &mut self,
        text: &str,
        format: &BibleFormat,
        version_code: &str,
    ) -> Result<(SourceText, HashMap<String, Chapter>)> {
        self.logger.info(format!("Processing version: {}", version_code));

        let source_text = self
            .parser
            .parse_source_text(text, *format, version_code)
            .with_context(|| format!("Failed to parse source text for {}", version_code))?;

        let validation_result = self.validator.validate_dataset(&source_text);
        if !validation_result.is_valid {
            for error in &validation_result.errors {
                self.logger.error(
                    error.message.clone(),
                    Some(serde_json::json!({
                        "version": version_code,
                        "book": error.context.book,
                        "chapter": error.context.chapter,
                        "verse": error.context.verse,
                    })),
                );
            }
        }

        for warning in &validation_result.warnings {
            self.logger.warning(
                warning.message.clone(),
                Some(serde_json::json!({
                    "version": version_code,
                    "book": warning.context.book,
                    "chapter": warning.context.chapter,
                    "verse": warning.context.verse,
                })),
            );
        }

        let mut chapters = HashMap::new();
        for book in &source_text.books {
            for chapter_data in &book.chapters {
                let chapter_key = format!("{}.{}", book.name, chapter_data.number);
                let chapter_text = chapter_data
                    .verses
                    .iter()
                    .map(|v| format!("{} {}", v.number, v.text))
                    .collect::<Vec<_>>()
                    .join("\n");

                let chapter = self
                    .parser
                    .parse_chapter(&chapter_text, &book.name, chapter_data.number, version_code)
                    .with_context(|| {
                        format!(
                            "Failed to parse chapter {} of {}",
                            chapter_data.number, book.name
                        )
                    })?;

                chapters.insert(chapter_key, chapter);
            }
        }

        self.logger.info(format!(
            "Processed {} books, {} chapters, {} verses for {}",
            source_text.books.len(),
            validation_result.statistics.total_chapters,
            validation_result.statistics.total_verses,
            version_code
        ));

        Ok((source_text, chapters))
    }

    pub fn generate_cross_references(
        &mut self,
        version_chapters: &HashMap<String, HashMap<String, Chapter>>,
    ) -> Result<CrossReferenceMap> {
        self.logger.info("Generating cross-version mappings...".to_string());

        let mappings = self
            .mapper
            .generate_mappings_with_fallback(version_chapters)
            .context("Failed to generate cross-version mappings")?;

        let metrics = mappings.metrics.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Metrics not computed"))?;

        self.logger.info(format!(
            "Mapping summary: {} total, {} mapped, {} nulls, {} conflicts ({:.2}% coverage)",
            metrics.total,
            metrics.mapped,
            metrics.nulls,
            metrics.conflicts,
            metrics.coverage * 100.0
        ));

        self.logger.info(format!(
            "Similarity thresholds: Jaccard={:.2}, Levenshtein={:.2}",
            metrics.similarity_thresholds.jaccard,
            metrics.similarity_thresholds.levenshtein
        ));

        if metrics.nulls > 0 {
            self.logger.warning(
                format!("{} references have null entries", metrics.nulls),
                None,
            );
        }

        if metrics.conflicts > 0 {
            let split_count = mappings.conflicts.iter()
                .filter(|c| matches!(c.conflict_type, crate::models::ConflictType::Split))
                .count();
            let merge_count = mappings.conflicts.iter()
                .filter(|c| matches!(c.conflict_type, crate::models::ConflictType::Merge))
                .count();
            let shift_count = mappings.conflicts.iter()
                .filter(|c| matches!(c.conflict_type, crate::models::ConflictType::Shift))
                .count();
            let absent_count = mappings.conflicts.iter()
                .filter(|c| matches!(c.conflict_type, crate::models::ConflictType::Absent))
                .count();

            self.logger.warning(
                format!(
                    "{} conflicts detected: {} split, {} merge, {} shift, {} absent",
                    metrics.conflicts, split_count, merge_count, shift_count, absent_count
                ),
                None,
            );
        }

        if let Some(ref versification) = mappings.versification {
            let schemes: Vec<String> = versification.keys().cloned().collect();
            self.logger.info(format!(
                "Versification schemes: {}",
                schemes.join(", ")
            ));
        }

        Ok(mappings)
    }

    pub fn save_cross_references(
        &mut self,
        mappings: &CrossReferenceMap,
        output_path: &Path,
    ) -> Result<()> {
        self.logger.info(format!(
            "Saving cross-references to {:?}",
            output_path
        ));

        self.mapper
            .generate_crossrefs_json(mappings, output_path)
            .context("Failed to save cross-reference JSON")?;

        self.logger.info("Cross-references saved successfully".to_string());

        Ok(())
    }

    pub fn generate_html(
        &self,
        chapters: &HashMap<String, Chapter>,
        version_code: &str,
        version_name: &str,
        template_dir: &Path,
        output_dir: &Path,
        crossrefs: Option<&CrossReferenceMap>,
        base_url: &str,
    ) -> Result<()> {
        self.logger.info(format!(
            "Generating HTML for {} chapters...",
            chapters.len()
        ));

        let html_generator = HtmlGenerator::new(template_dir, output_dir, self.logger.clone(), base_url)
            .context("Failed to create HTML generator")?;

        let mut redirect_count = 0;
        let mut books_map: std::collections::HashMap<String, Vec<u32>> = std::collections::HashMap::new();

        for (chapter_key, chapter) in chapters {
            let chapter_path = html_generator
                .generate_chapter_html(chapter, version_code, version_name, crossrefs)
                .with_context(|| format!("Failed to generate HTML for {}", chapter_key))?;

            let redirects = html_generator
                .generate_all_redirects(chapter, version_code, version_name, &chapter_path)
                .with_context(|| format!("Failed to generate redirects for {}", chapter_key))?;

            redirect_count += redirects.len();

            books_map
                .entry(chapter.book.clone())
                .or_insert_with(Vec::new)
                .push(chapter.chapter);
        }

        let mut books: Vec<String> = books_map.keys().cloned().collect();
        books.sort();

        for (book, chapters) in &books_map {
            let mut sorted_chapters = chapters.clone();
            sorted_chapters.sort();
            html_generator
                .generate_book_index(version_code, version_name, book, &sorted_chapters)
                .with_context(|| format!("Failed to generate book index for {}", book))?;
        }

        html_generator
            .generate_version_index(version_code, version_name, &books)
            .context("Failed to generate version index")?;

        self.logger.info(format!(
            "Generated {} HTML files, {} redirects, {} book indices, and 1 version index",
            chapters.len(),
            redirect_count,
            books_map.len()
        ));

        Ok(())
    }

    pub fn generate_json_api(
        &self,
        all_versions: &HashMap<String, HashMap<String, Chapter>>,
        crossrefs: Option<&CrossReferenceMap>,
        output_dir: &Path,
        minify: bool,
        compress: bool,
    ) -> Result<Option<String>> {
        self.logger.info("Generating JSON API...".to_string());

        let json_generator = JsonGenerator::new(output_dir, self.logger.clone(), minify, compress)
            .context("Failed to create JSON generator")?;

        let mut total_chapters = 0;
        for (version_code, chapters) in all_versions {
            for (chapter_key, chapter) in chapters {
                json_generator
                    .generate_chapter_json(chapter, version_code)
                    .with_context(|| format!("Failed to generate JSON for {}", chapter_key))?;
                total_chapters += 1;
            }
        }

        json_generator
            .generate_versions_json(all_versions)
            .context("Failed to generate versions.json")?;

        json_generator
            .generate_books_json(all_versions)
            .context("Failed to generate books.json")?;

        let crossrefs_sha256 = if let Some(crossrefs) = crossrefs {
            let crossrefs_path = json_generator
                .generate_crossrefs_json(crossrefs, None)
                .context("Failed to generate crossrefs.json")?;
            
            let crossrefs_content = fs::read_to_string(&crossrefs_path)?;
            Some(JsonGenerator::hash_json(&crossrefs_content))
        } else {
            None
        };

        self.logger.info(format!(
            "Generated JSON API: {} chapters, metadata files, {} compression",
            total_chapters,
            if compress { "with" } else { "without" }
        ));

        Ok(crossrefs_sha256)
    }

    pub fn generate_manifest_and_site(
        &self,
        all_versions: &HashMap<String, HashMap<String, Chapter>>,
        source_files: &[PathBuf],
        output_dir: &Path,
        schema_version: &str,
        minify: bool,
        mapper_thresholds: Option<(f64, f64)>,
        versification: Option<HashMap<String, Vec<String>>>,
        crossrefs_sha256: Option<String>,
        base_url: &str,
    ) -> Result<String> {
        self.logger.info("Generating manifest and site...".to_string());

        let manifest_generator = ManifestGenerator::new(output_dir, self.logger.clone())
            .context("Failed to create manifest generator")?;

        let available_versions: Vec<String> = all_versions.keys().cloned().collect();
        let source_checksums = ManifestGenerator::compute_source_checksums(source_files)
            .context("Failed to compute source checksums")?;

        let manifest = manifest_generator
            .generate_manifest(
                &available_versions,
                source_checksums,
                schema_version,
                mapper_thresholds,
                versification,
                crossrefs_sha256,
            )
            .context("Failed to generate manifest")?;

        let manifest_path = manifest_generator
            .save_manifest(&manifest, minify)
            .context("Failed to save manifest")?;

        let manifest_hash = ManifestGenerator::hash_manifest(
            &fs::read_to_string(&manifest_path)?
        );

        let site_generator = SiteGenerator::new(output_dir, self.logger.clone())
            .context("Failed to create site generator")?;

        site_generator
            .generate_index(all_versions, base_url)
            .context("Failed to generate index")?;

        site_generator
            .ensure_deterministic_structure()
            .context("Failed to ensure deterministic structure")?;

        self.logger.info(format!(
            "Generated manifest (SHA-256: {}) and site index",
            manifest_hash
        ));

        Ok(manifest_hash)
    }

    pub fn finalize(&self, stats: ProcessingStats) -> Result<DiagnosticReport> {
        self.logger
            .generate_report(stats)
            .context("Failed to generate diagnostic report")
    }

    pub fn generate_deterministic_build(
        &self,
        output_dir: &Path,
    ) -> Result<String> {
        self.logger.info("Ensuring deterministic build structure...".to_string());
        
        let site_generator = SiteGenerator::new(output_dir, self.logger.clone())
            .context("Failed to create site generator")?;
        
        site_generator
            .ensure_deterministic_structure()
            .context("Failed to ensure deterministic structure")?;
        
        let manifest_path = output_dir.join("manifest.json");
        if manifest_path.exists() {
            let manifest_content = fs::read_to_string(&manifest_path)?;
            let hash = ManifestGenerator::hash_manifest(&manifest_content);
            self.logger.info(format!("Manifest hash: {}", hash));
            Ok(hash)
        } else {
            Ok(String::new())
        }
    }

    pub fn rotate_logs(&self) -> Result<()> {
        self.logger.rotate_logs(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_processing_pipeline() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let mut pipeline = ProcessingPipeline::new(&log_dir).unwrap();

        let kjv_text = "Chapter 1\n1 In the beginning God created the heaven and the earth.\n2 And the earth was without form, and void;";
        let (source, chapters) = pipeline
            .process_version(kjv_text, &BibleFormat::KJV, "kjv")
            .unwrap();

        assert_eq!(source.version, "kjv");
        assert!(!chapters.is_empty());
    }

    #[test]
    fn test_cross_references_generation() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let mut pipeline = ProcessingPipeline::new(&log_dir).unwrap();

        let mut version_chapters: HashMap<String, HashMap<String, Chapter>> = HashMap::new();

        let mut kjv_chapters: HashMap<String, Chapter> = HashMap::new();
        let mut kjv_verses = HashMap::new();
        kjv_verses.insert(
            "1".to_string(),
            Verse {
                id: "test1".to_string(),
                number: "1".to_string(),
                text: "In the beginning God created the heaven and the earth.".to_string(),
                anchor: "#v1".to_string(),
                canonical_ref: "Genesis.1.1".to_string(),
            },
        );
        kjv_chapters.insert(
            "Genesis.1".to_string(),
            Chapter {
                book: "Genesis".to_string(),
                chapter: 1,
                verses: kjv_verses,
                metadata: ChapterMetadata {
                    verse_count: 1,
                    last_updated: None,
                },
            },
        );

        version_chapters.insert("kjv".to_string(), kjv_chapters);

        let mappings = pipeline
            .generate_cross_references(&version_chapters)
            .unwrap();

        assert_eq!(mappings.schema_version, "1.0");
    }
}
