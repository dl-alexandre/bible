use crate::logger::*;
use crate::models::*;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde_json::json;

pub struct SiteGenerator {
    output_base: PathBuf,
    logger: DiagnosticLogger,
}

impl SiteGenerator {
    pub fn new(output_dir: &Path, logger: DiagnosticLogger) -> Result<Self> {
        Ok(SiteGenerator {
            output_base: output_dir.to_path_buf(),
            logger,
        })
    }

    pub fn generate_index(
        &self,
        versions: &HashMap<String, HashMap<String, Chapter>>,
        base_url: &str,
    ) -> Result<PathBuf> {
        let output_path = self.output_base.join("index.html");

        let mut version_list: Vec<(&String, usize)> = versions
            .iter()
            .map(|(code, chapters)| (code, chapters.len()))
            .collect();
        version_list.sort_by_key(|(code, _)| code.to_string());

        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str("  <title>Bible Static Site - Available Versions</title>\n");
        html.push_str("  <meta name=\"description\" content=\"Read the Bible online with multiple translations: KJV, ASV, and WEB. Browse by book, chapter, and verse.\">\n");
        html.push_str(&format!("  <meta property=\"og:title\" content=\"Bible Static Site - Available Versions\">\n"));
        html.push_str("  <meta property=\"og:description\" content=\"Read the Bible online with multiple translations: KJV, ASV, and WEB. Browse by book, chapter, and verse.\">\n");
        html.push_str("  <meta property=\"og:type\" content=\"website\">\n");
        html.push_str(&format!("  <meta property=\"og:url\" content=\"{}\">\n", base_url));
        html.push_str(&format!("  <meta property=\"og:image\" content=\"{}static/og-image.png\">\n", base_url));
        html.push_str("  <meta name=\"twitter:card\" content=\"summary_large_image\">\n");
        html.push_str("  <meta name=\"twitter:title\" content=\"Bible Static Site - Available Versions\">\n");
        html.push_str("  <meta name=\"twitter:description\" content=\"Read the Bible online with multiple translations: KJV, ASV, and WEB. Browse by book, chapter, and verse.\">\n");
        html.push_str(&format!("  <meta name=\"twitter:image\" content=\"{}static/og-image.png\">\n", base_url));
        html.push_str(&format!("  <link rel=\"manifest\" href=\"{}static/app.webmanifest\">\n", base_url));
        html.push_str(&format!("  <link rel=\"icon\" href=\"{}static/favicon.ico\" type=\"image/x-icon\">\n", base_url));
        html.push_str("</head>\n");
        html.push_str("<body>\n");
        html.push_str("  <header>\n");
        html.push_str("    <h1>Bible Versions</h1>\n");
        html.push_str("  </header>\n");
        html.push_str("  <nav aria-label=\"Breadcrumb\">\n");
        html.push_str("    <ol>\n");
        html.push_str(&format!("      <li><a href=\"{}\">Home</a></li>\n", base_url));
        html.push_str("    </ol>\n");
        html.push_str("  </nav>\n");
        html.push_str("  <main>\n");
        html.push_str("    <section class=\"search-panel\" aria-labelledby=\"search-title\">\n");
        html.push_str("      <h2 id=\"search-title\">Search the Bible</h2>\n");
        html.push_str("      <label for=\"search-input\">Search phrase, book, or translation</label>\n");
        html.push_str("      <input id=\"search-input\" type=\"search\" autocomplete=\"off\" placeholder=\"Try: beginning, Psalms, KJV\">\n");
        html.push_str("      <label for=\"search-version\">Translation</label>\n");
        html.push_str("      <select id=\"search-version\"><option value=\"all\">All translations</option>");
        for (version_code, _) in &version_list {
            html.push_str(&format!("<option value=\"{}\">{}</option>", version_code, version_code.to_uppercase()));
        }
        html.push_str("</select>\n");
        html.push_str("      <p id=\"search-status\" class=\"search-status\" role=\"status\"></p>\n");
        html.push_str("      <ol id=\"search-results\" class=\"search-results\"></ol>\n");
        html.push_str("    </section>\n");
        html.push_str("    <section>\n");
        html.push_str("      <h2>Available Versions</h2>\n");
        html.push_str("      <ul>\n");

        for (version_code, chapter_count) in &version_list {
            let version_name = version_code.to_uppercase();
            html.push_str(&format!(
                "        <li><a href=\"{}{}/\">{}</a> ({} chapters)</li>\n",
                base_url, version_code, version_name, chapter_count
            ));
        }

        html.push_str("      </ul>\n");
        html.push_str("    </section>\n");

        html.push_str("    <section>\n");
        html.push_str("      <h2>Books</h2>\n");
        html.push_str("      <ul>\n");

        let mut book_owner: HashMap<String, String> = HashMap::new();
        for (version_code, chapters) in versions {
            for chapter_key in chapters.keys() {
                if let Some(book_name) = chapter_key.split('.').next() {
                    book_owner.entry(book_name.to_string()).or_insert_with(|| version_code.clone());
                }
            }
        }

        let mut sorted_books: Vec<String> = book_owner.keys().cloned().collect();
        sorted_books.sort();

        for book in sorted_books {
            let version_code = book_owner.get(&book).cloned().unwrap_or_default();
            if !version_code.is_empty() {
                let book_url = format!("{}{}/{}/1.html", base_url, version_code, book);
                html.push_str(&format!("        <li><a href=\"{}\">{}</a></li>\n", book_url, book));
            }
        }

        html.push_str("      </ul>\n");
        html.push_str("    </section>\n");
        html.push_str("  </main>\n");
        html.push_str("  <footer>\n");
        html.push_str(&format!("    <p><a href=\"{}manifest.json\">Manifest</a></p>\n", base_url));
        html.push_str(&format!("    <p><a href=\"{}provenance.html\">Data provenance</a></p>\n", base_url));
        html.push_str(&format!("    <p><a href=\"{}static/study.html\">My Study</a></p>\n", base_url));
        html.push_str("  </footer>\n");
        html.push_str(&format!("  <script src=\"{}static/search.js\"></script>\n", base_url));
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        fs::write(&output_path, &html)
            .context("Failed to write index.html")?;

        self.logger.info(format!(
            "Generated index.html ({} bytes)",
            html.len()
        ));

        Ok(output_path)
    }

    pub fn generate_search_index(
        &self,
        versions: &HashMap<String, HashMap<String, Chapter>>,
        base_url: &str,
    ) -> Result<()> {
        let mut version_codes: Vec<&String> = versions.keys().collect();
        version_codes.sort();
        for version in version_codes {
            let mut entries = Vec::new();
            let mut chapters: Vec<&Chapter> = versions[version].values().collect();
            chapters.sort_by(|a, b| a.book.cmp(&b.book).then(a.chapter.cmp(&b.chapter)));
            for chapter in chapters {
                let mut verses: Vec<&Verse> = chapter.verses.values().collect();
                verses.sort_by_key(|verse| verse.number.parse::<u32>().unwrap_or(0));
                let text = verses.iter().map(|verse| verse.text.as_str()).collect::<Vec<_>>().join(" ");
                entries.push(json!({
                    "v": version,
                    "b": chapter.book,
                    "c": chapter.chapter,
                    "t": text,
                    "u": format!("{}{}/{}/{}.html", base_url, version, chapter.book, chapter.chapter),
                }));
            }

            let output_path = self.output_base.join(format!("search-index-{}.json", version));
            fs::write(&output_path, serde_json::to_string(&entries)?)
                .context("Failed to write search index")?;
            self.logger.info(format!("Generated search index {} ({} entries)", version, entries.len()));
        }
        Ok(())
    }

    pub fn generate_provenance(&self, source_files: &[PathBuf], base_url: &str) -> Result<PathBuf> {
        let output_path = self.output_base.join("provenance.html");
        let checksums = crate::manifest_generator::ManifestGenerator::compute_source_checksums(source_files)?;
        let mut html = String::from("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>Data Provenance - Bible</title>");
        html.push_str(&format!("<link rel=\"stylesheet\" href=\"{}static/styles.css\"></head><body><a class=\"skip-link\" href=\"#main-content\">Skip to content</a><main id=\"main-content\" class=\"provenance\"><p><a href=\"{}\">Home</a></p><h1>Data provenance</h1><p>Dataset checksums for this build are recorded below. Source texts remain subject to their respective licenses.</p><p>See <code>DATA_SOURCES.md</code> in the repository for source and licensing notes.</p><ul>", base_url, base_url));
        let mut entries: Vec<_> = checksums.iter().collect();
        entries.sort_by_key(|(path, _)| *path);
        for (path, checksum) in entries {
            html.push_str(&format!("<li><code>{}</code><br><small>SHA-256: {}</small></li>", path, checksum));
        }
        html.push_str("</ul></main></body></html>");
        fs::write(&output_path, html).context("Failed to write provenance page")?;
        Ok(output_path)
    }

    pub fn ensure_deterministic_structure(&self) -> Result<()> {
        self.sort_directories_recursive(&self.output_base)?;
        Ok(())
    }

    fn sort_directories_recursive(&self, dir: &Path) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
            .map(|e| e.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()?;

        entries.sort();

        for entry in entries {
            if entry.is_dir() {
                self.sort_directories_recursive(&entry)?;
            }
        }

        Ok(())
    }

    pub fn generate_sitemap(
        &self,
        versions: &HashMap<String, HashMap<String, Chapter>>,
        base_url: &str,
    ) -> Result<PathBuf> {
        let output_path = self.output_base.join("sitemap.xml");

        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}</loc>\n", base_url));
        xml.push_str("    <changefreq>weekly</changefreq>\n");
        xml.push_str("    <priority>1.0</priority>\n");
        xml.push_str("  </url>\n");

        let mut version_list: Vec<&String> = versions.keys().collect();
        version_list.sort();

        for version_code in version_list {
            let version_url = format!("{}{}/", base_url, version_code);
            xml.push_str("  <url>\n");
            xml.push_str(&format!("    <loc>{}</loc>\n", version_url));
            xml.push_str("    <changefreq>weekly</changefreq>\n");
            xml.push_str("    <priority>0.8</priority>\n");
            xml.push_str("  </url>\n");

            let chapters = &versions[version_code];
            let mut chapter_keys: Vec<&String> = chapters.keys().collect();
            chapter_keys.sort();

            for chapter_key in chapter_keys {
                let parts: Vec<&str> = chapter_key.split('.').collect();
                if parts.len() >= 2 {
                    let book = parts[0];
                    let chapter = parts[1];
                    let chapter_url = format!("{}{}/{}/{}.html", base_url, version_code, book, chapter);
                    xml.push_str("  <url>\n");
                    xml.push_str(&format!("    <loc>{}</loc>\n", chapter_url));
                    xml.push_str("    <changefreq>monthly</changefreq>\n");
                    xml.push_str("    <priority>0.6</priority>\n");
                    xml.push_str("  </url>\n");
                }
            }
        }

        xml.push_str("</urlset>\n");

        fs::write(&output_path, &xml)
            .context("Failed to write sitemap.xml")?;

        self.logger.info(format!("Generated sitemap.xml ({} bytes)", xml.len()));

        Ok(output_path)
    }

    pub fn generate_robots_txt(&self) -> Result<PathBuf> {
        let output_path = self.output_base.join("robots.txt");

        let content = "User-agent: *\nAllow: /\n\nSitemap: /bible/sitemap.xml\n";

        fs::write(&output_path, content)
            .context("Failed to write robots.txt")?;

        self.logger.info("Generated robots.txt".to_string());

        Ok(output_path)
    }
}
