use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub build_id: String,
    pub timestamp: String,
    pub summary: ReportSummary,
    pub entries: Vec<LogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub errors: usize,
    pub warnings: usize,
    pub processed: ProcessingStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub books: usize,
    pub chapters: usize,
    pub verses: usize,
}

#[derive(Clone)]
pub struct DiagnosticLogger {
    log_dir: PathBuf,
    log_file: Arc<Mutex<Option<BufWriter<File>>>>,
    build_id: String,
    entries: Arc<Mutex<Vec<LogEntry>>>,
    error_count: Arc<Mutex<usize>>,
    warning_count: Arc<Mutex<usize>>,
}

impl DiagnosticLogger {
    pub fn new(log_dir: &Path) -> Result<Self> {
        fs::create_dir_all(log_dir)
            .with_context(|| format!("Failed to create log directory: {:?}", log_dir))?;

        let build_id = Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let log_file_path = log_dir.join(format!("build-{}.jsonl", build_id));

        let log_file = Some(BufWriter::new(
            File::create(&log_file_path)
                .with_context(|| format!("Failed to create log file: {:?}", log_file_path))?,
        ));

        Ok(DiagnosticLogger {
            log_dir: log_dir.to_path_buf(),
            log_file: Arc::new(Mutex::new(log_file)),
            build_id: build_id.clone(),
            entries: Arc::new(Mutex::new(Vec::new())),
            error_count: Arc::new(Mutex::new(0)),
            warning_count: Arc::new(Mutex::new(0)),
        })
    }

    pub fn log(&self, level: LogLevel, message: String, context: Option<serde_json::Value>) {
        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            level: level.clone(),
            message: message.clone(),
            context,
        };

        match level {
            LogLevel::Error => {
                *self.error_count.lock().unwrap() += 1;
            }
            LogLevel::Warning => {
                *self.warning_count.lock().unwrap() += 1;
            }
            _ => {}
        }

        self.entries.lock().unwrap().push(entry.clone());

        if let Ok(mut file_opt) = self.log_file.lock() {
            if let Some(ref mut file) = *file_opt {
                if let Ok(json) = serde_json::to_string(&entry) {
                    let _ = writeln!(file, "{}", json);
                }
            }
        }
    }

    pub fn info(&self, message: String) {
        self.log(LogLevel::Info, message, None);
    }

    pub fn warning(&self, message: String, context: Option<serde_json::Value>) {
        self.log(LogLevel::Warning, message, context);
    }

    pub fn error(&self, message: String, context: Option<serde_json::Value>) {
        self.log(LogLevel::Error, message, context);
    }

    pub fn generate_report(&self, stats: ProcessingStats) -> Result<DiagnosticReport> {
        if let Ok(mut file_opt) = self.log_file.lock() {
            if let Some(ref mut file) = *file_opt {
                file.flush()
                    .context("Failed to flush log file before generating report")?;
            }
        }

        Ok(DiagnosticReport {
            build_id: self.build_id.clone(),
            timestamp: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            summary: ReportSummary {
                errors: *self.error_count.lock().unwrap(),
                warnings: *self.warning_count.lock().unwrap(),
                processed: stats,
            },
            entries: self.entries.lock().unwrap().clone(),
        })
    }

    pub fn rotate_logs(&self, max_builds: usize) -> Result<()> {
        let mut build_files: Vec<(PathBuf, DateTime<Utc>)> = Vec::new();

        for entry in WalkDir::new(&self.log_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.starts_with("build-") && file_name.ends_with(".jsonl") {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let datetime: DateTime<Utc> = modified.into();
                            build_files.push((entry.path().to_path_buf(), datetime));
                        }
                    }
                }
            }
        }

        if build_files.len() > max_builds {
            build_files.sort_by(|a, b| a.1.cmp(&b.1));

            let to_delete = build_files.len() - max_builds;
            for (path, _) in build_files.iter().take(to_delete) {
                fs::remove_file(path)
                    .with_context(|| format!("Failed to delete old log file: {:?}", path))?;
            }
        }

        Ok(())
    }

    pub fn build_id(&self) -> &str {
        &self.build_id
    }
}

impl Drop for DiagnosticLogger {
    fn drop(&mut self) {
        if let Ok(mut file_opt) = self.log_file.lock() {
            if let Some(ref mut file) = *file_opt {
                let _ = file.flush();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let logger = DiagnosticLogger::new(temp_dir.path()).unwrap();
        assert!(!logger.build_id().is_empty());
    }

    #[test]
    fn test_logging_levels() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = DiagnosticLogger::new(temp_dir.path()).unwrap();

        logger.info("Test info message".to_string());
        logger.warning("Test warning".to_string(), None);
        logger.error("Test error".to_string(), None);

        assert_eq!(*logger.error_count.lock().unwrap(), 1);
        assert_eq!(*logger.warning_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_log_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let logger = DiagnosticLogger::new(temp_dir.path()).unwrap();

        for i in 0..15 {
            let log_file = temp_dir.path().join(format!("build-{}.jsonl", i));
            File::create(&log_file).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        logger.rotate_logs(10).unwrap();

        let remaining_logs: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_str().unwrap().starts_with("build-"))
            .collect();

        assert!(remaining_logs.len() <= 10);
    }

    #[test]
    fn test_generate_report() {
        let temp_dir = TempDir::new().unwrap();
        let mut logger = DiagnosticLogger::new(temp_dir.path()).unwrap();

        logger.info("Info message".to_string());
        logger.warning("Warning message".to_string(), None);
        logger.error("Error message".to_string(), None);

        let stats = ProcessingStats {
            books: 66,
            chapters: 1189,
            verses: 31102,
        };

        let report = logger.generate_report(stats).unwrap();

        assert_eq!(report.summary.errors, 1);
        assert_eq!(report.summary.warnings, 1);
        assert_eq!(report.summary.processed.books, 66);
        assert_eq!(report.entries.len(), 3);
    }
}
