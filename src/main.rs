mod cli;
mod html_generator;
mod json_generator;
mod logger;
mod manifest_generator;
mod mapper;
mod mapper_config;
#[cfg(test)]
mod mapper_tests;
mod models;
mod parser;
mod pipeline;
#[cfg(test)]
mod security_tests;
mod schema;
mod site_generator;
mod text_normalizer;
mod validation;
mod validator;

use crate::cli::Cli;
use crate::mapper_config::MapperConfig;
use crate::parser::BibleFormat;
use crate::pipeline::ProcessingPipeline;
use crate::validator::BuildValidator;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Bible Static Generator starting...");

    if cli.validate_only {
        println!("Validation-only mode: checking schemas...");
        let log_dir = cli.log_dir.as_deref().unwrap_or_else(|| Path::new("logs"));
        let logger = crate::logger::DiagnosticLogger::new(log_dir)
            .context("Failed to create logger")?;
        let validator = BuildValidator::new(&cli.out, logger)
            .context("Failed to create validator")?;
        validator.validate_all_json_files()?;
        return Ok(());
    }

    if cli.datasets.is_empty() {
        eprintln!("Error: No datasets provided. Use --datasets to specify input files.");
        return Err(anyhow::anyhow!("No datasets specified"));
    }

    let output_dir = &cli.out;
    let schema_dir = output_dir.join("schema");
    fs::create_dir_all(&schema_dir)
        .context("Failed to create schema directory")?;
    schema::generate_schemas(&schema_dir)
        .map_err(|e| anyhow::anyhow!("Schema generation failed: {}", e))?;
    fs::create_dir_all(output_dir)
        .context("Failed to create output directory")?;

    let log_dir = cli.log_dir.as_deref().unwrap_or_else(|| Path::new("logs"));
    fs::create_dir_all(log_dir)
        .context("Failed to create log directory")?;

    println!("Output directory: {:?}", output_dir);
    println!("Log directory: {:?}", log_dir);
    println!("Processing {} dataset(s)...", cli.datasets.len());

    let mapper_config = MapperConfig::new(cli.threshold_jaccard, cli.threshold_lev);
    let mapper = crate::mapper::CrossVersionMapper::with_config(&mapper_config)
        .context("Failed to create mapper with config")?;
    
    let mut pipeline = ProcessingPipeline {
        parser: crate::parser::TextParser::new()
            .context("Failed to create TextParser")?,
        validator: crate::validation::InputValidator::new()
            .context("Failed to create InputValidator")?,
        mapper,
        logger: crate::logger::DiagnosticLogger::new(log_dir)
            .context("Failed to create DiagnosticLogger")?,
    };

    let mut all_versions: HashMap<String, HashMap<String, crate::models::Chapter>> = HashMap::new();
    let mut source_files = Vec::new();

    for dataset_path in &cli.datasets {
        if !dataset_path.exists() {
            eprintln!("Warning: Dataset file not found: {:?}", dataset_path);
            continue;
        }

        source_files.push(dataset_path.clone());

        let content = fs::read_to_string(dataset_path)
            .with_context(|| format!("Failed to read dataset: {:?}", dataset_path))?;

        let version_code = dataset_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let format = detect_format(&content, &version_code);

        let (_source_text, chapters) = pipeline
            .process_version(&content, &format, &version_code)
            .with_context(|| format!("Failed to process version: {}", version_code))?;

        all_versions.insert(version_code, chapters);
    }

    if all_versions.is_empty() {
        return Err(anyhow::anyhow!("No valid datasets processed"));
    }

    println!("Generating cross-references...");
    let crossrefs = pipeline
        .generate_cross_references(&all_versions)
        .context("Failed to generate cross-references")?;

    let versification = crossrefs.versification.clone();
    let mapper_thresholds = crossrefs.metrics.as_ref().map(|m| {
        (m.similarity_thresholds.jaccard, m.similarity_thresholds.levenshtein)
    });

    let template_dir = cli.templates.as_deref().unwrap_or_else(|| Path::new("templates"));
    let html_generator = crate::html_generator::HtmlGenerator::new(
        template_dir,
        output_dir,
        pipeline.logger.clone(),
        &cli.base_url,
    )
    .context("Failed to create HTML generator")?;

    println!("Generating HTML pages...");
    
    let mut available_versions: Vec<(String, String)> = all_versions.keys()
        .map(|code| (code.clone(), version_display_name(code)))
        .collect();
    available_versions.sort_by(|a, b| a.0.cmp(&b.0));

    for (version_code, chapters) in &all_versions {
        let mut books_map: std::collections::HashMap<String, Vec<u32>> = std::collections::HashMap::new();
        for chapter in chapters.values() {
            books_map.entry(chapter.book.clone()).or_insert_with(Vec::new).push(chapter.chapter);
        }

        for chapters_list in books_map.values_mut() {
            chapters_list.sort();
            chapters_list.dedup();
        }

        for (book, chapter_numbers) in &books_map {
            for (idx, chapter_num) in chapter_numbers.iter().enumerate() {
                let key = format!("{}.{}", book, chapter_num);
                if let Some(chapter) = chapters.get(&key) {
                    let prev = if idx > 0 { Some(chapter_numbers[idx - 1]) } else { None };
                    let next = if idx + 1 < chapter_numbers.len() { Some(chapter_numbers[idx + 1]) } else { None };
                    html_generator
                        .generate_chapter_html(
                            chapter,
                            version_code,
                            &version_display_name(version_code),
                            &available_versions,
                            prev,
                            next,
                            Some(&crossrefs),
                        )
                        .with_context(|| format!("Failed to generate HTML for {}", key))?;
                }
            }
        }

        let mut books: Vec<String> = books_map.keys().cloned().collect();
        books.sort();

        for (book, chapter_numbers) in &books_map {
            html_generator
                .generate_book_index(version_code, &version_display_name(version_code), book, chapter_numbers)
                .with_context(|| format!("Failed to generate book index for {}", book))?;
        }

        html_generator
            .generate_version_index(version_code, &version_display_name(version_code), &books)
            .with_context(|| format!("Failed to generate version index for {}", version_code))?;
    }

    println!("Generating JSON API...");
    let crossrefs_sha256 = pipeline
        .generate_json_api(&all_versions, Some(&crossrefs), output_dir, cli.minify_json, cli.gzip_json)
        .context("Failed to generate JSON API")?;

    println!("Generating manifest and site...");
    pipeline
        .generate_manifest_and_site(
            &all_versions,
            &source_files,
            output_dir,
            &cli.schema_version,
            cli.minify_json,
            mapper_thresholds,
            versification,
            crossrefs_sha256,
            &cli.base_url,
        )
        .context("Failed to generate manifest and site")?;

    let site_generator = crate::site_generator::SiteGenerator::new(output_dir, pipeline.logger.clone())
        .context("Failed to create site generator")?;

    site_generator
        .generate_sitemap(&all_versions, &cli.base_url)
        .context("Failed to generate sitemap")?;

    site_generator
        .generate_robots_txt()
        .context("Failed to generate robots.txt")?;

    let stats = crate::logger::ProcessingStats {
        books: all_versions.values().flat_map(|chapters| chapters.keys()).collect::<std::collections::HashSet<_>>().len(),
        chapters: all_versions.values().map(|chapters| chapters.len()).sum(),
        verses: all_versions.values().flat_map(|chapters| chapters.values()).map(|c| c.verses.len()).sum(),
    };

    let report = pipeline.finalize(stats)?;
    println!("Build complete!");
    println!("Errors: {}, Warnings: {}", report.summary.errors, report.summary.warnings);

    println!("Running validation checks...");
    let validator = BuildValidator::new(output_dir, pipeline.logger.clone())?;
    validator.validate_all_json_files()?;
    validator.check_budgets()?;
    validator.check_determinism()?;

    Ok(())
}

fn detect_format(content: &str, version_code: &str) -> BibleFormat {
    let lower = version_code.to_lowercase();
    if lower.contains("bsb") || content.contains("Berean Standard Bible") || content.contains("BSB") {
        BibleFormat::BSB
    } else if lower.contains("web") || content.contains("WEB") || content.contains("World English Bible") {
        BibleFormat::WEB
    } else if lower.contains("asv") || content.contains("American Standard Version") {
        BibleFormat::ASV
    } else if lower.contains("oeb") || content.contains("Open English Bible") {
        BibleFormat::OEB
    } else if content.contains("Chapter") && content.contains("In the beginning") {
        BibleFormat::KJV
    } else {
        BibleFormat::KJV
    }
}

fn version_display_name(code: &str) -> String {
    match code.to_lowercase().as_str() {
        "kjv" => "King James Version".to_string(),
        "asv" => "American Standard Version".to_string(),
        "web" => "World English Bible".to_string(),
        "oeb" => "Open English Bible".to_string(),
        "bsb" => "Berean Standard Bible".to_string(),
        other => other.to_uppercase(),
    }
}
