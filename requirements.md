# Requirements Document

## Introduction

A static site generator that converts public-domain Bible datasets (KJV, ASV, WEB, OEB) into chapter-level HTML and JSON files, hosted on GitHub Pages. The system provides both a browsable HTML interface and a machine-readable JSON API with stable verse anchors and redirectable short URLs.

## Glossary

- **Bible_Generator**: The static site generator system that processes Bible datasets
- **GitHub_Pages**: The hosting platform that serves the generated static site
- **Verse_Anchor**: A stable HTML anchor in the format #v{verse_number} for direct verse linking
- **Short_URL**: A redirectable URL pattern /bible/{id}/{book}.{chapter}.{verse}.{version} that points to chapter pages
- **Cross_Version_Mapping**: Automatic mapping system that links equivalent verses across different Bible versions
- **Deterministic_ID**: Consistent, reproducible identifier generation that produces the same output for the same input
- **Source_Text**: Raw Bible dataset files in public domain (KJV, ASV, WEB, OEB)
- **Chapter_Page**: Individual HTML page containing all verses for a specific chapter
- **Metadata_Files**: JSON files containing structural information (books.json, versions.json, crossrefs.json)
- **Static_Site**: The complete generated website compatible with GitHub Pages hosting
- **Input_Validation**: Process of detecting and handling malformed, duplicate, or missing verses in Source_Text
- **Diagnostic_Output**: Log files or JSON reports generated during the build process for troubleshooting
- **Schema_Definition**: Standardized JSON structure specification for metadata files
- **Canonical_Verse_Number**: Standard verse numbering system used as the basis for cross-version mapping

## Requirements

### Requirement 1

**User Story:** As a Bible study application developer, I want to access Bible text through a JSON API, so that I can integrate scripture content into my application.

#### Acceptance Criteria

1. THE Bible_Generator SHALL generate one JSON file per chapter in the format /{version}/{book}/{chapter}.json
2. THE Bible_Generator SHALL use stable verse keys as object properties rather than array indices
3. THE Bible_Generator SHALL create a versions.json file listing all available Bible versions
4. THE Bible_Generator SHALL create a books.json file containing book metadata and chapter counts
5. THE Bible_Generator SHALL include schema_version field in all JSON output for compatibility tracking

### Requirement 2

**User Story:** As a web user, I want to browse Bible chapters in HTML format, so that I can read scripture content in a web browser.

#### Acceptance Criteria

1. THE Bible_Generator SHALL create individual HTML pages for each chapter in each version
2. THE Bible_Generator SHALL organize HTML files in per-version directory structures
3. THE Bible_Generator SHALL include navigation elements between chapters and books
4. THE Bible_Generator SHALL generate a browsable index page listing all versions and books
5. THE Bible_Generator SHALL ensure HTML output is valid and accessible

### Requirement 3

**User Story:** As a scripture reference system, I want stable verse anchors, so that I can link directly to specific verses.

#### Acceptance Criteria

1. THE Bible_Generator SHALL create HTML anchors in the format #v{verse_number} for each verse
2. THE Bible_Generator SHALL ensure verse anchors remain consistent across regenerations
3. THE Bible_Generator SHALL include verse numbers as clickable elements in HTML output
4. THE Bible_Generator SHALL generate anchor links that work with browser navigation
5. THE Bible_Generator SHALL maintain anchor stability when source text is updated

### Requirement 4

**User Story:** As a Bible app developer, I want short URLs for verse references, so that I can create shareable links to specific verses.

#### Acceptance Criteria

1. THE Bible_Generator SHALL create redirectable URLs in the format /bible/{id}/{book}.{chapter}.{verse}.{version}
2. THE Bible_Generator SHALL generate redirect pages that point to the appropriate chapter page with verse anchor
3. THE Bible_Generator SHALL ensure short URLs work with GitHub_Pages static hosting limitations
4. THE Bible_Generator SHALL maintain URL consistency across different Bible versions
5. THE Bible_Generator SHALL generate HTML redirect files using meta refresh or JavaScript fallback for GitHub_Pages static routing
6. THE Bible_Generator SHALL include canonical link elements in format <link rel="canonical" href="..."> in redirect pages

### Requirement 5

**User Story:** As a content maintainer, I want deterministic ID generation, so that builds are reproducible and version control friendly.

#### Acceptance Criteria

1. THE Bible_Generator SHALL generate identical output files when processing the same Source_Text
2. THE Bible_Generator SHALL use consistent naming conventions for all generated files
3. THE Bible_Generator SHALL ensure file timestamps and metadata are deterministic
4. THE Bible_Generator SHALL produce reproducible builds across different environments
5. THE Bible_Generator SHALL maintain stable file paths and directory structures

### Requirement 6

**User Story:** As a deployment system, I want optimized build output, so that the generated site loads quickly and uses minimal storage.

#### Acceptance Criteria

1. THE Bible_Generator SHALL minimize HTML file sizes through efficient markup generation
2. THE Bible_Generator SHALL eliminate duplicate content across generated files
3. THE Bible_Generator SHALL optimize directory structure for GitHub_Pages hosting
4. THE Bible_Generator SHALL generate a Static_Site compatible with GitHub_Pages file size and structure limitations

### Requirement 7

**User Story:** As a Bible study researcher, I want cross-version mapping, so that I can compare equivalent verses across different translations.

#### Acceptance Criteria

1. THE Bible_Generator SHALL use Canonical_Verse_Number as the primary basis for cross-version mapping
2. THE Bible_Generator SHALL generate crossrefs.json containing version-to-version mappings following documented algorithms in external specification
3. WHEN verse numbering differs between translations, THE Bible_Generator SHALL apply textual alignment fallback methods
4. THE Bible_Generator SHALL provide consistent reference systems across all versions using deterministic mapping rules
5. THE Bible_Generator SHALL store verse mismatches as null entries with reason field in crossrefs.json to preserve semantic context

### Requirement 8

**User Story:** As a system administrator, I want input validation and error handling, so that I can identify and resolve issues with source datasets.

#### Acceptance Criteria

1. THE Bible_Generator SHALL validate Source_Text for malformed verse formatting and report errors
2. THE Bible_Generator SHALL detect duplicate verse entries within the same chapter and version
3. WHEN missing verses are detected, THE Bible_Generator SHALL log the gaps and continue processing
4. THE Bible_Generator SHALL generate Diagnostic_Output in JSONL format with severity levels and rotate logs by deleting oldest when exceeding 10 builds
5. THE Bible_Generator SHALL sanitize text content using HTML entity encoding and prohibit inline scripts

### Requirement 9

**User Story:** As an API consumer, I want standardized metadata schemas, so that I can reliably parse and integrate the JSON data.

#### Acceptance Criteria

1. THE Bible_Generator SHALL follow a defined Schema_Definition for all Metadata_Files
2. THE Bible_Generator SHALL include version information in all JSON output for schema compatibility
3. THE Bible_Generator SHALL include placeholder extension fields in all JSON outputs for non-breaking additions
4. THE Bible_Generator SHALL validate generated JSON against the defined schema before output
5. THE Bible_Generator SHALL store all schema files under /schema/ directory with versioned filenames
6. THE Bible_Generator SHALL test backward compatibility between schema versions during validation

### Requirement 10

**User Story:** As a web accessibility user, I want semantic HTML structure, so that I can navigate the content with assistive technologies.

#### Acceptance Criteria

1. THE Bible_Generator SHALL use semantic HTML elements including main, nav, and section tags
2. THE Bible_Generator SHALL include ARIA roles and labels for navigation elements
3. THE Bible_Generator SHALL ensure proper heading hierarchy in generated HTML
4. THE Bible_Generator SHALL provide skip links for keyboard navigation
5. THE Bible_Generator SHALL validate accessibility using automated tools like axe-core or pa11y

### Requirement 11

**User Story:** As a performance-conscious user, I want quantified build optimization, so that the site meets specific performance thresholds.

#### Acceptance Criteria

1. THE Bible_Generator SHALL ensure HTML file size per chapter remains below 50 KB
2. THE Bible_Generator SHALL minify JSON output for production and optionally generate precompressed .json.gz artifacts with application/gzip MIME type
3. THE Bible_Generator SHALL complete full site generation within 2 minutes on GitHub Actions standard runners
4. THE Bible_Generator SHALL optimize asset loading for GitHub_Pages hosting constraints
5. THE Bible_Generator SHALL minimize total repository size for efficient Git operations

### Requirement 12

**User Story:** As a content maintainer, I want automated testing and validation, so that I can verify build correctness and consistency.

#### Acceptance Criteria

1. THE Bible_Generator SHALL include automated tests verifying deterministic output hashes
2. THE Bible_Generator SHALL validate that all generated Verse_Anchor links function correctly
3. THE Bible_Generator SHALL test Short_URL redirects for proper routing
4. THE Bible_Generator SHALL verify Cross_Version_Mapping accuracy through automated comparison
5. THE Bible_Generator SHALL provide test reports confirming schema compliance, data integrity, and accessibility verification

### Requirement 13

**User Story:** As a content maintainer, I want automated deployment, so that dataset changes trigger automatic site regeneration and deployment.

#### Acceptance Criteria

1. THE Bible_Generator SHALL integrate with GitHub Actions for continuous integration and deployment
2. WHEN Source_Text changes are detected, THE Bible_Generator SHALL automatically trigger a rebuild
3. THE Bible_Generator SHALL deploy updated Static_Site to GitHub_Pages without manual intervention
4. THE Bible_Generator SHALL validate build success before deploying to production
5. THE Bible_Generator SHALL maintain build logs with masked secrets and exclude sensitive data from deployment artifacts

### Requirement 14

**User Story:** As a downstream tool developer, I want a global manifest, so that I can discover available data and schema versions programmatically.

#### Acceptance Criteria

1. THE Bible_Generator SHALL create a manifest.json file containing available versions, schema version, and build metadata
2. THE Bible_Generator SHALL include build timestamp and source dataset checksums in the manifest
3. THE Bible_Generator SHALL provide API endpoint discovery information in the manifest
4. THE Bible_Generator SHALL link manifest.json in HTML head elements and serve at /manifest.json root for API consumers
5. THE Bible_Generator SHALL validate manifest.json against /schema/manifest-{version}.json for consistency
6. THE Bible_Generator SHALL update the manifest atomically with each successful build