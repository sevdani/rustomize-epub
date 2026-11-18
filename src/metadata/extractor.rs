//! Metadata extractor
use std::collections::HashMap;

/// Extracted metadata from EPUB
#[derive(Debug, Default)]
pub struct ExtractedMetadata {
    pub title: Option<String>,
    pub creator: Option<String>,
    pub contributor: Option<String>,
    pub publisher: Option<String>,
    pub date: Option<String>,
    pub identifier: Option<String>,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub rights: Option<String>,
    pub format: Option<String>,
    pub type_: Option<String>,
    pub coverage: Option<String>,
    pub relation: Option<String>,
    pub source: Option<String>,
    pub series: Option<String>,
    pub series_index: Option<String>,
    pub isbn: Option<String>,
}

/// Extract a Dublin Core tag value
fn extract_dc_tag(content: &str, tag: &str) -> Option<String> {
    let tag_start = format!("<dc:{}>", tag);
    let tag_end = format!("</dc:{}>", tag);
    
    if let Some(start) = content.find(&tag_start) {
        let start = start + tag_start.len();
        if let Some(end) = content[start..].find(&tag_end) {
            return Some(content[start..start + end].trim().to_string());
        }
    }
    None
}

/// Extract a meta tag value
fn extract_meta_tag(content: &str, name: &str) -> Option<String> {
    let name_pattern = format!("name=\"{}\"", name);
    if let Some(start) = content.find(&name_pattern) {
        // Find content attribute
        let content_pattern = "content=\"";
        if let Some(content_start) = content[start..].find(content_pattern) {
            let content_start = content_start + content_pattern.len();
            if let Some(content_end) = content[start + content_start..].find('"') {
                return Some(content[start + content_start..start + content_start + content_end].to_string());
            }
        }
    }
    None
}
