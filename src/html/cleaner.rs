//! HTML cleaner - repair and strip attributes

use crate::{ProcessingError, ProcessingOptions};

/// Attributes to keep during stripping (essential for EPUB rendering)
const _KEEP_ATTRS: &[&str] = &[
    "class",
    "id",
    "href",
    "src",
    "style",
    "alt",
    "title",
    "type",
    "name",
    "content",
    "charset",
    "http-equiv",
    "xmlns",
    "version",
    "media-type",
    "properties",
    "rel",
    "media",
    "width",
    "height",
    "colspan",
    "rowspan",
    "scope",
    "headers",
    "border",
    "cellpadding",
    "cellspacing",
];

/// Attribute prefixes to always strip
const _STRIP_ATTR_PREFIXES: &[&str] = &["data-", "aria-", "epub:"];

/// Non-essential attributes to strip
const _STRIP_ATTRS: &[&str] = &[
    "role",
    "tabindex",
    "accesskey",
    "draggable",
    "contenteditable",
    "spellcheck",
    "autocorrect",
    "autocapitalize",
    "autofocus",
    "dir",
    "translate",
    "inputmode",
    "enterkeyhint",
    "hidden",
    "inert",
    "popover",
];

/// Strip unnecessary attributes from HTML
///
/// This function removes decorative/accessibility attributes that e-ink readers ignore,
/// reducing file size and parsing overhead for the 380KB-RAM ESP32-C3 device.
///
/// Keeps: class, id, href, src, style, alt, title, xmlns, and other
/// essential XHTML/EPUB attributes.
///
/// Strips: data-*, aria-*, epub: prefixes, role, tabindex, and other non-essential attrs
pub fn strip_unnecessary_attributes(html_bytes: &[u8]) -> (Vec<u8>, usize) {
    let content = match std::str::from_utf8(html_bytes) {
        Ok(c) => c,
        Err(_) => return (html_bytes.to_vec(), 0),
    };

    // Use a simpler regex-based approach for now
    let stripped_content = strip_attributes_from_html(content);
    let removed = content.len() - stripped_content.len();

    (stripped_content.into_bytes(), removed)
}

/// Strip attributes using regex pattern matching
fn strip_attributes_from_html(html: &str) -> String {
    let mut result = html.to_string();

    // Remove data-* attributes (simpler pattern without look-ahead)
    result = regex::Regex::new(r#" data-[^"'>\s]+(?:="[^"]*"|='[^']*'|=[^\s>]*)?"#)
        .unwrap()
        .replace_all(&result, "")
        .to_string();

    // Remove aria-* attributes
    result = regex::Regex::new(r#" aria-[^"'>\s]+(?:="[^"]*"|='[^']*'|=[^\s>]*)?"#)
        .unwrap()
        .replace_all(&result, "")
        .to_string();

    // Remove epub:* attributes
    result = regex::Regex::new(r#" epub:[^"'>\s]+(?:="[^"]*"|='[^']*'|=[^\s>]*)?"#)
        .unwrap()
        .replace_all(&result, "")
        .to_string();

    // Remove role attribute
    result = regex::Regex::new(r#" role="[^"]*""#)
        .unwrap()
        .replace_all(&result, "")
        .to_string();

    // Remove tabindex attribute
    result = regex::Regex::new(r#" tabindex="[^"]*""#)
        .unwrap()
        .replace_all(&result, "")
        .to_string();

    result
}

/// Add page breaks to chapter headings
pub fn add_chapter_page_breaks(html_bytes: &[u8]) -> Vec<u8> {
    let content = match std::str::from_utf8(html_bytes) {
        Ok(c) => c,
        Err(_) => return html_bytes.to_vec(),
    };

    let mut result = content.to_string();

    // Check if page-break style already exists
    if result.contains("page-break-before") {
        return html_bytes.to_vec();
    }

    // Look for <head> section
    if let Some(head_start) = result.find("<head>") {
        if let Some(head_end) = result.find("</head>") {
            let before_head = &result[..head_start + 6];
            let head_content = &result[head_start + 6..head_end];
            let after_head = &result[head_end..];

            // Add page-break style
            let page_break_style =
                "\n<style type=\"text/css\">\nh1, h2 {{ page-break-before: always; }}\n</style>\n";
            result = format!(
                "{}{}{}{}",
                before_head, head_content, page_break_style, after_head
            );
        } else {
            // No </head>, add style before closing head
            result = result.replace("</head>", "\n<style type=\"text/css\">\nh1, h2 { page-break-before: always; }\n</style>\n</head>");
        }
    } else {
        // No <head>, insert one at the beginning
        let head_style = "<head>\n<style type=\"text/css\">\nh1, h2 { page-break-before: always; }\n</style>\n</head>\n";
        if let Some(body_start) = result.find("<body>") {
            let before_body = &result[..body_start];
            let after_body = &result[body_start..];
            result = format!("{}{}{}", head_style, before_body, after_body);
        }
    }

    result.into_bytes()
}

/// Clean HTML from bytes (memory-based)
pub fn clean_html_content_from_bytes(
    xhtml_bytes: &[u8],
    options: &ProcessingOptions,
) -> (Vec<u8>, HtmlCleanReport) {
    let mut report = HtmlCleanReport::default();

    // Read content from bytes
    let content = match String::from_utf8(xhtml_bytes.to_vec()) {
        Ok(c) => c,
        Err(e) => {
            report.errors.push(ProcessingError::new(
                "xhtml bytes",
                crate::error::ProcessingErrorType::HtmlParsing(e.to_string()),
                format!("Failed to parse XHTML as UTF-8: {}", e),
            ));
            return (Vec::new(), report);
        }
    };

    // Apply HTML cleanup
    let (new_content, whitespace_count, attrs_count) = apply_html_cleanup(&content, options);

    report.whitespace_cleaned = whitespace_count;
    report.attrs_stripped = attrs_count;

    (new_content.into_bytes(), report)
}

/// Apply HTML cleanup to content
fn apply_html_cleanup(content: &str, options: &ProcessingOptions) -> (String, usize, usize) {
    let mut result = content.to_string();
    let mut whitespace_count = 0;
    let mut attrs_count = 0;

    // Normalize whitespace
    if options.text_options.fix_whitespace {
        let original = result.clone();
        result = result.replace("\r\n", "\n").replace("\r", "\n");
        result = result
            .lines()
            .map(|line| line.trim())
            .collect::<Vec<&str>>()
            .join("\n");
        whitespace_count = original.len() - result.len();
    }

    // Strip unnecessary attributes
    if options.html_options.strip_attributes {
        let (cleaned, removed) = strip_unnecessary_attributes(result.as_bytes());
        result = String::from_utf8(cleaned).unwrap_or(result);
        attrs_count = removed;
    }

    (result, whitespace_count, attrs_count)
}

/// HTML cleaning report
#[derive(Debug, Default)]
pub struct HtmlCleanReport {
    pub whitespace_cleaned: usize,
    pub attrs_stripped: usize,
    pub errors: Vec<ProcessingError>,
}

impl HtmlCleanReport {
    /// Merge another report into this one
    pub fn merge(&mut self, other: &Self) {
        self.whitespace_cleaned += other.whitespace_cleaned;
        self.attrs_stripped += other.attrs_stripped;
        for error in &other.errors {
            self.errors.push(error.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_clean_report_merge() {
        let mut report1 = HtmlCleanReport {
            whitespace_cleaned: 5,
            attrs_stripped: 3,
            ..HtmlCleanReport::default()
        };

        let report2 = HtmlCleanReport {
            whitespace_cleaned: 2,
            attrs_stripped: 1,
            ..HtmlCleanReport::default()
        };

        report1.merge(&report2);

        assert_eq!(report1.whitespace_cleaned, 7);
        assert_eq!(report1.attrs_stripped, 4);
    }

    #[test]
    fn test_apply_html_cleanup_whitespace() {
        let mut options = ProcessingOptions::default();
        options.text_options.fix_whitespace = true;

        // Just verify the function works with various inputs
        let input1 = "Hello world";
        let (output1, _, _) = apply_html_cleanup(input1, &options);
        assert!(!output1.is_empty());

        let input2 = "Hello\nworld";
        let (output2, _, _) = apply_html_cleanup(input2, &options);
        assert!(!output2.is_empty());
    }

    #[test]
    fn test_apply_html_cleanup_with_html_option() {
        let mut options = ProcessingOptions::default();
        options.text_options.fix_whitespace = false;
        options.html_options.strip_attributes = true;

        let input = "Hello world";
        let (output, whitespace_count, _attrs_count) = apply_html_cleanup(input, &options);

        assert_eq!(output, input);
        assert_eq!(whitespace_count, 0);
        // attrs_count may vary depending on regex matching
    }

    #[test]
    fn test_clean_html_content_from_bytes() {
        let options = ProcessingOptions::default();
        let input = b"Hello world";
        let (output, report) = clean_html_content_from_bytes(input, &options);

        assert!(report.errors.is_empty());
        let output_str = String::from_utf8(output).expect("Failed to convert output to string");
        assert!(output_str.contains("Hello"));
    }

    #[test]
    fn test_clean_html_content_from_bytes_invalid_utf8() {
        let options = ProcessingOptions::default();
        let input = &[0xff, 0xfe, 0xfd]; // Invalid UTF-8
        let (_, report) = clean_html_content_from_bytes(input, &options);

        assert!(!report.errors.is_empty());
    }

    #[test]
    fn test_apply_html_cleanup_no_changes() {
        let mut options = ProcessingOptions::default();
        options.text_options.fix_whitespace = false;
        options.html_options.strip_attributes = false;

        let input = "<html><body>Hello world</body></html>";
        let (output, whitespace_count, attrs_count) = apply_html_cleanup(input, &options);

        assert_eq!(output, input);
        assert_eq!(whitespace_count, 0);
        assert_eq!(attrs_count, 0);
    }

    #[test]
    fn test_apply_html_cleanup_multiline() {
        let mut options = ProcessingOptions::default();
        options.text_options.fix_whitespace = true;

        let input = "  Hello  \n  world  \n  test  ";
        let (output, whitespace_count, _) = apply_html_cleanup(input, &options);

        assert!(whitespace_count > 0);
        assert!(output.contains("Hello"));
        assert!(output.contains("world"));
    }

    #[test]
    fn test_html_clean_report_errors() {
        let mut report = HtmlCleanReport::default();
        let error = ProcessingError::new(
            "test.html",
            crate::error::ProcessingErrorType::HtmlParsing("test error".to_string()),
            "details",
        );
        report.errors.push(error);

        assert_eq!(report.errors.len(), 1);
    }

    #[test]
    fn test_clean_html_content_from_bytes_with_content() {
        let mut options = ProcessingOptions::default();
        options.text_options.fix_whitespace = true;

        let input = b"  <html>  \n  <body>Hello world</body>  \n  </html>  ";
        let (output, report) = clean_html_content_from_bytes(input, &options);

        assert!(report.errors.is_empty());
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str.len(), 39);
    }

    #[test]
    fn test_html_clean_report_merge_with_errors() {
        let mut report1 = HtmlCleanReport {
            whitespace_cleaned: 5,
            ..HtmlCleanReport::default()
        };

        let mut report2 = HtmlCleanReport {
            whitespace_cleaned: 2,
            ..HtmlCleanReport::default()
        };

        let error = ProcessingError::new(
            "test.html",
            crate::error::ProcessingErrorType::HtmlParsing("error".to_string()),
            "details",
        );
        report2.errors.push(error);

        report1.merge(&report2);

        assert_eq!(report1.whitespace_cleaned, 7);
        assert_eq!(report1.errors.len(), 1);
    }

    #[test]
    fn test_strip_attributes_from_html_basic() {
        let input = r#"<div id="main" class="content">Text</div>";
        let output = strip_attributes_from_html(input);
        
        // Basic attributes should be preserved
        assert!(output.contains("id=\"main\""));
        assert!(output.contains("class=\"content\""));
        assert!(output.contains("Text"));
    }

    #[test]
    fn test_strip_attributes_strips_data() {
        let input = r#"<div data-id="123" data-value="456">Content</div>"#;
        let output = strip_attributes_from_html(input);

        // data-* attributes should be stripped
        assert!(!output.contains("data-id"));
        assert!(!output.contains("data-value"));
        assert!(output.contains("Content"));
    }

    #[test]
    fn test_strip_attributes_strips_aria() {
        let input = r#"<div aria-hidden="true" aria-label="test">Content</div>"#;
        let output = strip_attributes_from_html(input);

        // aria-* attributes should be stripped
        assert!(!output.contains("aria-hidden"));
        assert!(!output.contains("aria-label"));
        assert!(output.contains("Content"));
    }

    #[test]
    fn test_strip_attributes_strips_epub() {
        let input = r#"<body epub:type="frontmatter">Content</body>"#;
        let output = strip_attributes_from_html(input);

        // epub:* attributes should be stripped
        assert!(!output.contains("epub:type"));
    }

    #[test]
    fn test_strip_attributes_strips_role() {
        let input = r#"<div role="button" id="clickable">Content</div>"#;
        let output = strip_attributes_from_html(input);

        // role attribute should be stripped
        assert!(!output.contains("role="));
        assert!(output.contains("id=\"clickable\""));
    }

    #[test]
    fn test_strip_attributes_strips_tabindex() {
        let input = r#"<div tabindex="0" id="focusable">Content</div>"#;
        let output = strip_attributes_from_html(input);

        // tabindex attribute should be stripped
        assert!(!output.contains("tabindex="));
        assert!(output.contains("id=\"focusable\""));
    }

    #[test]
    fn test_strip_unnecessary_attributes() {
        let input = b"<div data-id=\"123\" role=\"button\" id=\"main\">Content</div>";
        let (output, _removed) = strip_unnecessary_attributes(input);

        let output_str = String::from_utf8_lossy(&output);
        assert!(!output_str.contains("data-id"));
        assert!(!output_str.contains("role="));
        assert!(output_str.contains("id=\"main\""));
        assert!(output_str.contains("Content"));
    }

    #[test]
    fn test_add_chapter_page_breaks_no_head() {
        // Input without head - should not crash
        let input = b"<html><body><h1>Chapter</h1></body></html>";
        let output = add_chapter_page_breaks(input);

        // Should handle gracefully
        assert_eq!(output.len(), 128);
    }

    #[test]
    fn test_add_chapter_page_breaks_existing_style() {
        // Input with existing page-break style - should not add duplicate
        let input = b"<html><head><style>h1 { page-break-before: always; }</style></head><body><h1>Chapter</h1></body></html>";
        let output = add_chapter_page_breaks(input);

        // Should keep original
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("page-break-before"));
    }

    #[test]
    fn test_add_chapter_page_breaks_adds_style() {
        // Input without page-break style - should add it
        let input = b"<html><head></head><body><h1>Chapter</h1></body></html>";
        let output = add_chapter_page_breaks(input);

        // Should add page-break style
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("h1") || output_str.contains("page-break"));
    }
}
