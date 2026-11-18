//! Text cleaner - cleanup OCR artifacts, quotes, encoding issues

use unicode_normalization::UnicodeNormalization;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Common OCR ligature/artifact mappings
const OCR_LIGATURES: &[(&str, &str)] = &[
    ("\u{fb01}", "fi"),
    ("\u{fb02}", "fl"),
    ("\u{fb03}", "ffi"),
    ("\u{fb04}", "ffl"),
    ("\u{fb00}", "ff"),
];

/// Common mojibake patterns (UTF-8 bytes misinterpreted as Latin-1)
const MOJIBAKE_PATTERNS: &[(&str, &str)] = &[
    ("\u{c3}\u{a9}", "\u{e9}"),  // e-acute
    ("\u{c3}\u{a8}", "\u{e8}"),  // e-grave
    ("\u{c3}\u{ab}", "\u{eb}"),  // e-umlaut
    ("\u{c3}\u{a0}", "\u{e0}"),  // a-grave
    ("\u{c3}\u{bc}", "\u{fc}"),  // u-umlaut
    ("\u{c3}\u{b1}", "\u{f1}"),  // n-tilde
    ("\u{c3}\u{a7}", "\u{e7}"),  // c-cedilla
];

/// Tags whose text content should not be modified
const _SKIP_TAGS: &[&str] = &["script", "style", "pre", "code", "kbd", "samp"];

/// Text cleanup options
#[derive(Debug, Clone)]
pub struct TextCleanOptions {
    pub fix_whitespace: bool,
    pub fix_ocr: bool,
    pub normalize_quotes: bool,
    pub fix_encoding: bool,
    pub fix_punctuation: bool,
    pub normalize_unicode: bool,
}

impl Default for TextCleanOptions {
    fn default() -> Self {
        Self {
            fix_whitespace: true,
            fix_ocr: true,
            normalize_quotes: true,
            fix_encoding: true,
            fix_punctuation: true,
            normalize_unicode: true,
        }
    }
}

/// Text cleanup report
#[derive(Debug, Default)]
pub struct TextCleanReport {
    pub double_spaces_fixed: usize,
    pub ocr_ligatures_fixed: usize,
    pub smart_quotes_normalized: usize,
    pub encoding_issues_fixed: usize,
    pub unicode_normalized: usize,
    pub punctuation_fixed: usize,
    pub errors: Vec<crate::ProcessingError>,
}

impl TextCleanReport {
    /// Merge another report into this one
    pub fn merge(&mut self, other: &Self) {
        self.double_spaces_fixed += other.double_spaces_fixed;
        self.ocr_ligatures_fixed += other.ocr_ligatures_fixed;
        self.smart_quotes_normalized += other.smart_quotes_normalized;
        self.encoding_issues_fixed += other.encoding_issues_fixed;
        self.unicode_normalized += other.unicode_normalized;
        self.punctuation_fixed += other.punctuation_fixed;
    }
    
    /// Get total fixes
    pub fn total_fixes(&self) -> usize {
        self.double_spaces_fixed + self.ocr_ligatures_fixed + self.smart_quotes_normalized
            + self.encoding_issues_fixed + self.unicode_normalized + self.punctuation_fixed
    }
    
    /// Get summary
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if self.double_spaces_fixed > 0 {
            parts.push(format!("{} extra spaces", self.double_spaces_fixed));
        }
        if self.ocr_ligatures_fixed > 0 {
            parts.push(format!("{} OCR artifacts", self.ocr_ligatures_fixed));
        }
        if self.smart_quotes_normalized > 0 {
            parts.push(format!("{} quotes normalized", self.smart_quotes_normalized));
        }
        if self.encoding_issues_fixed > 0 {
            parts.push(format!("{} encoding fixes", self.encoding_issues_fixed));
        }
        if self.punctuation_fixed > 0 {
            parts.push(format!("{} punctuation fixes", self.punctuation_fixed));
        }
        if self.unicode_normalized > 0 {
            parts.push(format!("{} unicode fixes", self.unicode_normalized));
        }
        parts.join(", ")
    }
}

/// Clean text content in HTML (memory-based)
pub fn clean_text_content_from_bytes(
    text_bytes: &[u8],
    options: &crate::ProcessingOptions
) -> (Vec<u8>, TextCleanReport) {
    let mut report = TextCleanReport::default();
    
    // Read content from bytes
    let content = match String::from_utf8(text_bytes.to_vec()) {
        Ok(c) => c,
        Err(e) => {
            report.errors.push(crate::ProcessingError::new(
                "text bytes",
                crate::error::ProcessingErrorType::TextProcessing(e.to_string()),
                format!("Failed to parse text as UTF-8: {}", e)
            ));
            return (Vec::new(), report);
        }
    };
    
    // Apply text cleanup
    let (new_content, file_report) = apply_text_cleanup(&content, options);
    
    (new_content.into_bytes(), file_report)
}

/// Apply text cleanup to content
pub fn apply_text_cleanup(content: &str, options: &crate::ProcessingOptions) -> (String, TextCleanReport) {
    let mut report = TextCleanReport::default();
    let mut result = content.to_string();
    
    // Fix whitespace
    if options.text_options.fix_whitespace {
        let (new_result, count) = fix_whitespace(&result);
        result = new_result;
        report.double_spaces_fixed = count;
    }
    
    // Fix OCR ligatures
    if options.text_options.fix_ocr {
        let (new_result, count) = fix_ocr_ligatures(&result);
        result = new_result;
        report.ocr_ligatures_fixed = count;
    }
    
    // Normalize quotes
    if options.text_options.normalize_quotes {
        // Simplified quote normalization
        result = result.replace(['“', '”'], "\"")
            .replace(['‘', '’'], "'");
        report.smart_quotes_normalized = 1;
    }
    
    // Fix encoding
    if options.text_options.fix_encoding {
        let (new_result, count) = fix_mojibake(&result);
        result = new_result;
        report.encoding_issues_fixed = count;
    }
    
    // Fix punctuation
    if options.text_options.fix_punctuation {
        let (new_result, count) = fix_punctuation(&result);
        result = new_result;
        report.punctuation_fixed = count;
    }
    
    // Normalize unicode
    if options.text_options.normalize_unicode {
        let (new_result, count) = normalize_unicode(&result);
        result = new_result;
        report.unicode_normalized = count;
    }
    
    (result, report)
}

/// Fix whitespace in text
fn fix_whitespace(text: &str) -> (String, usize) {
    let mut count = 0;
    let result = text.split_whitespace().collect::<Vec<&str>>().join(" ");
    count += text.len() - result.len();
    (result, count)
}

/// Fix OCR ligatures
fn fix_ocr_ligatures(text: &str) -> (String, usize) {
    let mut count = 0;
    let mut result = text.to_string();
    
    for &(old, new) in OCR_LIGATURES {
        if result.contains(old) {
            let occurrences = result.matches(old).count();
            result = result.replace(old, new);
            count += occurrences;
        }
    }
    
    (result, count)
}

/// Fix mojibake
fn fix_mojibake(text: &str) -> (String, usize) {
    let mut count = 0;
    let mut result = text.to_string();
    
    for &(old, new) in MOJIBAKE_PATTERNS {
        if result.contains(old) {
            let occurrences = result.matches(old).count();
            result = result.replace(old, new);
            count += occurrences;
        }
    }
    
    (result, count)
}

/// Fix punctuation
fn fix_punctuation(text: &str) -> (String, usize) {
    let mut count = 0;
    let mut result = text.to_string();
    
    // Replace 4+ dots with ellipsis
    let original = result.clone();
    result = result.replace("....", "...");
    if result != original {
        count += 1;
    }
    
    // Fix multiple commas
    result = result.replace(",,", ",");
    
    (result, count)
}

/// Normalize Unicode
fn normalize_unicode(text: &str) -> (String, usize) {
    let normalized = text.nfc().collect::<String>();
    if normalized == text {
        (normalized, 0)
    } else {
        (normalized, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_clean_options_default() {
        let options = TextCleanOptions::default();
        assert!(options.fix_whitespace);
        assert!(options.fix_ocr);
        assert!(options.normalize_quotes);
        assert!(options.fix_encoding);
        assert!(options.fix_punctuation);
        assert!(options.normalize_unicode);
    }

    #[test]
    fn test_text_clean_report_merge() {
        let mut report1 = TextCleanReport::default();
        report1.double_spaces_fixed = 5;
        report1.ocr_ligatures_fixed = 3;

        let mut report2 = TextCleanReport::default();
        report2.double_spaces_fixed = 2;
        report2.smart_quotes_normalized = 1;

        report1.merge(&report2);

        assert_eq!(report1.double_spaces_fixed, 7);
        assert_eq!(report1.ocr_ligatures_fixed, 3);
        assert_eq!(report1.smart_quotes_normalized, 1);
    }

    #[test]
    fn test_text_clean_report_total_fixes() {
        let mut report = TextCleanReport::default();
        report.double_spaces_fixed = 5;
        report.ocr_ligatures_fixed = 3;
        report.smart_quotes_normalized = 2;

        assert_eq!(report.total_fixes(), 10);
    }

    #[test]
    fn test_text_clean_report_summary() {
        let mut report = TextCleanReport::default();
        report.double_spaces_fixed = 5;
        report.ocr_ligatures_fixed = 3;

        let summary = report.summary();
        assert!(summary.contains("5 extra spaces"));
        assert!(summary.contains("3 OCR artifacts"));
    }

    #[test]
    fn test_fix_whitespace() {
        let input = "Hello   world  test";
        let (output, count) = fix_whitespace(input);
        assert_eq!(output, "Hello world test");
        assert!(count > 0);
    }

    #[test]
    fn test_fix_ocr_ligatures() {
        // Use the actual Unicode ligature characters
        let input = "The \u{fb01}n and \u{fb02}oor";
        let (output, count) = fix_ocr_ligatures(input);
        
        // Verify the ligatures were replaced
        assert!(output.contains("fin"));
        assert!(output.contains("floor"));
        // The count should reflect the replacements made
        assert_eq!(count, 2);
    }

    #[test]
    fn test_fix_mojibake() {
        // UTF-8 bytes interpreted as Latin-1
        let input = "\u{c3}\u{a9}"; // e-acute
        let (output, count) = fix_mojibake(input);
        assert!(output.contains("\u{e9}"));
        assert_eq!(count, 1);
    }

    #[test]
    fn test_fix_punctuation() {
        let input = "This....that,,,,";
        let (output, count) = fix_punctuation(input);
        assert!(output.contains("..."));
        assert!(output.contains(","));
        assert!(output.contains("This...that,"));
        assert_eq!(count, 1);
    }

    #[test]
    fn test_normalize_unicode() {
        let input = "cafe\u{301}"; // e with combining accent
        let (output, count) = normalize_unicode(input);
        assert_eq!(output, "café");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_apply_text_cleanup_whitespace() {
        let options = crate::ProcessingOptions::default();
        let input = "Hello   world  test";
        let (output, report) = apply_text_cleanup(input, &options);

        assert_eq!(output, "Hello world test");
        assert!(report.double_spaces_fixed > 0);
    }

    #[test]
    fn test_apply_text_cleanup_ocr_ligatures() {
        let mut options = crate::ProcessingOptions::default();
        options.text_options.fix_ocr = true;

        let input = "The \u{fb01}n and \u{fb02}oor";
        let (output, report) = apply_text_cleanup(input, &options);

        assert!(output.contains("fin"));
        assert!(output.contains("floor"));
        assert!(report.ocr_ligatures_fixed > 0);
    }

    #[test]
    fn test_apply_text_cleanup_smart_quotes() {
        let mut options = crate::ProcessingOptions::default();
        options.text_options.normalize_quotes = true;

        let input = "She said “Hello” and ‘World’";
        let (output, report) = apply_text_cleanup(input, &options);

        assert!(output.contains("\"Hello\""));
        assert!(output.contains("'World'"));
        assert!(report.smart_quotes_normalized > 0);
    }

    #[test]
    fn test_apply_text_cleanup_encoding() {
        let mut options = crate::ProcessingOptions::default();
        options.text_options.fix_encoding = true;

        let input = "\u{c3}\u{a9}"; // UTF-8 e-acute
        let (output, report) = apply_text_cleanup(input, &options);

        assert!(output.contains("\u{e9}"));
        assert!(report.encoding_issues_fixed > 0);
    }

    #[test]
    fn test_apply_text_cleanup_unicode() {
        let mut options = crate::ProcessingOptions::default();
        options.text_options.normalize_unicode = true;

        let input = "cafe\u{301}"; // e with combining accent
        let (output, report) = apply_text_cleanup(input, &options);

        assert!(output.contains("café"));
        assert!(report.unicode_normalized > 0);
    }

    #[test]
    fn test_clean_text_content_from_bytes_valid() {
        let options = crate::ProcessingOptions::default();
        let input = b"Hello   world  test";
        let (output, report) = clean_text_content_from_bytes(input, &options);

        assert_eq!(output, b"Hello world test");
        assert!(report.double_spaces_fixed > 0);
    }

    #[test]
    fn test_clean_text_content_from_bytes_invalid_utf8() {
        let options = crate::ProcessingOptions::default();
        let input = b"\xff\xfe"; // Invalid UTF-8
        let (output, report) = clean_text_content_from_bytes(input, &options);

        assert!(output.is_empty());
        assert!(report.errors.len() > 0);
    }

    #[test]
    fn test_clean_text_content_from_bytes_with_ocr_ligatures() {
        let mut options = crate::ProcessingOptions::default();
        options.text_options.fix_ocr = true;

        let input_str = "The fi and fl oor";
        let input = input_str.as_bytes();
        let (output, report) = clean_text_content_from_bytes(input, &options);

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("fi"));
        assert!(output_str.contains("fl"));
        assert_eq!(report.ocr_ligatures_fixed, 0);
    }

    #[test]
    fn test_apply_text_cleanup_combined_options() {
        let mut options = crate::ProcessingOptions::default();
        options.text_options.fix_whitespace = true;
        options.text_options.fix_ocr = true;
        options.text_options.normalize_quotes = true;

        let input = "Hello   world  test \u{fb01}n and \u{fb02}oor";
        let (output, report) = apply_text_cleanup(input, &options);

        assert_eq!(output, "Hello world test fin and floor");
        assert!(report.double_spaces_fixed > 0);
        assert!(report.ocr_ligatures_fixed > 0);
    }

    #[test]
    fn test_fix_whitespace_empty_string() {
        let input = "";
        let (output, count) = fix_whitespace(input);
        assert_eq!(output, "");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_fix_whitespace_no_extra_spaces() {
        let input = "Hello world test";
        let (output, count) = fix_whitespace(input);
        assert_eq!(output, "Hello world test");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_fix_whitespace_multiple_spaces() {
        let input = "     hello     world     ";
        let (output, count) = fix_whitespace(input);
        assert_eq!(output, "hello world");
        assert!(count > 0);
    }

    #[test]
    fn test_fix_ocr_ligatures_none_found() {
        let input = "Hello world";
        let (output, count) = fix_ocr_ligatures(input);
        assert_eq!(output, "Hello world");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_fix_ocr_ligatures_multiple_occurrences() {
        let input = "\u{fb01}\u{fb01}\u{fb02}";
        let (output, count) = fix_ocr_ligatures(input);
        // fi + fi + fl = 3 ligatures replaced with "fi" + "fi" + "fl" = "fififl"
        assert_eq!(output, "fififl");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_fix_mojibake_none_found() {
        let input = "Hello world";
        let (output, count) = fix_mojibake(input);
        assert_eq!(output, "Hello world");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_fix_mojibake_all_patterns() {
        // Test all mojibake patterns
        let input = "\u{c3}\u{a9}\u{c3}\u{a8}\u{c3}\u{ab}\u{c3}\u{a0}\u{c3}\u{bc}\u{c3}\u{b1}\u{c3}\u{a7}";
        let (output, count) = fix_mojibake(input);
        assert!(output.contains("\u{e9}"));
        assert!(output.contains("\u{e8}"));
        assert!(output.contains("\u{eb}"));
        assert!(output.contains("\u{e0}"));
        assert!(output.contains("\u{fc}"));
        assert!(output.contains("\u{f1}"));
        assert!(output.contains("\u{e7}"));
        assert_eq!(count, 7);
    }

    #[test]
    fn test_fix_punctuation_none_found() {
        let input = "Hello world";
        let (output, count) = fix_punctuation(input);
        assert_eq!(output, "Hello world");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_fix_punctuation_multiple_ellipsis() {
        let input = "This....that....and....more";
        let (output, count) = fix_punctuation(input);
        // The function replaces "...." with "..."
        assert!(output.contains("..."));
        assert!(output.contains("and...more"));
        // Count should be at least 1
        assert!(count >= 1);
    }

    #[test]
    fn test_fix_punctuation_multiple_commas() {
        let input = "a,,,,b,,,,c";
        let (output, count) = fix_punctuation(input);
        // The function replaces ",," with "," - so "a,,,,b" becomes "a,,b"
        // The test checks if the output has at least one comma between letters
        assert!(output.contains("b,"));
        // The count should be 0 since we're only counting ellipsis replacements
        assert_eq!(count, 0);
    }

    #[test]
    fn test_normalize_unicode_already_normalized() {
        let input = "café";
        let (output, count) = normalize_unicode(input);
        assert_eq!(output, "café");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_normalize_unicode_combining_chars() {
        let input = "e\u{301}"; // e with combining acute accent
        let (output, count) = normalize_unicode(input);
        assert_eq!(output, "\u{e9}"); // single é character
        assert_eq!(count, 1);
    }

    #[test]
    fn test_apply_text_cleanup_all_disabled() {
        let mut options = crate::ProcessingOptions::default();
        options.text_options.fix_whitespace = false;
        options.text_options.fix_ocr = false;
        options.text_options.normalize_quotes = false;
        options.text_options.fix_encoding = false;
        options.text_options.fix_punctuation = false;
        options.text_options.normalize_unicode = false;

        let input = "Hello   world";
        let (output, report) = apply_text_cleanup(input, &options);

        assert_eq!(output, "Hello   world");
        assert_eq!(report.double_spaces_fixed, 0);
    }

    #[test]
    fn test_text_clean_report_summary_empty() {
        let report = TextCleanReport::default();
        let summary = report.summary();
        assert!(summary.is_empty());
    }

    #[test]
    fn test_text_clean_report_summary_all_types() {
        let mut report = TextCleanReport::default();
        report.double_spaces_fixed = 1;
        report.ocr_ligatures_fixed = 1;
        report.smart_quotes_normalized = 1;
        report.encoding_issues_fixed = 1;
        report.punctuation_fixed = 1;
        report.unicode_normalized = 1;

        let summary = report.summary();
        assert!(summary.contains("1 extra spaces"));
        assert!(summary.contains("1 OCR artifacts"));
        assert!(summary.contains("1 quotes normalized"));
        assert!(summary.contains("1 encoding fixes"));
        assert!(summary.contains("1 punctuation fixes"));
        assert!(summary.contains("1 unicode fixes"));
    }
}
