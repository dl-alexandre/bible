use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Source text representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceText {
    pub version: String,
    pub books: Vec<BookData>,
    pub metadata: SourceMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetadata {
    pub description: Option<String>,
    pub language: Option<String>,
}

/// Book data from source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookData {
    pub name: String,
    pub abbreviation: String,
    pub chapters: Vec<ChapterData>,
}

/// Chapter data from source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterData {
    pub number: u32,
    pub verses: Vec<VerseData>,
}

/// Verse data from source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerseData {
    pub number: String, // Support for verse ranges like "1-2"
    pub text: String,
    pub footnotes: Option<Vec<String>>,
}

/// Generated output structures

/// Verse in output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verse {
    pub id: String,
    pub number: String,
    pub text: String,
    pub anchor: String, // #v{number}
    pub canonical_ref: String, // book.chapter.verse
}

/// Bible version metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BibleVersion {
    pub code: String, // kjv, asv, web, oeb
    pub name: String,
    pub description: String,
    pub books: Vec<BookReference>,
    pub metadata: VersionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct VersionMetadata {
    pub language: String,
    pub year: Option<u32>,
}

/// Book reference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BookReference {
    pub name: String,
    pub abbreviation: String,
    pub chapters: u32,
    pub testament: Testament,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum Testament {
    Old,
    New,
}

/// Chapter structure for processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub book: String,
    pub chapter: u32,
    pub verses: std::collections::HashMap<String, Verse>,
    pub metadata: ChapterMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ChapterMetadata {
    pub verse_count: u32,
    pub last_updated: Option<String>, // ISO 8601 formatted timestamp
}

/// JSON output structures

/// Chapter JSON output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ChapterJson {
    pub schema_version: String,
    pub book: String,
    pub chapter: u32,
    pub version: String,
    pub verses: std::collections::HashMap<String, String>,
    pub metadata: ChapterMetadata,
    pub extensions: serde_json::Value, // Future extensibility
}

/// Global manifest
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct GlobalManifest {
    pub schema_version: String,
    pub build_timestamp: String,
    pub source_checksums: std::collections::HashMap<String, String>,
    pub available_versions: Vec<String>,
    pub api_endpoints: ApiEndpoints,
    pub schema_locations: std::collections::HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapper_thresholds: Option<MapperThresholds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versification: Option<std::collections::HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossrefs_sha256: Option<String>,
    pub extensions: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct MapperThresholds {
    pub jaccard: f64,
    pub levenshtein: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ApiEndpoints {
    pub versions: String,
    pub books: String,
    pub crossrefs: String,
    pub chapters: String, // Template: /{version}/{book}/{chapter}.json
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct VersionEntry {
    pub code: String,
    pub name: String,
    pub book_count: usize,
    pub chapter_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct VersionsJson {
    pub schema_version: String,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct BookEntry {
    pub name: String,
    pub chapter_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct BooksJson {
    pub schema_version: String,
    pub books: Vec<BookEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictType {
    Split,
    Merge,
    Shift,
    Absent,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CrossReferenceMap {
    pub schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versification: Option<std::collections::HashMap<String, Vec<String>>>,
    pub mappings: std::collections::HashMap<String, std::collections::HashMap<String, MappingEntry>>,
    pub conflicts: Vec<EnhancedMappingConflict>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MappingMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum MappingEntry {
    Ref {
        #[serde(rename = "ref")]
        ref_: String,
    },
    Null {
        #[serde(rename = "ref")]
        ref_: Option<String>,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct VerseReference {
    pub book: String,
    pub chapter: u32,
    pub verse: String,
}

impl VerseReference {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.book, self.chapter, self.verse)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct EnhancedMappingConflict {
    pub canonical: String,
    pub version: String,
    #[serde(rename = "type")]
    pub conflict_type: ConflictType,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[allow(dead_code)]
pub struct ConflictDetail {
    pub version: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[allow(dead_code)]
pub struct NullEntry {
    pub version: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct MappingMetrics {
    pub total: usize,
    pub mapped: usize,
    pub nulls: usize,
    pub conflicts: usize,
    pub coverage: f64,
    pub similarity_thresholds: SimilarityThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SimilarityThresholds {
    pub jaccard: f64,
    pub levenshtein: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_chapter_json_serialization() {
        let chapter = ChapterJson {
            schema_version: "1.0".to_string(),
            book: "Genesis".to_string(),
            chapter: 1,
            version: "kjv".to_string(),
            verses: std::collections::HashMap::from([
                ("1".to_string(), "In the beginning God created the heaven and the earth.".to_string()),
                ("2".to_string(), "And the earth was without form, and void; and darkness was upon the face of the deep. And the Spirit of God moved upon the face of the waters.".to_string()),
            ]),
            metadata: ChapterMetadata {
                verse_count: 2,
                last_updated: Some("2023-10-01T00:00:00Z".to_string()),
            },
            extensions: serde_json::Value::Null,
        };

        let json = serde_json::to_string(&chapter).unwrap();
        let deserialized: ChapterJson = serde_json::from_str(&json).unwrap();

        assert_eq!(chapter, deserialized);
    }

    #[test]
    fn test_global_manifest_serialization() {
        let manifest = GlobalManifest {
            schema_version: "1.0".to_string(),
            build_timestamp: "2023-10-01T00:00:00Z".to_string(),
            source_checksums: std::collections::HashMap::from([
                ("kjv".to_string(), "abc123".to_string()),
            ]),
            available_versions: vec!["kjv".to_string()],
            api_endpoints: ApiEndpoints {
                versions: "/versions.json".to_string(),
                books: "/books.json".to_string(),
                crossrefs: "/crossrefs.json".to_string(),
                chapters: "/{version}/{book}/{chapter}.json".to_string(),
            },
            schema_locations: std::collections::HashMap::from([
                ("chapter".to_string(), "/schema/chapter-1.0.json".to_string()),
            ]),
            mapper_thresholds: None,
            versification: None,
            crossrefs_sha256: None,
            extensions: serde_json::Value::Null,
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: GlobalManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(manifest, deserialized);
    }

    #[test]
    fn test_cross_reference_map_serialization() {
        let crossrefs = CrossReferenceMap {
            schema_version: "1.0".to_string(),
            versification: None,
            mappings: std::collections::HashMap::from([
                ("Genesis.1.1".to_string(), std::collections::HashMap::from([
                    ("kjv".to_string(), MappingEntry::Ref {
                        ref_: "Genesis.1.1".to_string(),
                    }),
                ])),
            ]),
            conflicts: vec![],
            metrics: Some(MappingMetrics {
                total: 1,
                mapped: 1,
                nulls: 0,
                conflicts: 0,
                coverage: 1.0,
                similarity_thresholds: SimilarityThresholds {
                    jaccard: 0.70,
                    levenshtein: 0.15,
                },
            }),
        };

        let json = serde_json::to_string(&crossrefs).unwrap();
        let deserialized: CrossReferenceMap = serde_json::from_str(&json).unwrap();

        assert_eq!(crossrefs, deserialized);
    }

    // TODO: Add schema validation tests once validate_json is implemented
}
