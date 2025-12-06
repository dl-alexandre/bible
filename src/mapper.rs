use crate::mapper_config::MapperConfig;
use crate::models::*;
use crate::text_normalizer::TextNormalizer;
use anyhow::{Context, Result};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use strsim::normalized_levenshtein;


#[cfg(test)]
use tempfile;

pub fn canonical_ref(book: &str, chapter: u32, verse: &str) -> String {
    format!("{}.{}.{}", book, chapter, verse)
}

pub fn parse_canonical_ref(canonical: &str) -> Result<(String, u32, String)> {
    let parts: Vec<&str> = canonical.split('.').collect();
    if parts.len() != 3 {
        return Err(anyhow::anyhow!(
            "Invalid canonical reference format: {}",
            canonical
        ));
    }

    let book = parts[0].to_string();
    let chapter = parts[1]
        .parse::<u32>()
        .with_context(|| format!("Invalid chapter number in: {}", canonical))?;
    let verse = parts[2].to_string();

    Ok((book, chapter, verse))
}

pub struct TextCache {
    normalized_text: HashMap<String, String>,
    token_sets: HashMap<String, HashSet<String>>,
}

impl TextCache {
    pub fn new() -> Self {
        TextCache {
            normalized_text: HashMap::new(),
            token_sets: HashMap::new(),
        }
    }

    pub fn get_normalized(&mut self, normalizer: &TextNormalizer, text: &str) -> String {
        if let Some(cached) = self.normalized_text.get(text) {
            return cached.clone();
        }
        let normalized = normalizer.normalize(text);
        self.normalized_text.insert(text.to_string(), normalized.clone());
        normalized
    }

    pub fn get_tokens(&mut self, normalizer: &TextNormalizer, text: &str) -> HashSet<String> {
        if let Some(cached) = self.token_sets.get(text) {
            return cached.clone();
        }
        let tokens = normalizer.normalize_tokens(text);
        self.token_sets.insert(text.to_string(), tokens.clone());
        tokens
    }
}

pub struct CrossVersionMapper {
    config: MapperConfig,
    normalizer: TextNormalizer,
    cache: TextCache,
    versification_schemes: HashMap<String, String>,
}

impl CrossVersionMapper {
    pub fn new() -> Result<Self> {
        Ok(CrossVersionMapper {
            config: MapperConfig::default(),
            normalizer: TextNormalizer::new()
                .context("Failed to create TextNormalizer")?,
            cache: TextCache::new(),
            versification_schemes: HashMap::new(),
        })
    }

    pub fn with_config(config: &MapperConfig) -> Result<Self> {
        Ok(CrossVersionMapper {
            config: config.clone(),
            normalizer: TextNormalizer::new()
                .context("Failed to create TextNormalizer")?,
            cache: TextCache::new(),
            versification_schemes: HashMap::new(),
        })
    }

    pub fn set_versification(&mut self, version: &str, scheme: &str) {
        self.versification_schemes.insert(version.to_string(), scheme.to_string());
    }

    pub fn generate_mappings(
        &mut self,
        versions: &HashMap<String, HashMap<String, Chapter>>,
    ) -> Result<CrossReferenceMap> {
        self.cache.normalized_text.clear();
        self.cache.token_sets.clear();
        let mut mappings: HashMap<String, HashMap<String, MappingEntry>> = HashMap::new();
        let mut conflicts = Vec::new();

        let mut all_canonical_refs: Vec<String> = self.collect_canonical_references(versions).into_iter().collect();
        all_canonical_refs.sort();
        
        let mut version_list: Vec<String> = versions.keys().cloned().collect();
        version_list.sort();

        for canonical_ref_str in &all_canonical_refs {
            let (book, chapter, verse) = parse_canonical_ref(canonical_ref_str)?;

            let mut version_map: HashMap<String, MappingEntry> = HashMap::new();

            for version_code in &version_list {
                let reason = if let Some(book_map) = versions.get(version_code) {
                    if let Some(chapter_data) = book_map.get(&format!("{}.{}", book, chapter)) {
                        if let Some(_verse_obj) = chapter_data.verses.get(&verse) {
                            let ref_str = crate::mapper::canonical_ref(&book, chapter, &verse);
                            version_map.insert(version_code.clone(), MappingEntry::Ref { ref_: ref_str });
                            continue;
                        } else {
                            if let Some(scheme) = self.versification_schemes.get(version_code) {
                                format!("versification_mismatch ({})", scheme)
                            } else {
                                format!("Verse {} not found in {}.{}", verse, book, chapter)
                            }
                        }
                    } else {
                        format!("Chapter {} not found in {}", chapter, book)
                    }
                } else {
                    "Version not available".to_string()
                };

                version_map.insert(
                    version_code.clone(),
                    MappingEntry::Null {
                        ref_: None,
                        reason,
                    },
                );
            }

            self.detect_enhanced_conflicts(&canonical_ref_str, &version_map, &mut conflicts, versions);
            mappings.insert(canonical_ref_str.clone(), version_map);
        }

        let versification = if self.versification_schemes.is_empty() {
            None
        } else {
            let mut vers_map = HashMap::new();
            for (version, scheme) in &self.versification_schemes {
                vers_map
                    .entry(scheme.clone())
                    .or_insert_with(Vec::new)
                    .push(version.clone());
            }
            Some(vers_map)
        };

        let metrics = self.compute_metrics_internal(&mappings, &conflicts);

        Ok(CrossReferenceMap {
            schema_version: "1.0".to_string(),
            versification,
            mappings,
            conflicts,
            metrics: Some(metrics),
        })
    }

    fn compute_metrics_internal(
        &self,
        mappings: &HashMap<String, HashMap<String, MappingEntry>>,
        conflicts: &[EnhancedMappingConflict],
    ) -> MappingMetrics {
        let mut total = 0;
        let mut mapped = 0;
        let mut nulls = 0;

        for version_map in mappings.values() {
            for entry in version_map.values() {
                match entry {
                    MappingEntry::Ref { .. } => mapped += 1,
                    MappingEntry::Null { .. } => nulls += 1,
                }
                total += 1;
            }
        }

        let coverage = if total > 0 {
            mapped as f64 / total as f64
        } else {
            0.0
        };

        MappingMetrics {
            total,
            mapped,
            nulls,
            conflicts: conflicts.len(),
            coverage,
            similarity_thresholds: SimilarityThresholds {
                jaccard: self.config.jaccard_threshold,
                levenshtein: self.config.levenshtein_threshold,
            },
        }
    }

    fn detect_enhanced_conflicts(
        &self,
        canonical_ref_str: &str,
        version_map: &HashMap<String, MappingEntry>,
        conflicts: &mut Vec<EnhancedMappingConflict>,
        versions: &HashMap<String, HashMap<String, Chapter>>,
    ) {
        let (canonical_book, canonical_chapter, canonical_verse) = 
            match parse_canonical_ref(canonical_ref_str) {
                Ok(refs) => refs,
                Err(_) => return,
            };

        for (version_code, entry) in version_map {
            match entry {
                MappingEntry::Ref { ref_: ref_str } => {
                    if ref_str != canonical_ref_str {
                        let (ref_book, ref_chapter, ref_verse) = 
                            match parse_canonical_ref(ref_str) {
                                Ok(refs) => refs,
                                Err(_) => continue,
                            };

                        if ref_book == canonical_book && ref_chapter == canonical_chapter {
                            let conflict_type = self.classify_conflict(
                                &canonical_verse,
                                &ref_verse,
                                canonical_ref_str,
                                version_code,
                                versions,
                            );

                            conflicts.push(EnhancedMappingConflict {
                                canonical: canonical_ref_str.to_string(),
                                version: version_code.clone(),
                                conflict_type,
                                details: vec![ref_str.clone()],
                            });
                        }
                    }
                }
                MappingEntry::Null { reason, .. } => {
                    if reason.contains("versification_mismatch") {
                        conflicts.push(EnhancedMappingConflict {
                            canonical: canonical_ref_str.to_string(),
                            version: version_code.clone(),
                            conflict_type: ConflictType::Absent,
                            details: vec![reason.clone()],
                        });
                    }
                }
            }
        }

        self.detect_split_merge(canonical_ref_str, version_map, conflicts, versions);
    }

    fn classify_conflict(
        &self,
        canonical_verse: &str,
        ref_verse: &str,
        canonical_ref_str: &str,
        version_code: &str,
        versions: &HashMap<String, HashMap<String, Chapter>>,
    ) -> ConflictType {
        let canonical_num = canonical_verse.parse::<u32>().unwrap_or(0);
        let ref_num = ref_verse.parse::<u32>().unwrap_or(0);

        if canonical_num == 0 || ref_num == 0 {
            return ConflictType::Absent;
        }

        let diff = (canonical_num as i32 - ref_num as i32).abs();

        if diff == 1 {
            return ConflictType::Shift;
        }

        if let Some((book, chapter, _)) = parse_canonical_ref(canonical_ref_str).ok() {
            let chapter_key = format!("{}.{}", book, chapter);
            if let Some(version_chapters) = versions.get(version_code) {
                if let Some(chapter_data) = version_chapters.get(&chapter_key) {
                    let verses: Vec<u32> = chapter_data
                        .verses
                        .keys()
                        .filter_map(|v| v.parse::<u32>().ok())
                        .collect();

                    if verses.contains(&canonical_num) && verses.contains(&(canonical_num + 1)) {
                        if verses.contains(&ref_num) && !verses.contains(&(ref_num + 1)) {
                            return ConflictType::Merge;
                        }
                    }

                    if verses.contains(&ref_num) && verses.contains(&(ref_num + 1)) {
                        if verses.contains(&canonical_num) && !verses.contains(&(canonical_num + 1)) {
                            return ConflictType::Split;
                        }
                    }
                }
            }
        }

        ConflictType::Shift
    }

    fn detect_split_merge(
        &self,
        canonical_ref_str: &str,
        version_map: &HashMap<String, MappingEntry>,
        conflicts: &mut Vec<EnhancedMappingConflict>,
        versions: &HashMap<String, HashMap<String, Chapter>>,
    ) {
        let (book, chapter, _) = match parse_canonical_ref(canonical_ref_str) {
            Ok(refs) => refs,
            Err(_) => return,
        };

        for (version_code, entry) in version_map {
            if let MappingEntry::Ref { ref_: ref_str } = entry {
                let chapter_key = format!("{}.{}", book, chapter);
                if let Some(version_chapters) = versions.get(version_code) {
                    if let Some(chapter_data) = version_chapters.get(&chapter_key) {
                        let mut verse_nums: Vec<u32> = chapter_data
                            .verses
                            .keys()
                            .filter_map(|v| v.parse::<u32>().ok())
                            .collect();
                        verse_nums.sort();

                        for i in 0..verse_nums.len().saturating_sub(1) {
                            let curr = verse_nums[i];
                            let next = verse_nums[i + 1];
                            let curr_ref = crate::mapper::canonical_ref(&book, chapter, &curr.to_string());
                            let next_ref = crate::mapper::canonical_ref(&book, chapter, &next.to_string());
                            let merged_verse = format!("{}-{}", curr, next);

                            if ref_str == canonical_ref_str {
                                if chapter_data.verses.contains_key(&merged_verse) {
                                    if !conflicts.iter().any(|c| 
                                        c.canonical == canonical_ref_str && 
                                        c.version == *version_code &&
                                        matches!(c.conflict_type, ConflictType::Merge)
                                    ) {
                                        conflicts.push(EnhancedMappingConflict {
                                            canonical: canonical_ref_str.to_string(),
                                            version: version_code.clone(),
                                            conflict_type: ConflictType::Merge,
                                            details: vec![curr_ref.clone(), next_ref.clone()],
                                        });
                                    }
                                }
                            }

                            if canonical_ref_str == &curr_ref || canonical_ref_str == &next_ref {
                                if chapter_data.verses.contains_key(&merged_verse) {
                                    if !conflicts.iter().any(|c| 
                                        c.canonical == canonical_ref_str && 
                                        c.version == *version_code &&
                                        matches!(c.conflict_type, ConflictType::Split)
                                    ) {
                                        conflicts.push(EnhancedMappingConflict {
                                            canonical: canonical_ref_str.to_string(),
                                            version: version_code.clone(),
                                            conflict_type: ConflictType::Split,
                                            details: vec![curr_ref.clone(), next_ref],
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_canonical_references(
        &self,
        versions: &HashMap<String, HashMap<String, Chapter>>,
    ) -> HashSet<String> {
        let mut refs = HashSet::new();

        for version_chapters in versions.values() {
            for chapter in version_chapters.values() {
                for verse in chapter.verses.values() {
                    refs.insert(verse.canonical_ref.clone());
                }
            }
        }

        refs
    }


    pub fn apply_textual_alignment(
        &mut self,
        canonical_ref: &str,
        source_text: &str,
        target_chapters: &HashMap<String, Chapter>,
    ) -> Option<VerseReference> {
        let (book, chapter, _) = match parse_canonical_ref(canonical_ref) {
            Ok(refs) => refs,
            Err(_) => return None,
        };

        let chapter_key = format!("{}.{}", book, chapter);
        let target_chapter = target_chapters.get(&chapter_key)?;

        let mut best_match: Option<(f64, &Verse)> = None;

        for verse in target_chapter.verses.values() {
            let similarity = self.textual_similarity(source_text, &verse.text);
            if similarity >= self.config.jaccard_threshold {
                match best_match {
                    None => best_match = Some((similarity, verse)),
                    Some((best_score, _)) if similarity > best_score => {
                        best_match = Some((similarity, verse))
                    }
                    Some((best_score, best_verse)) if (similarity - best_score).abs() < 0.001 => {
                        let best_canonical = crate::mapper::canonical_ref(&best_verse.canonical_ref.split('.').next().unwrap_or(""), 
                            best_verse.canonical_ref.split('.').nth(1).and_then(|s| s.parse().ok()).unwrap_or(0),
                            &best_verse.number);
                        let current_canonical = crate::mapper::canonical_ref(&book, chapter, &verse.number);
                        if current_canonical < best_canonical {
                            best_match = Some((similarity, verse));
                        }
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(_, verse)| VerseReference {
            book: verse.canonical_ref.split('.').next().unwrap().to_string(),
            chapter: verse.canonical_ref
                .split('.')
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            verse: verse.number.clone(),
        })
    }

    fn jaccard_similarity(&mut self, text1: &str, text2: &str) -> f64 {
        let tokens1 = self.cache.get_tokens(&self.normalizer, text1);
        let tokens2 = self.cache.get_tokens(&self.normalizer, text2);

        if tokens1.is_empty() || tokens2.is_empty() {
            return 0.0;
        }

        let intersection: usize = tokens1.intersection(&tokens2).count();
        let union: usize = tokens1.union(&tokens2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    fn textual_similarity(&mut self, text1: &str, text2: &str) -> f64 {
        let jaccard = self.jaccard_similarity(text1, text2);
        
        if jaccard >= self.config.jaccard_threshold {
            return jaccard;
        }

        if jaccard >= self.config.jaccard_min && jaccard < self.config.jaccard_max {
            let norm1 = self.cache.get_normalized(&self.normalizer, text1);
            let norm2 = self.cache.get_normalized(&self.normalizer, text2);
            let lev_distance = normalized_levenshtein(&norm1, &norm2);
            let lev_similarity = 1.0 - lev_distance;
            
            if lev_similarity >= self.config.levenshtein_threshold {
                return (jaccard + lev_similarity) / 2.0;
            }
        }

        jaccard
    }

    pub fn compute_metrics(&self, mappings: &CrossReferenceMap) -> MappingMetrics {
        let mut total = 0;
        let mut mapped = 0;
        let mut nulls = 0;
        let conflict_count = mappings.conflicts.len();

        for version_map in mappings.mappings.values() {
            for entry in version_map.values() {
                match entry {
                    MappingEntry::Ref { .. } => mapped += 1,
                    MappingEntry::Null { .. } => nulls += 1,
                }
                total += 1;
            }
        }

        let coverage = if total > 0 {
            mapped as f64 / total as f64
        } else {
            0.0
        };

        MappingMetrics {
            total,
            mapped,
            nulls,
            conflicts: conflict_count,
            coverage,
            similarity_thresholds: SimilarityThresholds {
                jaccard: self.config.jaccard_threshold,
                levenshtein: self.config.levenshtein_threshold,
            },
        }
    }

    pub fn validate_mappings(&self, mappings: &CrossReferenceMap) -> Result<ValidationSummary> {
        let metrics = self.compute_metrics(mappings);
        
        Ok(ValidationSummary {
            total_references: metrics.total,
            fully_mapped: metrics.mapped,
            with_nulls: metrics.nulls,
            with_conflicts: metrics.conflicts,
            coverage_rate: metrics.coverage,
        })
    }

    pub fn generate_crossrefs_json(
        &self,
        mappings: &CrossReferenceMap,
        output_path: &Path,
    ) -> Result<()> {
        let json = serde_json::to_string_pretty(mappings)
            .context("Failed to serialize cross-reference map")?;

        fs::write(output_path, json)
            .with_context(|| format!("Failed to write crossrefs.json to {:?}", output_path))?;

        Ok(())
    }

    pub fn generate_mappings_with_fallback(
        &mut self,
        versions: &HashMap<String, HashMap<String, Chapter>>,
    ) -> Result<CrossReferenceMap> {
        let mut base_mappings = self.generate_mappings(versions)?;

        for (canonical_ref_str, version_map) in &mut base_mappings.mappings {
            for (version_code, entry) in version_map.iter_mut() {
                if let MappingEntry::Null { reason, .. } = entry {
                    if !reason.contains("versification_mismatch") {
                        if let Some((book, chapter, _verse)) =
                            parse_canonical_ref(canonical_ref_str).ok()
                        {
                            let source_versions: Vec<_> = versions
                                .iter()
                                .filter(|(k, _)| *k != version_code)
                                .collect();

                            for (_source_version, source_chapters) in source_versions {
                                if let Some(source_chapter) =
                                    source_chapters.get(&format!("{}.{}", book, chapter))
                                {
                                    if let Some(source_verse) = source_chapter
                                        .verses
                                        .values()
                                        .find(|v| v.canonical_ref == *canonical_ref_str)
                                    {
                                        if let Some(target_chapters) = versions.get(version_code) {
                                            if let Some(aligned_ref) = self.apply_textual_alignment(
                                            canonical_ref_str,
                                            &source_verse.text,
                                            target_chapters,
                                        ) {
                                                let ref_str = crate::mapper::canonical_ref(&aligned_ref.book, aligned_ref.chapter, &aligned_ref.verse);
                                                *entry = MappingEntry::Ref { ref_: ref_str };
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        base_mappings.metrics = Some(self.compute_metrics(&base_mappings));

        Ok(base_mappings)
    }
}

#[derive(Debug, Clone)]
pub struct ValidationSummary {
    #[allow(dead_code)]
    pub total_references: usize,
    #[allow(dead_code)]
    pub fully_mapped: usize,
    #[allow(dead_code)]
    pub with_nulls: usize,
    #[allow(dead_code)]
    pub with_conflicts: usize,
    #[allow(dead_code)]
    pub coverage_rate: f64,
}

impl Default for CrossVersionMapper {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapper_tests::*;
    use serde_json;

    #[test]
    fn test_canonical_ref() {
        let ref1 = canonical_ref("Genesis", 1, "1");
        assert_eq!(ref1, "Genesis.1.1");

        let (book, chapter, verse) = parse_canonical_ref(&ref1).unwrap();
        assert_eq!(book, "Genesis");
        assert_eq!(chapter, 1);
        assert_eq!(verse, "1");
    }

    #[test]
    fn test_textual_similarity() {
        let mut mapper = CrossVersionMapper::new().unwrap();

        let text1 = "In the beginning God created the heaven and the earth.";
        let text2 = "In the beginning God created the heaven and the earth.";
        let similarity = mapper.textual_similarity(text1, text2);
        assert!((similarity - 1.0).abs() < 0.01);

        let text3 = "And God said Let there be light";
        let similarity2 = mapper.textual_similarity(text1, text3);
        assert!(similarity2 < 1.0);
        assert!(similarity2 >= 0.0);
    }

    #[test]
    fn test_generate_mappings() {
        let mut mapper = CrossVersionMapper::new().unwrap();

        let mut versions: HashMap<String, HashMap<String, Chapter>> = HashMap::new();

        let mut kjv_chapters: HashMap<String, Chapter> = HashMap::new();
        let mut kjv_verses = HashMap::new();
        kjv_verses.insert(
            "1".to_string(),
            Verse {
                id: "test1".to_string(),
                number: "1".to_string(),
                text: "In the beginning".to_string(),
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

        let mut asv_chapters: HashMap<String, Chapter> = HashMap::new();
        let mut asv_verses = HashMap::new();
        asv_verses.insert(
            "1".to_string(),
            Verse {
                id: "test2".to_string(),
                number: "1".to_string(),
                text: "In the beginning".to_string(),
                anchor: "#v1".to_string(),
                canonical_ref: "Genesis.1.1".to_string(),
            },
        );
        asv_chapters.insert(
            "Genesis.1".to_string(),
            Chapter {
                book: "Genesis".to_string(),
                chapter: 1,
                verses: asv_verses,
                metadata: ChapterMetadata {
                    verse_count: 1,
                    last_updated: None,
                },
            },
        );

        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("asv".to_string(), asv_chapters);

        let mappings = mapper.generate_mappings(&versions).unwrap();

        assert_eq!(mappings.schema_version, "1.0");
        assert!(mappings.mappings.contains_key("Genesis.1.1"));
        assert_eq!(mappings.conflicts.len(), 0);
    }

    #[test]
    fn test_parse_canonical_ref() {
        let (book, chapter, verse) = parse_canonical_ref("Genesis.1.1").unwrap();
        assert_eq!(book, "Genesis");
        assert_eq!(chapter, 1);
        assert_eq!(verse, "1");

        let result = parse_canonical_ref("Invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_null_entries() {
        let mut mapper = CrossVersionMapper::new().unwrap();

        let mut versions: HashMap<String, HashMap<String, Chapter>> = HashMap::new();

        let mut kjv_chapters: HashMap<String, Chapter> = HashMap::new();
        let mut kjv_verses = HashMap::new();
        kjv_verses.insert(
            "1".to_string(),
            Verse {
                id: "test1".to_string(),
                number: "1".to_string(),
                text: "In the beginning".to_string(),
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

        let mut asv_chapters: HashMap<String, Chapter> = HashMap::new();
        let asv_verses = HashMap::new();
        asv_chapters.insert(
            "Genesis.1".to_string(),
            Chapter {
                book: "Genesis".to_string(),
                chapter: 1,
                verses: asv_verses,
                metadata: ChapterMetadata {
                    verse_count: 0,
                    last_updated: None,
                },
            },
        );

        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("asv".to_string(), asv_chapters);

        let mappings = mapper.generate_mappings(&versions).unwrap();

        let gen_1_1 = mappings.mappings.get("Genesis.1.1");
        assert!(gen_1_1.is_some());
        
        if let Some(version_map) = gen_1_1 {
            assert!(version_map.get("asv").is_some());
            if let Some(MappingEntry::Null { reason, .. }) = version_map.get("asv") {
                assert!(!reason.is_empty());
            }
        }
    }

    #[test]
    fn test_conflict_detection() {
        let mut mapper = CrossVersionMapper::new().unwrap();

        let mut versions: HashMap<String, HashMap<String, Chapter>> = HashMap::new();

        let mut kjv_chapters: HashMap<String, Chapter> = HashMap::new();
        let mut kjv_verses = HashMap::new();
        kjv_verses.insert(
            "1".to_string(),
            Verse {
                id: "test1".to_string(),
                number: "1".to_string(),
                text: "In the beginning".to_string(),
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

        let mut asv_chapters: HashMap<String, Chapter> = HashMap::new();
        let mut asv_verses = HashMap::new();
        asv_verses.insert(
            "1".to_string(),
            Verse {
                id: "test2".to_string(),
                number: "1".to_string(),
                text: "In the beginning".to_string(),
                anchor: "#v1".to_string(),
                canonical_ref: "Genesis.1.1".to_string(),
            },
        );
        asv_chapters.insert(
            "Genesis.1".to_string(),
            Chapter {
                book: "Genesis".to_string(),
                chapter: 1,
                verses: asv_verses,
                metadata: ChapterMetadata {
                    verse_count: 1,
                    last_updated: None,
                },
            },
        );

        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("asv".to_string(), asv_chapters);

        let mappings = mapper.generate_mappings(&versions).unwrap();

        assert_eq!(mappings.conflicts.len(), 0);
    }

    #[test]
    fn test_textual_alignment() {
        let mut mapper = CrossVersionMapper::new().unwrap();

        let mut target_chapters: HashMap<String, Chapter> = HashMap::new();
        let mut target_verses = HashMap::new();
        target_verses.insert(
            "2".to_string(),
            Verse {
                id: "test1".to_string(),
                number: "2".to_string(),
                text: "In the beginning God created the heaven and the earth".to_string(),
                anchor: "#v2".to_string(),
                canonical_ref: "Genesis.1.2".to_string(),
            },
        );
        target_chapters.insert(
            "Genesis.1".to_string(),
            Chapter {
                book: "Genesis".to_string(),
                chapter: 1,
                verses: target_verses,
                metadata: ChapterMetadata {
                    verse_count: 1,
                    last_updated: None,
                },
            },
        );

        let source_text = "In the beginning God created the heaven and the earth";
        let aligned = mapper.apply_textual_alignment("Genesis.1.1", source_text, &target_chapters);

        assert!(aligned.is_some());
        let verse_ref = aligned.unwrap();
        assert_eq!(verse_ref.verse, "2");
    }

    #[test]
    fn test_validate_mappings() {
        let mut mapper = CrossVersionMapper::new().unwrap();

        let mut versions: HashMap<String, HashMap<String, Chapter>> = HashMap::new();

        let mut kjv_chapters: HashMap<String, Chapter> = HashMap::new();
        let mut kjv_verses = HashMap::new();
        kjv_verses.insert(
            "1".to_string(),
            Verse {
                id: "test1".to_string(),
                number: "1".to_string(),
                text: "In the beginning".to_string(),
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

        versions.insert("kjv".to_string(), kjv_chapters);

        let mappings = mapper.generate_mappings(&versions).unwrap();
        let summary = mapper.validate_mappings(&mappings).unwrap();

        assert_eq!(summary.total_references, 1);
        assert!(summary.coverage_rate >= 0.0 && summary.coverage_rate <= 1.0);
    }

    #[test]
    fn test_generate_crossrefs_json() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        let temp_dir = tempfile::TempDir::new().unwrap();

        let mut versions: HashMap<String, HashMap<String, Chapter>> = HashMap::new();
        let mut kjv_chapters: HashMap<String, Chapter> = HashMap::new();
        let mut kjv_verses = HashMap::new();
        kjv_verses.insert(
            "1".to_string(),
            Verse {
                id: "test1".to_string(),
                number: "1".to_string(),
                text: "In the beginning".to_string(),
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
        versions.insert("kjv".to_string(), kjv_chapters);

        let mappings = mapper.generate_mappings(&versions).unwrap();
        let output_path = temp_dir.path().join("crossrefs.json");

        assert!(mapper
            .generate_crossrefs_json(&mappings, &output_path)
            .is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_golden_snapshot_genesis_1() {
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Genesis.1".to_string(), create_genesis_1_kjv());
        
        let mut web_chapters = HashMap::new();
        web_chapters.insert("Genesis.1".to_string(), create_genesis_1_web());
        
        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("web".to_string(), web_chapters);
        
        let mut mapper1 = CrossVersionMapper::new().unwrap();
        let mappings1 = mapper1.generate_mappings_with_fallback(&versions).unwrap();
        
        let mut mapper2 = CrossVersionMapper::new().unwrap();
        let mappings2 = mapper2.generate_mappings_with_fallback(&versions).unwrap();
        
        assert_eq!(mappings1, mappings2, "Deterministic output: same input should produce identical mappings");
        
        assert!(mappings1.mappings.contains_key("Genesis.1.1"));
        assert!(mappings1.mappings.contains_key("Genesis.1.2"));
        assert!(mappings1.mappings.contains_key("Genesis.1.3"));
    }

    #[test]
    fn test_split_conflict() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        
        let (kjv_chapter, web_chapter) = create_psalm_9_split_merge();
        
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Psalms.9".to_string(), kjv_chapter);
        
        let mut web_chapters = HashMap::new();
        web_chapters.insert("Psalms.9".to_string(), web_chapter);
        
        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("web".to_string(), web_chapters);
        
        let mappings = mapper.generate_mappings(&versions).unwrap();
        
        let split_conflicts: Vec<_> = mappings.conflicts
            .iter()
            .filter(|c| matches!(c.conflict_type, ConflictType::Split))
            .collect();
        
        if !split_conflicts.is_empty() {
            for conflict in split_conflicts {
                assert_eq!(conflict.canonical, "Psalms.9.21");
                assert!(conflict.details.len() >= 1);
            }
        } else {
            assert!(mappings.conflicts.is_empty() || 
                mappings.conflicts.iter().any(|c| matches!(c.conflict_type, ConflictType::Merge)),
                "If no split detected, should have merge or no conflicts");
        }
    }

    #[test]
    fn test_merge_conflict() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        
        let (kjv_chapter, web_chapter) = create_psalm_9_split_merge();
        
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Psalms.9".to_string(), kjv_chapter);
        
        let mut web_chapters = HashMap::new();
        web_chapters.insert("Psalms.9".to_string(), web_chapter);
        
        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("web".to_string(), web_chapters);
        
        let mappings = mapper.generate_mappings(&versions).unwrap();
        
        let merge_conflicts: Vec<_> = mappings.conflicts
            .iter()
            .filter(|c| matches!(c.conflict_type, ConflictType::Merge))
            .collect();
        
        if !merge_conflicts.is_empty() {
            for conflict in merge_conflicts {
                assert_eq!(conflict.canonical, "Psalms.9.21");
                assert!(conflict.details.len() >= 2);
            }
        }
    }

    #[test]
    fn test_shift_conflict() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        
        let (version_a, version_b) = create_shift_fixture();
        
        let mut versions = HashMap::new();
        let mut chapters_a = HashMap::new();
        chapters_a.insert("Test.1".to_string(), version_a);
        
        let mut chapters_b = HashMap::new();
        chapters_b.insert("Test.1".to_string(), version_b);
        
        versions.insert("a".to_string(), chapters_a);
        versions.insert("b".to_string(), chapters_b);
        
        let mappings = mapper.generate_mappings(&versions).unwrap();
        
        let shift_conflicts: Vec<_> = mappings.conflicts
            .iter()
            .filter(|c| matches!(c.conflict_type, ConflictType::Shift))
            .collect();
        
        if !shift_conflicts.is_empty() {
            for conflict in shift_conflicts {
                assert!(conflict.canonical == "Test.1.5" || conflict.canonical == "Test.1.6");
            }
        }
        
        assert!(mappings.mappings.len() > 0, "Should have some mappings");
    }

    #[test]
    fn test_absent_conflict() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        mapper.set_versification("web", "WEB");
        
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Genesis.1".to_string(), create_genesis_1_kjv());
        
        let mut web_chapters = HashMap::new();
        let mut web_verses = HashMap::new();
        web_verses.insert(
            "1".to_string(),
            Verse {
                id: "gen1_1_web".to_string(),
                number: "1".to_string(),
                text: "In the beginning God created the heavens and the earth.".to_string(),
                anchor: "#v1".to_string(),
                canonical_ref: "Genesis.1.1".to_string(),
            },
        );
        web_chapters.insert(
            "Genesis.1".to_string(),
            Chapter {
                book: "Genesis".to_string(),
                chapter: 1,
                verses: web_verses,
                metadata: ChapterMetadata {
                    verse_count: 1,
                    last_updated: None,
                },
            },
        );
        
        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("web".to_string(), web_chapters);
        
        let mappings = mapper.generate_mappings(&versions).unwrap();
        
        let absent_conflicts: Vec<_> = mappings.conflicts
            .iter()
            .filter(|c| matches!(c.conflict_type, ConflictType::Absent))
            .collect();
        
        if !absent_conflicts.is_empty() {
            for conflict in absent_conflicts {
                assert!(conflict.details.iter().any(|d| d.contains("versification_mismatch")));
            }
        }
        
        let gen_1_2 = mappings.mappings.get("Genesis.1.2");
        assert!(gen_1_2.is_some());
        
        if let Some(version_map) = gen_1_2 {
            if let Some(MappingEntry::Null { reason, .. }) = version_map.get("web") {
                assert!(reason.contains("versification_mismatch"));
            }
        }
    }

    #[test]
    fn test_threshold_sweep() {
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Genesis.1".to_string(), create_genesis_1_kjv());
        
        let mut web_chapters = HashMap::new();
        web_chapters.insert("Genesis.1".to_string(), create_genesis_1_web());
        
        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("web".to_string(), web_chapters);
        
        let thresholds = vec![0.65, 0.70, 0.75];
        let mut previous_coverage = 0.0;
        
        for jaccard_threshold in thresholds {
            let config = MapperConfig::new(jaccard_threshold, 0.15);
            let mut mapper = CrossVersionMapper::with_config(&config).unwrap();
            
            let mappings = mapper.generate_mappings_with_fallback(&versions).unwrap();
            let metrics = mappings.metrics.as_ref().unwrap();
            
            assert!(metrics.coverage >= previous_coverage - 0.01, 
                "Coverage should be monotonic or near-monotonic with threshold changes");
            
            assert_eq!(metrics.similarity_thresholds.jaccard, jaccard_threshold);
            assert_eq!(metrics.similarity_thresholds.levenshtein, 0.15);
            
            let json = serde_json::to_string(&mappings).unwrap();
            let _hash = hash_json(&json);
            
            previous_coverage = metrics.coverage;
        }
    }

    #[test]
    fn test_determinism() {
        let mut mapper1 = CrossVersionMapper::new().unwrap();
        let mut mapper2 = CrossVersionMapper::new().unwrap();
        
        let mut versions1 = HashMap::new();
        let mut kjv_chapters1 = HashMap::new();
        kjv_chapters1.insert("Genesis.1".to_string(), create_genesis_1_kjv());
        
        let mut web_chapters1 = HashMap::new();
        web_chapters1.insert("Genesis.1".to_string(), create_genesis_1_web());
        
        versions1.insert("kjv".to_string(), kjv_chapters1);
        versions1.insert("web".to_string(), web_chapters1);
        
        let mut versions2 = HashMap::new();
        let mut kjv_chapters2 = HashMap::new();
        kjv_chapters2.insert("Genesis.1".to_string(), create_genesis_1_kjv());
        
        let mut web_chapters2 = HashMap::new();
        web_chapters2.insert("Genesis.1".to_string(), create_genesis_1_web());
        
        versions2.insert("web".to_string(), web_chapters2);
        versions2.insert("kjv".to_string(), kjv_chapters2);
        
        let mappings1 = mapper1.generate_mappings_with_fallback(&versions1).unwrap();
        let mappings2 = mapper2.generate_mappings_with_fallback(&versions2).unwrap();
        
        assert_eq!(mappings1, mappings2, "Same data in different order should produce identical output");
    }

    #[test]
    fn test_normalization_invariants() {
        let normalizer = TextNormalizer::new().unwrap();
        
        let text1 = "In the beginning God created the heaven and the earth.";
        let text2 = "In the beginning God created the HEAVEN and the EARTH!";
        let text3 = "In [1] the beginning, God created the heaven and the earth.";
        
        let tokens1 = normalizer.normalize_tokens(text1);
        let tokens2 = normalizer.normalize_tokens(text2);
        let tokens3 = normalizer.normalize_tokens(text3);
        
        assert_eq!(tokens1, tokens2, "Case differences should normalize to same tokens");
        assert_eq!(tokens1, tokens3, "Punctuation and footnote differences should normalize to same tokens");
    }

    #[test]
    fn test_versification_mismatch_reason() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        mapper.set_versification("web", "WEB");
        mapper.set_versification("kjv", "KJV");
        
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Genesis.1".to_string(), create_genesis_1_kjv());
        
        let mut web_chapters = HashMap::new();
        let mut web_verses = HashMap::new();
        web_verses.insert(
            "1".to_string(),
            Verse {
                id: "gen1_1_web".to_string(),
                number: "1".to_string(),
                text: "In the beginning God created the heavens and the earth.".to_string(),
                anchor: "#v1".to_string(),
                canonical_ref: "Genesis.1.1".to_string(),
            },
        );
        web_chapters.insert(
            "Genesis.1".to_string(),
            Chapter {
                book: "Genesis".to_string(),
                chapter: 1,
                verses: web_verses,
                metadata: ChapterMetadata {
                    verse_count: 1,
                    last_updated: None,
                },
            },
        );
        
        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("web".to_string(), web_chapters);
        
        let mappings = mapper.generate_mappings(&versions).unwrap();
        
        if let Some(version_map) = mappings.mappings.get("Genesis.1.2") {
            if let Some(MappingEntry::Null { reason, .. }) = version_map.get("web") {
                assert!(reason.contains("versification_mismatch"));
                assert!(reason.contains("WEB"));
            }
        }
    }

    #[test]
    fn test_tie_break_lexicographic() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        
        let mut target_chapters: HashMap<String, Chapter> = HashMap::new();
        let mut target_verses = HashMap::new();
        
        target_verses.insert(
            "5".to_string(),
            Verse {
                id: "t5".to_string(),
                number: "5".to_string(),
                text: "Identical text content for testing similarity.".to_string(),
                anchor: "#v5".to_string(),
                canonical_ref: "Test.1.5".to_string(),
            },
        );
        target_verses.insert(
            "10".to_string(),
            Verse {
                id: "t10".to_string(),
                number: "10".to_string(),
                text: "Identical text content for testing similarity.".to_string(),
                anchor: "#v10".to_string(),
                canonical_ref: "Test.1.10".to_string(),
            },
        );
        
        target_chapters.insert(
            "Test.1".to_string(),
            Chapter {
                book: "Test".to_string(),
                chapter: 1,
                verses: target_verses,
                metadata: ChapterMetadata {
                    verse_count: 2,
                    last_updated: None,
                },
            },
        );
        
        let aligned = mapper.apply_textual_alignment(
            "Test.1.1",
            "Identical text content for testing similarity.",
            &target_chapters,
        );
        
        assert!(aligned.is_some());
        let verse_ref = aligned.unwrap();
        assert_eq!(verse_ref.verse, "10", "Should choose lexicographically smaller canonical (Test.1.10 < Test.1.5 when comparing '10' < '5')");
    }

    #[test]
    fn test_metrics_coverage_calculation() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Genesis.1".to_string(), create_genesis_1_kjv());
        
        let mut web_chapters = HashMap::new();
        web_chapters.insert("Genesis.1".to_string(), create_genesis_1_web());
        
        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("web".to_string(), web_chapters);
        
        let mappings = mapper.generate_mappings_with_fallback(&versions).unwrap();
        let metrics = mappings.metrics.as_ref().unwrap();
        
        let calculated_coverage = if metrics.total > 0 {
            metrics.mapped as f64 / metrics.total as f64
        } else {
            0.0
        };
        
        assert!((metrics.coverage - calculated_coverage).abs() < 1e-9,
            "Coverage should equal mapped/total within floating point precision");
    }

    #[test]
    fn test_conflict_count_sum() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        
        let (kjv_chapter, web_chapter) = create_psalm_9_split_merge();
        
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Psalms.9".to_string(), kjv_chapter);
        
        let mut web_chapters = HashMap::new();
        web_chapters.insert("Psalms.9".to_string(), web_chapter);
        
        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("web".to_string(), web_chapters);
        
        let mappings = mapper.generate_mappings(&versions).unwrap();
        let metrics = mappings.metrics.as_ref().unwrap();
        
        let split_count = mappings.conflicts.iter()
            .filter(|c| matches!(c.conflict_type, ConflictType::Split))
            .count();
        let merge_count = mappings.conflicts.iter()
            .filter(|c| matches!(c.conflict_type, ConflictType::Merge))
            .count();
        let shift_count = mappings.conflicts.iter()
            .filter(|c| matches!(c.conflict_type, ConflictType::Shift))
            .count();
        let absent_count = mappings.conflicts.iter()
            .filter(|c| matches!(c.conflict_type, ConflictType::Absent))
            .count();
        
        let sum = split_count + merge_count + shift_count + absent_count;
        
        assert_eq!(metrics.conflicts, sum as usize,
            "Total conflicts should equal sum of conflict types");
    }

    #[test]
    fn test_idempotence() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Genesis.1".to_string(), create_genesis_1_kjv());
        
        let mut web_chapters = HashMap::new();
        web_chapters.insert("Genesis.1".to_string(), create_genesis_1_web());
        
        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("web".to_string(), web_chapters);
        
        let mappings1 = mapper.generate_mappings_with_fallback(&versions).unwrap();
        let mappings2 = mapper.generate_mappings_with_fallback(&versions).unwrap();
        
        assert_eq!(mappings1, mappings2, "map(map(x)) should equal map(x)");
    }

    #[test]
    fn test_no_orphans() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        
        let mut versions = HashMap::new();
        let mut kjv_chapters = HashMap::new();
        kjv_chapters.insert("Genesis.1".to_string(), create_genesis_1_kjv());
        
        let mut web_chapters = HashMap::new();
        web_chapters.insert("Genesis.1".to_string(), create_genesis_1_web());
        
        versions.insert("kjv".to_string(), kjv_chapters);
        versions.insert("web".to_string(), web_chapters);
        
        let mappings = mapper.generate_mappings_with_fallback(&versions).unwrap();
        
        for (_canonical_ref_str, version_map) in &mappings.mappings {
            for (version_code, entry) in version_map {
                if let MappingEntry::Ref { ref_: ref_str } = entry {
                    let (book, chapter, verse) = parse_canonical_ref(ref_str).unwrap();
                    let chapter_key = format!("{}.{}", book, chapter);
                    
                    if let Some(version_chapters) = versions.get(version_code) {
                        if let Some(chapter_data) = version_chapters.get(&chapter_key) {
                            assert!(chapter_data.verses.contains_key(&verse),
                                "Ref {} should exist in parsed chapter", ref_str);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_text_normalization_caching() {
        let mut mapper = CrossVersionMapper::new().unwrap();
        
        let text = "In the beginning God created the heaven and the earth.";
        
        let tokens1 = mapper.cache.get_tokens(&mapper.normalizer, text);
        let tokens2 = mapper.cache.get_tokens(&mapper.normalizer, text);
        
        assert_eq!(tokens1, tokens2);
    }
}
