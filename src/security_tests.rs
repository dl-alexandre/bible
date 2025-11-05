#[cfg(test)]
mod tests {
    use crate::validation::InputValidator;

    #[test]
    fn test_sanitize_script_injection() {
        let validator = InputValidator::new().unwrap();
        
        let malicious = "<script>alert('xss')</script>In the beginning";
        let sanitized = validator.sanitize_text(malicious);
        
        assert!(!sanitized.contains("<script>"), "Script tags should be removed");
        assert!(!sanitized.contains("alert"), "Script content should be removed");
        assert!(sanitized.contains("In the beginning"), "Legitimate text should remain");
        assert!(sanitized.contains("&lt;"), "HTML should be escaped");
    }

    #[test]
    fn test_sanitize_malformed_html() {
        let validator = InputValidator::new().unwrap();
        
        let malformed = "<p>Unclosed tag<div>Nested<script>evil</div>";
        let sanitized = validator.sanitize_text(malformed);
        
        assert!(!sanitized.contains("<script>"), "Script tags should be removed");
        assert!(sanitized.contains("Unclosed tag"), "Text content should remain");
        assert!(sanitized.contains("&lt;"), "HTML should be escaped");
    }

    #[test]
    fn test_sanitize_event_handlers() {
        let validator = InputValidator::new().unwrap();
        
        let with_events = r#"<div onclick="malicious()">Text</div>"#;
        let sanitized = validator.sanitize_text(with_events);
        
        assert!(!sanitized.contains("onclick"), "Event handlers should be removed");
        assert!(sanitized.contains("Text"), "Legitimate text should remain");
        assert!(sanitized.contains("&lt;"), "HTML should be escaped");
    }

    #[test]
    fn test_sanitize_javascript_uri() {
        let validator = InputValidator::new().unwrap();
        
        let js_uri = r#"<a href="javascript:alert('xss')">Link</a>"#;
        let sanitized = validator.sanitize_text(js_uri);
        
        assert!(!sanitized.contains("javascript:"), "JavaScript URIs should be removed");
        assert!(sanitized.contains("Link"), "Text content should remain");
    }

    #[test]
    fn test_sanitize_iframe() {
        let validator = InputValidator::new().unwrap();
        
        let iframe = r#"<iframe src="evil.com"></iframe>Text"#;
        let sanitized = validator.sanitize_text(iframe);
        
        assert!(!sanitized.contains("<iframe"), "Iframe tags should be removed");
        assert!(sanitized.contains("Text"), "Legitimate text should remain");
    }
}

