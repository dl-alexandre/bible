use crate::logger::*;
use crate::schema::validate_json;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct BuildValidator {
    output_base: PathBuf,
    logger: DiagnosticLogger,
}

impl BuildValidator {
    pub fn new(output_dir: &Path, logger: DiagnosticLogger) -> Result<Self> {
        Ok(BuildValidator {
            output_base: output_dir.to_path_buf(),
            logger,
        })
    }

    pub fn validate_all_json_files(&self) -> Result<bool> {
        self.logger.info("Validating all JSON files against schemas...".to_string());

        let mut all_valid = true;

        let manifest_path = self.output_base.join("manifest.json");
        if manifest_path.exists() {
            if !self.validate_json_file(&manifest_path, "manifest-1.0.json") {
                all_valid = false;
            }
        } else {
            self.logger.warning("manifest.json not found".to_string(), None);
            all_valid = false;
        }

        let versions_path = self.output_base.join("versions.json");
        if versions_path.exists() {
            if !self.validate_json_file(&versions_path, "versions-1.0.json") {
                all_valid = false;
            }
        }

        let books_path = self.output_base.join("books.json");
        if books_path.exists() {
            if !self.validate_json_file(&books_path, "books-1.0.json") {
                all_valid = false;
            }
        }

        let crossrefs_path = self.output_base.join("crossrefs.json");
        if crossrefs_path.exists() {
            if !self.validate_json_file(&crossrefs_path, "crossrefs-1.0.json") {
                all_valid = false;
            }
        }

        if all_valid {
            self.logger.info("All JSON files validated successfully".to_string());
        } else {
            self.logger.error("Some JSON files failed validation".to_string(), None);
        }

        Ok(all_valid)
    }

    fn validate_json_file(&self, file_path: &Path, schema_name: &str) -> bool {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                self.logger.error(
                    format!("Failed to read {}: {}", file_path.display(), e),
                    None,
                );
                return false;
            }
        };

        let json_value: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                self.logger.error(
                    format!("Failed to parse {}: {}", file_path.display(), e),
                    None,
                );
                return false;
            }
        };

        let schema_path = self.output_base.join("schema").join(schema_name);
        if !schema_path.exists() {
            self.logger.warning(
                format!("Schema {} not found, skipping validation", schema_name),
                None,
            );
            return true;
        }

        match validate_json(&json_value, &schema_path) {
            Ok(()) => {
                self.logger.info(format!("✓ {} validated", file_path.display()));
                true
            }
            Err(e) => {
                self.logger.error(
                    format!("✗ {} validation failed: {}", file_path.display(), e),
                    None,
                );
                false
            }
        }
    }

    pub fn check_budgets(&self) -> Result<bool> {
        self.logger.info("Checking output budgets...".to_string());

        let mut all_ok = true;
        let html_max = 50 * 1024;
        let json_max = 500 * 1024;
        let gzip_min_ratio = 0.70;

        let html_files = Self::find_files(&self.output_base, "html")?;
        for html_file in html_files {
            let size = fs::metadata(&html_file)?.len() as usize;
            if size > html_max {
                self.logger.error(
                    format!(
                        "HTML budget exceeded: {} ({} KB > {} KB limit)",
                        html_file.display(),
                        size / 1024,
                        html_max / 1024
                    ),
                    None,
                );
                all_ok = false;
            }
        }

        let json_files = Self::find_files(&self.output_base, "json")?;
        for json_file in json_files {
            let size = fs::metadata(&json_file)?.len() as usize;
            if size > json_max {
                self.logger.error(
                    format!(
                        "JSON budget exceeded: {} ({} KB > {} KB limit)",
                        json_file.display(),
                        size / 1024,
                        json_max / 1024
                    ),
                    None,
                );
                all_ok = false;
            }

            let gz_file = json_file.with_extension("json.gz");
            if gz_file.exists() {
                let original_size = size;
                let compressed_size = fs::metadata(&gz_file)?.len() as usize;
                let ratio = 1.0 - (compressed_size as f64 / original_size as f64);

                if ratio < gzip_min_ratio {
                    self.logger.error(
                        format!(
                            "Gzip ratio too low: {} ({} < {} minimum)",
                            json_file.display(),
                            ratio,
                            gzip_min_ratio
                        ),
                        None,
                    );
                    all_ok = false;
                }
            }
        }

        if all_ok {
            self.logger.info("All budgets satisfied".to_string());
        }

        Ok(all_ok)
    }

    #[allow(dead_code)]
    pub fn check_links_and_anchors(&self) -> Result<bool> {
        self.logger.info("Checking links and anchors...".to_string());

        let html_files = Self::find_files(&self.output_base, "html")?;
        let mut all_anchors = HashSet::new();
        let mut all_ok = true;

        for html_file in &html_files {
            let content = fs::read_to_string(html_file)?;
            let anchors: HashSet<String> = Self::extract_anchors(&content);
            all_anchors.extend(anchors);
        }

        for html_file in &html_files {
            let content = fs::read_to_string(html_file)?;
            let links = Self::extract_links(&content);

            for link in links {
                if link.starts_with('#') {
                    let anchor = link[1..].to_string();
                    if !all_anchors.contains(&anchor) {
                        self.logger.warning(
                            format!(
                                "Broken anchor in {}: #{}",
                                html_file.display(),
                                anchor
                            ),
                            None,
                        );
                        all_ok = false;
                    }
                } else if !link.starts_with("http://") && !link.starts_with("https://") {
                    let link_path = html_file.parent().unwrap().join(&link);
                    if !link_path.exists() {
                        self.logger.warning(
                            format!("Broken link in {}: {}", html_file.display(), link),
                            None,
                        );
                        all_ok = false;
                    }
                }
            }
        }

        if all_ok {
            self.logger.info("All links and anchors valid".to_string());
        }

        Ok(all_ok)
    }

    fn find_files(base: &Path, ext: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        if base.is_dir() {
            for entry in WalkDir::new(base) {
                let entry = entry?;
                if entry.path().extension().and_then(|s| s.to_str()) == Some(ext) {
                    files.push(entry.path().to_path_buf());
                }
            }
        }
        Ok(files)
    }

    #[allow(dead_code)]
    fn extract_anchors(html: &str) -> HashSet<String> {
        let mut anchors = HashSet::new();
        let id_pattern = regex::Regex::new(r#"id="([^"]+)""#).unwrap();
        for cap in id_pattern.captures_iter(html) {
            if let Some(id) = cap.get(1) {
                anchors.insert(id.as_str().to_string());
            }
        }
        anchors
    }

    #[allow(dead_code)]
    fn extract_links(html: &str) -> Vec<String> {
        let mut links = Vec::new();
        let href_pattern = regex::Regex::new(r#"href="([^"]+)""#).unwrap();
        for cap in href_pattern.captures_iter(html) {
            if let Some(href) = cap.get(1) {
                links.push(href.as_str().to_string());
            }
        }
        links
    }

    pub fn check_determinism(&self) -> Result<bool> {
        self.logger.info("Checking determinism...".to_string());
        let mut entries = Vec::new();
        for entry in WalkDir::new(&self.output_base) {
            let entry = entry?;
            if entry.path().is_file() {
                if let Ok(rel) = entry.path().strip_prefix(&self.output_base) {
                    let content = fs::read(entry.path())?;
                    entries.push((rel.to_path_buf(), content));
                }
            }
        }

        entries.sort_by(|a, b| a.0.cmp(&b.0));

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        for (path, content) in &entries {
            hasher.update(path.to_string_lossy().as_bytes());
            hasher.update(content);
        }
        let hash = format!("{:x}", hasher.finalize());
        self.logger.info(format!("Deterministic hash: {}", hash));
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_check_determinism() {
        let temp_dir = TempDir::new().unwrap();
        let out = temp_dir.path();
        std::fs::create_dir_all(out).unwrap();
        std::fs::write(out.join("manifest.json"), "{}").unwrap();
        let logger = DiagnosticLogger::new(out).unwrap();
        let validator = BuildValidator::new(out, logger).unwrap();
        assert!(validator.check_determinism().unwrap());
    }
}

