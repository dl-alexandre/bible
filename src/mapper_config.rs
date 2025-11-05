use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapperConfig {
    pub jaccard_threshold: f64,
    pub levenshtein_threshold: f64,
    pub jaccard_min: f64,
    pub jaccard_max: f64,
}

impl Default for MapperConfig {
    fn default() -> Self {
        MapperConfig {
            jaccard_threshold: 0.70,
            levenshtein_threshold: 0.15,
            jaccard_min: 0.60,
            jaccard_max: 0.75,
        }
    }
}

impl MapperConfig {
    pub fn new(jaccard: f64, levenshtein: f64) -> Self {
        MapperConfig {
            jaccard_threshold: jaccard,
            levenshtein_threshold: levenshtein,
            jaccard_min: jaccard - 0.15,
            jaccard_max: jaccard + 0.05,
        }
    }
}

