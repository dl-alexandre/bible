use crate::logger::*;
use crate::models::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Tera, Context as TeraContext};

pub struct HtmlGenerator {
    tera: Tera,
    logger: DiagnosticLogger,
    output_base: PathBuf,
    base_url: String,
}

impl HtmlGenerator {
    pub fn new(template_dir: &Path, output_dir: &Path, logger: DiagnosticLogger, base_url: &str) -> Result<Self> {
        let mut tera = Tera::new(
            template_dir
                .join("*.html")
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid template path"))?,
        )
        .context("Failed to initialize Tera templates")?;

        // Auto-escape for safety
        tera.autoescape_on(vec![".html"]);

        let output_base = output_dir.to_path_buf();

        Ok(HtmlGenerator {
            tera,
            logger,
            output_base,
            base_url: base_url.to_string(),
        })
    }

    pub fn generate_chapter_html(
        &self,
        chapter: &Chapter,
        version_code: &str,
        version_name: &str,
        _crossrefs: Option<&CrossReferenceMap>,
    ) -> Result<PathBuf> {
        let book_dir = self.output_base.join("bible").join(version_code).join(&chapter.book);
        fs::create_dir_all(&book_dir)
            .context("Failed to create book directory")?;

        let output_path = book_dir.join(format!("{}.html", chapter.chapter));

        let mut context = TeraContext::new();
        context.insert("book", &chapter.book);
        context.insert("chapter", &chapter.chapter);
        context.insert("version_code", version_code);
        context.insert("version_name", version_name);
        context.insert("last_updated", &chapter.metadata.last_updated.as_deref().unwrap_or("Unknown"));
        context.insert("manifest_tag", r#"<link rel="manifest" href="/manifest.json">"#);
        context.insert("base_url", &self.base_url);

        let mut verses: Vec<VerseContext> = chapter
            .verses
            .values()
            .map(|v| VerseContext {
                number: v.number.clone(),
                text: v.text.clone(),
                canonical_ref: v.canonical_ref.clone(),
                anchor: v.anchor.clone(),
            })
            .collect();

        verses.sort_by(|a, b| {
            let a_num: u32 = a.number.parse().unwrap_or(0);
            let b_num: u32 = b.number.parse().unwrap_or(0);
            a_num.cmp(&b_num)
        });

        context.insert("verses", &verses);

        let prev_chapter = if chapter.chapter > 1 {
            Some(chapter.chapter - 1)
        } else {
            None
        };
        context.insert("prev_chapter", &prev_chapter);

        context.insert("next_chapter", &Some(chapter.chapter + 1));

        let html = self
            .tera
            .render("chapter.html", &context)
            .context("Failed to render chapter template")?;

        fs::write(&output_path, html)
            .context("Failed to write HTML file")?;

        self.logger.info(format!(
            "Generated HTML: {}",
            output_path.display()
        ));

        Ok(output_path)
    }

    pub fn generate_redirect(
        &self,
        book: &str,
        chapter: u32,
        verse: &str,
        version_code: &str,
        version_name: &str,
        _chapter_path: &Path,
    ) -> Result<PathBuf> {
        let redirect_dir = self
            .output_base
            .join("bible")
            .join(version_code)
            .join(book);

        fs::create_dir_all(&redirect_dir)
            .context("Failed to create redirect directory")?;

        let redirect_path = redirect_dir.join(format!(
            "{}.{}.{}.html",
            book, chapter, verse
        ));

        let target_url = format!(
            "/bible/{}/{}/{}.html#v{}",
            version_code, book, chapter, verse
        );

        let mut context = TeraContext::new();
        context.insert("book", book);
        context.insert("chapter", &chapter);
        context.insert("verse", verse);
        context.insert("version_code", version_code);
        context.insert("version_name", version_name);
        context.insert("target_url", &target_url);

        let html = self
            .tera
            .render("redirect.html", &context)
            .context("Failed to render redirect template")?;

        fs::write(&redirect_path, html)
            .context("Failed to write redirect file")?;

        self.logger.info(format!(
            "Generated redirect: {} -> {}",
            redirect_path.display(),
            target_url
        ));

        Ok(redirect_path)
    }

    pub fn generate_all_redirects(
        &self,
        chapter: &Chapter,
        version_code: &str,
        version_name: &str,
        chapter_path: &Path,
    ) -> Result<Vec<PathBuf>> {
        let mut redirects = Vec::new();

        for verse in chapter.verses.values() {
            let redirect = self.generate_redirect(
                &chapter.book,
                chapter.chapter,
                &verse.number,
                version_code,
                version_name,
                chapter_path,
            )?;
            redirects.push(redirect);
        }

        Ok(redirects)
    }
}

#[derive(serde::Serialize)]
struct VerseContext {
    number: String,
    text: String,
    canonical_ref: String,
    anchor: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapper_tests::create_genesis_1_kjv;
    use tempfile::TempDir;

    #[test]
    fn test_generate_chapter_html() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");
        
        std::fs::create_dir_all(&template_dir).unwrap();
        std::fs::write(
            template_dir.join("chapter.html"),
            r#"<html><body><h1>{{ book }} {{ chapter }}</h1>
            {% for verse in verses %}
            <p id="v{{ verse.number }}" data-verse="{{ verse.canonical_ref }}">
            <span class="verse-number">{{ verse.number }}</span>
            <span class="verse-text">{{ verse.text }}</span>
            </p>
            {% endfor %}
            </body></html>"#,
        ).unwrap();

        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = HtmlGenerator::new(&template_dir, &output_dir, logger, "https://example.com").unwrap();
        let chapter = create_genesis_1_kjv();
        
        let output_path = generator
            .generate_chapter_html(&chapter, "kjv", "King James Version", None)
            .unwrap();

        assert!(output_path.exists());
        
        let html = std::fs::read_to_string(&output_path).unwrap();
        assert!(html.contains("Genesis"));
        assert!(html.contains("id=\"v1\""));
        assert!(html.contains("data-verse=\"Genesis.1.1\""));
        assert!(html.contains("In the beginning"));
    }

    #[test]
    fn test_anchor_stability() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");
        
        std::fs::create_dir_all(&template_dir).unwrap();
        std::fs::write(
            template_dir.join("chapter.html"),
            r#"<html><body>
            {% for verse in verses %}
            <p id="v{{ verse.number }}">{{ verse.number }}</p>
            {% endfor %}
            </body></html>"#,
        ).unwrap();

        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = HtmlGenerator::new(&template_dir, &output_dir, logger, "https://example.com").unwrap();
        let chapter = create_genesis_1_kjv();
        
        let output_path = generator
            .generate_chapter_html(&chapter, "kjv", "KJV", None)
            .unwrap();

        let html = std::fs::read_to_string(&output_path).unwrap();
        
        assert!(html.contains("id=\"v1\""));
        assert!(html.contains("id=\"v2\""));
        assert!(html.contains("id=\"v3\""));
    }

    #[test]
    fn test_data_verse_attributes() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");
        
        std::fs::create_dir_all(&template_dir).unwrap();
        std::fs::write(
            template_dir.join("chapter.html"),
            r#"<html><body>
            {% for verse in verses %}
            <p data-verse="{{ verse.canonical_ref }}">{{ verse.number }}</p>
            {% endfor %}
            </body></html>"#,
        ).unwrap();

        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = HtmlGenerator::new(&template_dir, &output_dir, logger, "https://example.com").unwrap();
        let chapter = create_genesis_1_kjv();
        
        let output_path = generator
            .generate_chapter_html(&chapter, "kjv", "KJV", None)
            .unwrap();

        let html = std::fs::read_to_string(&output_path).unwrap();
        
        assert!(html.contains("data-verse=\"Genesis.1.1\""));
        assert!(html.contains("data-verse=\"Genesis.1.2\""));
        assert!(html.contains("data-verse=\"Genesis.1.3\""));
    }

    #[test]
    fn test_generate_redirect() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");
        
        std::fs::create_dir_all(&template_dir).unwrap();
        std::fs::write(
            template_dir.join("redirect.html"),
            r#"<html><head>
            <meta http-equiv="refresh" content="0;url={{ target_url }}">
            </head><body></body></html>"#,
        ).unwrap();

        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = HtmlGenerator::new(&template_dir, &output_dir, logger, "https://example.com").unwrap();
        
        let redirect_path = generator
            .generate_redirect("Genesis", 1, "1", "kjv", "KJV", temp_dir.path())
            .unwrap();

        assert!(redirect_path.exists());
        
        let html = std::fs::read_to_string(&redirect_path).unwrap();
        assert!(html.contains("http-equiv=\"refresh\""));
        assert!(html.contains("/bible/kjv/Genesis/1.html#v1"));
    }

    #[test]
    fn test_navigation_structure() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");
        
        std::fs::create_dir_all(&template_dir).unwrap();
        std::fs::write(
            template_dir.join("chapter.html"),
            r#"<html><body>
            <nav aria-label="Chapter navigation">
            {% if prev_chapter %}
            <a href="/bible/{{ version_code }}/{{ book }}/{{ prev_chapter }}" rel="prev">← {{ prev_chapter }}</a>
            {% endif %}
            {% if next_chapter %}
            <a href="/bible/{{ version_code }}/{{ book }}/{{ next_chapter }}" rel="next">{{ next_chapter }} →</a>
            {% endif %}
            </nav>
            </body></html>"#,
        ).unwrap();

        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = HtmlGenerator::new(&template_dir, &output_dir, logger, "https://example.com").unwrap();
        let chapter = create_genesis_1_kjv();
        
        let output_path = generator
            .generate_chapter_html(&chapter, "kjv", "KJV", None)
            .unwrap();

        let html = std::fs::read_to_string(&output_path).unwrap();
        
        assert!(html.contains("aria-label=\"Chapter navigation\""));
        assert!(html.contains("rel=\"prev\""));
        assert!(html.contains("rel=\"next\""));
    }

    #[test]
    fn test_generate_all_redirects() {
        let temp_dir = TempDir::new().unwrap();
        let template_dir = temp_dir.path().join("templates");
        let output_dir = temp_dir.path().join("output");
        
        std::fs::create_dir_all(&template_dir).unwrap();
        std::fs::write(
            template_dir.join("redirect.html"),
            r#"<html><head><meta http-equiv="refresh" content="0;url={{ target_url }}"></head></html>"#,
        ).unwrap();

        let log_dir = temp_dir.path().join("logs");
        let logger = DiagnosticLogger::new(&log_dir).unwrap();
        
        let generator = HtmlGenerator::new(&template_dir, &output_dir, logger, "https://example.com").unwrap();
        let chapter = create_genesis_1_kjv();
        
        let chapter_path = generator
            .generate_chapter_html(&chapter, "kjv", "KJV", None)
            .unwrap();

        let redirects = generator
            .generate_all_redirects(&chapter, "kjv", "KJV", &chapter_path)
            .unwrap();

        assert_eq!(redirects.len(), 3);
        
        for redirect in redirects {
            assert!(redirect.exists());
            let html = std::fs::read_to_string(&redirect).unwrap();
            assert!(html.contains("http-equiv=\"refresh\""));
        }
    }
}

