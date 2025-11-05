use crate::logger::*;
use crate::models::*;
use crate::schema::validate_json;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct ManifestGenerator {
    output_base: PathBuf,
    logger: DiagnosticLogger,
    build_timestamp: String,
}

impl ManifestGenerator {
    pub fn new(output_dir: &Path, logger: DiagnosticLogger) -> Result<Self> {
        let normalized_timestamp = Self::normalize_timestamp(&Utc::now());
        
        Ok(ManifestGenerator {
            output_base: output_dir.to_path_buf(),
            logger,
            build_timestamp: normalized_timestamp,
        })
    }

    pub fn with_timestamp(output_dir: &Path, logger: DiagnosticLogger, timestamp: DateTime<Utc>) -> Self {
        ManifestGenerator {
            output_base: output_dir.to_path_buf(),
            logger,
            build_timestamp: Self::normalize_timestamp(&timestamp),
        }
    }

    pub fn normalize_timestamp(dt: &DateTime<Utc>) -> String {
        dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }

    pub fn generate_manifest(
        &self,
        available_versions: &[String],
        source_checksums: HashMap<String, String>,
        schema_version: &str,
        mapper_thresholds: Option<(f64, f64)>,
        versification: Option<HashMap<String, Vec<String>>>,
        crossrefs_sha256: Option<String>,
    ) -> Result<GlobalManifest> {
        let mut sorted_versions = available_versions.to_vec();
        sorted_versions.sort();

        let mut sorted_checksums: Vec<(String, String)> = source_checksums.into_iter().collect();
        sorted_checksums.sort_by_key(|(k, _)| k.clone());

        let api_endpoints = ApiEndpoints {
            versions: "/versions.json".to_string(),
            books: "/books.json".to_string(),
            crossrefs: "/crossrefs.json".to_string(),
            chapters: "/{version}/{book}/{chapter}.json".to_string(),
        };

        let mut schema_locations = HashMap::new();
        schema_locations.insert("manifest".to_string(), format!("/schema/manifest-{}.json", schema_version));
        schema_locations.insert("chapter".to_string(), format!("/schema/chapter-{}.json", schema_version));
        schema_locations.insert("crossrefs".to_string(), format!("/schema/crossrefs-{}.json", schema_version));

        let mut extensions = serde_json::Map::new();
        
        let mapper_thresholds = mapper_thresholds.map(|(jaccard, levenshtein)| {
            crate::models::MapperThresholds {
                jaccard,
                levenshtein,
            }
        });

        let manifest = GlobalManifest {
            schema_version: schema_version.to_string(),
            build_timestamp: self.build_timestamp.clone(),
            source_checksums: sorted_checksums.into_iter().collect(),
            available_versions: sorted_versions,
            api_endpoints,
            schema_locations,
            mapper_thresholds,
            versification,
            crossrefs_sha256,
            extensions: json!(extensions),
        };

        Ok(manifest)
    }

    pub fn save_manifest(&self, manifest: &GlobalManifest, minify: bool) -> Result<PathBuf> {
        let output_path = self.output_base.join("manifest.json");

        let json_str = if minify {
            serde_json::to_string(manifest)?
        } else {
            serde_json::to_string_pretty(manifest)?
        };

        let schema_path = self.output_base.join("schema").join("manifest-1.0.json");
        if schema_path.exists() {
            let json_value: Value = serde_json::from_str(&json_str)?;
            if let Err(e) = validate_json(&json_value, &schema_path) {
                self.logger.warning(
                    format!("Manifest schema validation warning: {}", e),
                    None,
                );
            }
        }

        fs::write(&output_path, &json_str)
            .context("Failed to write manifest.json")?;

        let hash = Self::hash_manifest(&json_str);
        self.logger.info(format!(
            "Generated manifest.json ({} bytes, SHA-256: {})",
            json_str.len(),
            hash
        ));

        Ok(output_path)
    }

    pub fn hash_manifest(json: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn compute_file_checksum(file_path: &Path) -> Result<String> {
        let mut file = fs::File::open(file_path)
            .context(format!("Failed to open file for checksum: {:?}", file_path))?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .context("Failed to read file for checksum")?;
        
        let mut hasher = Sha256::new();
        hasher.update(&buffer);
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn compute_source_checksums(
        source_files: &[PathBuf],
    ) -> Result<HashMap<String, String>> {
        let mut checksums = HashMap::new();
        
        for file_path in source_files {
            if file_path.exists() {
                let checksum = Self::compute_file_checksum(file_path)?;
                let file_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                checksums.insert(file_name, checksum);
            }
        }
        
        Ok(checksums)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::collections::HashMap;

    #[test]
    fn test_manifest_generation() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = ManifestGenerator::new(temp_dir.path(), logger).unwrap();
        let versions = vec!["kjv".to_string(), "web".to_string()];
        let checksums = HashMap::new();
        
        let manifest = generator.generate_manifest(&versions, checksums, "1.0", None, None, None).unwrap();
        
        assert_eq!(manifest.schema_version, "1.0");
        assert_eq!(manifest.available_versions.len(), 2);
        assert_eq!(manifest.available_versions[0], "kjv");
        assert_eq!(manifest.available_versions[1], "web");
    }

    #[test]
    fn test_manifest_deterministic_timestamp() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let fixed_time = Utc::now();
        let generator1 = ManifestGenerator::with_timestamp(temp_dir.path(), logger.clone(), fixed_time);
        let generator2 = ManifestGenerator::with_timestamp(temp_dir.path(), logger, fixed_time);
        
        let versions = vec!["kjv".to_string()];
        let checksums = HashMap::new();
        
        let manifest1 = generator1.generate_manifest(&versions, checksums.clone(), "1.0", None, None, None).unwrap();
        let manifest2 = generator2.generate_manifest(&versions, checksums, "1.0", None, None, None).unwrap();
        
        assert_eq!(manifest1.build_timestamp, manifest2.build_timestamp);
    }

    #[test]
    fn test_manifest_schema_validation() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = ManifestGenerator::new(temp_dir.path(), logger).unwrap();
        let versions = vec!["kjv".to_string()];
        let checksums = HashMap::new();
        
        let manifest = generator.generate_manifest(&versions, checksums, "1.0", None, None, None).unwrap();
        let output_path = generator.save_manifest(&manifest, false).unwrap();
        
        assert!(output_path.exists());
        
        let json_content = std::fs::read_to_string(&output_path).unwrap();
        let parsed: GlobalManifest = serde_json::from_str(&json_content).unwrap();
        
        assert_eq!(parsed.schema_version, "1.0");
        assert_eq!(parsed.available_versions.len(), 1);
    }

    #[test]
    fn test_manifest_hash_stability() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let fixed_time = Utc::now();
        let generator = ManifestGenerator::with_timestamp(temp_dir.path(), logger, fixed_time);
        let versions = vec!["kjv".to_string()];
        let checksums = HashMap::new();
        
        let manifest = generator.generate_manifest(&versions, checksums, "1.0", None, None, None).unwrap();
        let json1 = serde_json::to_string(&manifest).unwrap();
        let json2 = serde_json::to_string(&manifest).unwrap();
        
        let hash1 = ManifestGenerator::hash_manifest(&json1);
        let hash2 = ManifestGenerator::hash_manifest(&json2);
        
        assert_eq!(hash1, hash2, "Same manifest should produce same hash");
    }

    #[test]
    fn test_compute_file_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "Hello, World!").unwrap();
        
        let checksum1 = ManifestGenerator::compute_file_checksum(&test_file).unwrap();
        let checksum2 = ManifestGenerator::compute_file_checksum(&test_file).unwrap();
        
        assert_eq!(checksum1, checksum2, "Same file should produce same checksum");
        assert_eq!(checksum1.len(), 64, "SHA-256 should be 64 hex characters");
    }

    #[test]
    fn test_compute_source_checksums() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        
        std::fs::write(&file1, "Content 1").unwrap();
        std::fs::write(&file2, "Content 2").unwrap();
        
        let source_files = vec![file1.clone(), file2.clone()];
        let checksums = ManifestGenerator::compute_source_checksums(&source_files).unwrap();
        
        assert_eq!(checksums.len(), 2);
        assert!(checksums.contains_key("file1.txt"));
        assert!(checksums.contains_key("file2.txt"));
    }

    #[test]
    fn test_manifest_sorted_versions() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = ManifestGenerator::new(temp_dir.path(), logger).unwrap();
        let versions = vec!["web".to_string(), "kjv".to_string(), "asv".to_string()];
        let checksums = HashMap::new();
        
        let manifest = generator.generate_manifest(&versions, checksums, "1.0", None, None, None).unwrap();
        
        assert_eq!(manifest.available_versions, vec!["asv", "kjv", "web"]);
    }

    #[test]
    fn test_manifest_api_endpoints() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = ManifestGenerator::new(temp_dir.path(), logger).unwrap();
        let versions = vec!["kjv".to_string()];
        let checksums = HashMap::new();
        
        let manifest = generator.generate_manifest(&versions, checksums, "1.0", None, None, None).unwrap();
        
        assert_eq!(manifest.api_endpoints.versions, "/versions.json");
        assert_eq!(manifest.api_endpoints.books, "/books.json");
        assert_eq!(manifest.api_endpoints.crossrefs, "/crossrefs.json");
        assert_eq!(manifest.api_endpoints.chapters, "/{version}/{book}/{chapter}.json");
    }

    #[test]
    fn test_full_pipeline_reproducibility() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        let log_dir1 = temp_dir1.path().join("logs");
        let log_dir2 = temp_dir2.path().join("logs");
        let logger1 = DiagnosticLogger::new(&log_dir1).unwrap();
        let logger2 = DiagnosticLogger::new(&log_dir2).unwrap();
        
        let fixed_time = Utc::now();
        let generator1 = ManifestGenerator::with_timestamp(temp_dir1.path(), logger1, fixed_time);
        let generator2 = ManifestGenerator::with_timestamp(temp_dir2.path(), logger2, fixed_time);
        
        let versions = vec!["kjv".to_string()];
        let checksums = HashMap::new();
        
        let manifest1 = generator1.generate_manifest(&versions, checksums.clone(), "1.0", None, None, None).unwrap();
        let manifest2 = generator2.generate_manifest(&versions, checksums, "1.0", None, None, None).unwrap();
        
        assert_eq!(manifest1, manifest2, "Two consecutive builds with same input should produce identical manifests");
        
        let json1 = serde_json::to_string_pretty(&manifest1).unwrap();
        let json2 = serde_json::to_string_pretty(&manifest2).unwrap();
        
        let parsed1: GlobalManifest = serde_json::from_str(&json1).unwrap();
        let parsed2: GlobalManifest = serde_json::from_str(&json2).unwrap();
        
        assert_eq!(parsed1, parsed2, "Parsed manifests should be identical");
    }
}
