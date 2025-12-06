use crate::models::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub fn create_genesis_1_kjv() -> Chapter {
    let mut verses = HashMap::new();
    verses.insert(
        "1".to_string(),
        Verse {
            id: "gen1_1".to_string(),
            number: "1".to_string(),
            text: "In the beginning God created the heaven and the earth.".to_string(),
            anchor: "#v1".to_string(),
            canonical_ref: "Genesis.1.1".to_string(),
        },
    );
    verses.insert(
        "2".to_string(),
        Verse {
            id: "gen1_2".to_string(),
            number: "2".to_string(),
            text: "And the earth was without form, and void; and darkness was upon the face of the deep. And the Spirit of God moved upon the face of the waters.".to_string(),
            anchor: "#v2".to_string(),
            canonical_ref: "Genesis.1.2".to_string(),
        },
    );
    verses.insert(
        "3".to_string(),
        Verse {
            id: "gen1_3".to_string(),
            number: "3".to_string(),
            text: "And God said, Let there be light: and there was light.".to_string(),
            anchor: "#v3".to_string(),
            canonical_ref: "Genesis.1.3".to_string(),
        },
    );

    Chapter {
        book: "Genesis".to_string(),
        chapter: 1,
        verses,
        metadata: ChapterMetadata {
            verse_count: 3,
            last_updated: None,
        },
    }
}

pub fn create_genesis_1_web() -> Chapter {
    let mut verses = HashMap::new();
    verses.insert(
        "1".to_string(),
        Verse {
            id: "gen1_1_web".to_string(),
            number: "1".to_string(),
            text: "In the beginning God created the heavens and the earth.".to_string(),
            anchor: "#v1".to_string(),
            canonical_ref: "Genesis.1.1".to_string(),
        },
    );
    verses.insert(
        "2".to_string(),
        Verse {
            id: "gen1_2_web".to_string(),
            number: "2".to_string(),
            text: "The earth was formless and empty. Darkness was on the surface of the deep. God's Spirit was hovering over the surface of the waters.".to_string(),
            anchor: "#v2".to_string(),
            canonical_ref: "Genesis.1.2".to_string(),
        },
    );
    verses.insert(
        "3".to_string(),
        Verse {
            id: "gen1_3_web".to_string(),
            number: "3".to_string(),
            text: "God said, \"Let there be light,\" and there was light.".to_string(),
            anchor: "#v3".to_string(),
            canonical_ref: "Genesis.1.3".to_string(),
        },
    );

    Chapter {
        book: "Genesis".to_string(),
        chapter: 1,
        verses,
        metadata: ChapterMetadata {
            verse_count: 3,
            last_updated: None,
        },
    }
}

pub fn create_psalm_9_split_merge() -> (Chapter, Chapter) {
    let mut kjv_verses = HashMap::new();
    kjv_verses.insert(
        "20".to_string(),
        Verse {
            id: "ps9_20".to_string(),
            number: "20".to_string(),
            text: "Put them in fear, O LORD: that the nations may know themselves to be but men.".to_string(),
            anchor: "#v20".to_string(),
            canonical_ref: "Psalms.9.20".to_string(),
        },
    );
    kjv_verses.insert(
        "21".to_string(),
        Verse {
            id: "ps9_21".to_string(),
            number: "21".to_string(),
            text: "Set thou a wicked man over him: and let Satan stand at his right hand.".to_string(),
            anchor: "#v21".to_string(),
            canonical_ref: "Psalms.9.21".to_string(),
        },
    );

    let mut web_verses = HashMap::new();
    web_verses.insert(
        "20".to_string(),
        Verse {
            id: "ps9_20_web".to_string(),
            number: "20".to_string(),
            text: "Put them in fear, LORD. Let the nations know that they are only men.".to_string(),
            anchor: "#v20".to_string(),
            canonical_ref: "Psalms.9.20".to_string(),
        },
    );
    web_verses.insert(
        "20-21".to_string(),
        Verse {
            id: "ps9_20_21_web".to_string(),
            number: "20-21".to_string(),
            text: "Set a wicked man over him. Let an adversary stand at his right hand.".to_string(),
            anchor: "#v20-21".to_string(),
            canonical_ref: "Psalms.9.20".to_string(),
        },
    );

    (
        Chapter {
            book: "Psalms".to_string(),
            chapter: 9,
            verses: kjv_verses,
            metadata: ChapterMetadata {
                verse_count: 2,
                last_updated: None,
            },
        },
        Chapter {
            book: "Psalms".to_string(),
            chapter: 9,
            verses: web_verses,
            metadata: ChapterMetadata {
                verse_count: 2,
                last_updated: None,
            },
        },
    )
}

pub fn create_shift_fixture() -> (Chapter, Chapter) {
    let mut version_a = HashMap::new();
    version_a.insert(
        "5".to_string(),
        Verse {
            id: "a5".to_string(),
            number: "5".to_string(),
            text: "This is verse five content.".to_string(),
            anchor: "#v5".to_string(),
            canonical_ref: "Test.1.5".to_string(),
        },
    );

    let mut version_b = HashMap::new();
    version_b.insert(
        "6".to_string(),
        Verse {
            id: "b6".to_string(),
            number: "6".to_string(),
            text: "This is verse five content.".to_string(),
            anchor: "#v6".to_string(),
            canonical_ref: "Test.1.6".to_string(),
        },
    );

    (
        Chapter {
            book: "Test".to_string(),
            chapter: 1,
            verses: version_a,
            metadata: ChapterMetadata {
                verse_count: 1,
                last_updated: None,
            },
        },
        Chapter {
            book: "Test".to_string(),
            chapter: 1,
            verses: version_b,
            metadata: ChapterMetadata {
                verse_count: 1,
                last_updated: None,
            },
        },
    )
}

pub fn hash_json(json: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    format!("{:x}", hasher.finalize())
}

