use crate::models::*;
use schemars::schema_for;
use std::fs;
use std::path::Path;

/// Generate all JSON schemas
pub fn generate_schemas(schema_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(schema_dir)?;

    let chapter_schema = schema_for!(ChapterJson);
    let chapter_json = serde_json::to_string_pretty(&chapter_schema)?;
    fs::write(schema_dir.join("chapter-1.0.json"), chapter_json)?;

    let manifest_schema = schema_for!(GlobalManifest);
    let manifest_json = serde_json::to_string_pretty(&manifest_schema)?;
    fs::write(schema_dir.join("manifest-1.0.json"), manifest_json)?;

    let crossrefs_schema = schema_for!(CrossReferenceMap);
    let crossrefs_json = serde_json::to_string_pretty(&crossrefs_schema)?;
    fs::write(schema_dir.join("crossrefs-1.0.json"), crossrefs_json)?;

    let versions_schema = schema_for!(VersionsJson);
    let versions_json = serde_json::to_string_pretty(&versions_schema)?;
    fs::write(schema_dir.join("versions-1.0.json"), versions_json)?;

    let books_schema = schema_for!(BooksJson);
    let books_json = serde_json::to_string_pretty(&books_schema)?;
    fs::write(schema_dir.join("books-1.0.json"), books_json)?;

    Ok(())
}

/// Validate JSON against schema
pub fn validate_json(json: &serde_json::Value, schema_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use jsonschema::JSONSchema;
    
    let schema_content = fs::read_to_string(schema_path)?;
    let schema_json: serde_json::Value = serde_json::from_str(&schema_content)?;
    
    // Compile and validate in the same scope to keep schema_json alive
    let compiled = JSONSchema::compile(&schema_json)
        .map_err(|e| format!("Failed to compile schema: {:?}", e))?;
    
    // Collect all validation errors immediately while compiled is in scope
    let validation_result = compiled.validate(json);
    let error_iter = match validation_result {
        Ok(()) => return Ok(()),
        Err(errors) => errors,
    };
    
    let error_msgs: Vec<String> = error_iter.map(|e| format!("{}", e)).collect();
    if !error_msgs.is_empty() {
        Err(format!("Validation error: {}", error_msgs.join("; ")).into())
    } else {
    Ok(())
    }
}
