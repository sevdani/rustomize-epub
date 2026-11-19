//! Metadata editor - applies user edits to EPUB metadata

use crate::types::MetadataEdits;

/// Apply metadata edits to OPF content
/// Overwrites dc:title, dc:creator, dc:series, dc:language, dc:series-index if provided
pub fn apply_metadata_edits(opf_content: &str, edits: &MetadataEdits) -> String {
    let mut new_content = opf_content.to_string();
    
    // Apply title edit
    if let Some(title) = &edits.title {
        new_content = apply_dc_tag_edit(&new_content, "title", title);
    }
    
    // Apply author edit (dc:creator)
    if let Some(author) = &edits.author {
        new_content = apply_dc_tag_edit(&new_content, "creator", author);
    }
    
    // Apply series edit
    if let Some(series) = &edits.series {
        new_content = apply_dc_tag_edit(&new_content, "series", series);
    }
    
    // Apply series index edit (dc:series-index or meta tag)
    if let Some(index) = &edits.series_index {
        new_content = apply_series_index_edit(&new_content, index);
    }
    
    // Apply language edit
    if let Some(lang) = &edits.language {
        new_content = apply_dc_tag_edit(&new_content, "language", lang);
    }
    
    new_content
}

/// Apply an edit to a Dublin Core tag - replaces existing tag or adds new one
fn apply_dc_tag_edit(content: &str, tag_name: &str, new_value: &str) -> String {
    let tag_start = format!("<dc:{}>", tag_name);
    let tag_end = format!("</dc:{}>", tag_name);
    
    let mut result = content.to_string();
    
    // Find and replace existing tag
    if let Some(start) = result.find(&tag_start) {
        // Find the end of the opening tag
        if let Some(tag_end_pos) = result[start..].find('>') {
            let content_start = start + tag_end_pos + 1;
            
            // Find the closing tag
            if let Some(closing) = result[content_start..].find(&tag_end) {
                let content_end = content_start + closing + tag_end.len();
                
                // Replace the entire tag
                let before = &result[..start];
                let after = &result[content_end..];
                result = format!("{}<dc:{}>{}</dc:{}>{}", before, tag_name, new_value, tag_name, after);
            }
        }
    } else {
        // Tag doesn't exist, need to add it in metadata section
        // Find the metadata section and add before </metadata>
        if let Some(metadata_end) = result.find("</metadata>") {
            let insert_pos = metadata_end;
            let before = &result[..insert_pos];
            let after = &result[insert_pos..];
            result = format!("{}<dc:{}>{}</dc:{}>{}", before, tag_name, new_value, tag_name, after);
        }
    }
    
    result
}

/// Apply series index edit (dc:series-index or meta tag)
fn apply_series_index_edit(content: &str, new_index: &str) -> String {
    let mut result = content.to_string();
    
    // Try dc:series-index first
    result = apply_dc_tag_edit(&result, "series-index", new_index);
    
    // Also handle meta property="book-series-index"
    result = apply_meta_property_edit(&result, "book-series-index", new_index);
    
    result
}

/// Apply edit to a meta property tag
fn apply_meta_property_edit(content: &str, property: &str, new_value: &str) -> String {
    let attr_start = format!("property=\"{}\"", property);
    
    // Find meta tags with this property
    if let Some(start) = content.find(&attr_start) {
        // Find the start of this tag (backwards to find <)
        let tag_start_pos = content[..start].rfind('<').unwrap_or(0);
        
        // Find the end of the tag
        if let Some(end) = content[tag_start_pos..].find('>') {
            let full_end = tag_start_pos + end + 1;
            
            // Build new tag
            let new_tag = format!(r#"<meta property="{}" content="{}"/>"#, property, new_value);
            let before = &content[..tag_start_pos];
            let after = &content[full_end..];
            return format!("{}{}{}", before, new_tag, after);
        }
    }
    
    // Tag doesn't exist, add it in metadata section
    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_metadata_edits_title_only() {
        let opf_content = r#"<?xml version="1.0"?>
<package unique-identifier="uuid_id" version="2.0">
  <metadata>
    <dc:title>Old Title</dc:title>
    <dc:creator>Old Author</dc:creator>
  </metadata>
</package>"#;
        
        let edits = MetadataEdits {
            title: Some("New Title".to_string()),
            author: None,
            series: None,
            series_index: None,
            language: None,
        };
        
        let result = apply_metadata_edits(opf_content, &edits);
        assert!(result.contains("<dc:title>New Title</dc:title>"));
        assert!(result.contains("<dc:creator>Old Author</dc:creator>"));
    }

    #[test]
    fn test_apply_metadata_edits_author_only() {
        let opf_content = r#"<?xml version="1.0"?>
<package unique-identifier="uuid_id" version="2.0">
  <metadata>
    <dc:title>Old Title</dc:title>
    <dc:creator>Old Author</dc:creator>
  </metadata>
</package>"#;
        
        let edits = MetadataEdits {
            title: None,
            author: Some("New Author".to_string()),
            series: None,
            series_index: None,
            language: None,
        };
        
        let result = apply_metadata_edits(opf_content, &edits);
        assert!(result.contains("<dc:title>Old Title</dc:title>"));
        assert!(result.contains("<dc:creator>New Author</dc:creator>"));
    }

    #[test]
    fn test_apply_metadata_edits_both() {
        let opf_content = r#"<?xml version="1.0"?>
<package unique-identifier="uuid_id" version="2.0">
  <metadata>
    <dc:title>Old Title</dc:title>
    <dc:creator>Old Author</dc:creator>
  </metadata>
</package>"#;
        
        let edits = MetadataEdits {
            title: Some("New Title".to_string()),
            author: Some("New Author".to_string()),
            series: None,
            series_index: None,
            language: None,
        };
        
        let result = apply_metadata_edits(opf_content, &edits);
        assert!(result.contains("<dc:title>New Title</dc:title>"));
        assert!(result.contains("<dc:creator>New Author</dc:creator>"));
    }

    #[test]
    fn test_apply_metadata_edits_series() {
        let opf_content = r#"<?xml version="1.0"?>
<package unique-identifier="uuid_id" version="2.0">
  <metadata>
    <dc:title>Book Title</dc:title>
    <dc:creator>Author</dc:creator>
    <dc:series>Old Series</dc:series>
  </metadata>
</package>"#;
        
        let edits = MetadataEdits {
            title: None,
            author: None,
            series: Some("New Series".to_string()),
            series_index: None,
            language: None,
        };
        
        let result = apply_metadata_edits(opf_content, &edits);
        assert!(result.contains("<dc:series>New Series</dc:series>"));
    }

    #[test]
    fn test_apply_metadata_edits_language() {
        let opf_content = r#"<?xml version="1.0"?>
<package unique-identifier="uuid_id" version="2.0">
  <metadata>
    <dc:title>Book Title</dc:title>
    <dc:creator>Author</dc:creator>
    <dc:language>old</dc:language>
  </metadata>
</package>"#;
        
        let edits = MetadataEdits {
            title: None,
            author: None,
            series: None,
            series_index: None,
            language: Some("en".to_string()),
        };
        
        let result = apply_metadata_edits(opf_content, &edits);
        assert!(result.contains("<dc:language>en</dc:language>"));
    }

    #[test]
    fn test_apply_metadata_edits_all_fields() {
        let opf_content = r#"<?xml version="1.0"?>
<package unique-identifier="uuid_id" version="2.0">
  <metadata>
    <dc:title>Old Title</dc:title>
    <dc:creator>Old Author</dc:creator>
    <dc:series>Old Series</dc:series>
    <dc:series-index>1</dc:series-index>
    <dc:language>old</dc:language>
  </metadata>
</package>"#;
        
        let edits = MetadataEdits {
            title: Some("New Title".to_string()),
            author: Some("New Author".to_string()),
            series: Some("New Series".to_string()),
            series_index: Some("2".to_string()),
            language: Some("en".to_string()),
        };
        
        let result = apply_metadata_edits(opf_content, &edits);
        assert!(result.contains("<dc:title>New Title</dc:title>"));
        assert!(result.contains("<dc:creator>New Author</dc:creator>"));
        assert!(result.contains("<dc:series>New Series</dc:series>"));
        assert!(result.contains("<dc:series-index>2</dc:series-index>"));
        assert!(result.contains("<dc:language>en</dc:language>"));
    }

    #[test]
    fn test_apply_metadata_edits_preserves_other_metadata() {
        let opf_content = r#"<?xml version="1.0"?>
<package unique-identifier="uuid_id" version="2.0">
  <metadata>
    <dc:title>Old Title</dc:title>
    <dc:creator>Old Author</dc:creator>
    <dc:publisher>Old Publisher</dc:publisher>
    <dc:date>2020-01-01</dc:date>
  </metadata>
</package>"#;
        
        let edits = MetadataEdits {
            title: Some("New Title".to_string()),
            author: Some("New Author".to_string()),
            series: None,
            series_index: None,
            language: None,
        };
        
        let result = apply_metadata_edits(opf_content, &edits);
        assert!(result.contains("<dc:publisher>Old Publisher</dc:publisher>"));
        assert!(result.contains("<dc:date>2020-01-01</dc:date>"));
    }

    #[test]
    fn test_apply_metadata_edits_missing_tags() {
        let opf_content = r#"<?xml version="1.0"?>
<package unique-identifier="uuid_id" version="2.0">
  <metadata>
    <dc:publisher>Publisher</dc:publisher>
  </metadata>
</package>"#;
        
        let edits = MetadataEdits {
            title: Some("New Title".to_string()),
            author: Some("New Author".to_string()),
            series: None,
            series_index: None,
            language: None,
        };
        
        let result = apply_metadata_edits(opf_content, &edits);
        // Tags should be added before </metadata>
        assert!(result.contains("<dc:title>New Title</dc:title>"));
        assert!(result.contains("<dc:creator>New Author</dc:creator>"));
    }

    #[test]
    fn test_apply_series_index_edit_meta_tag() {
        let opf_content = r#"<?xml version="1.0"?>
<package unique-identifier="uuid_id" version="2.0">
  <metadata>
    <dc:title>Book</dc:title>
    <meta property="book-series-index">1</meta>
  </metadata>
</package>"#;
        
        let result = apply_series_index_edit(opf_content, "2");
        assert!(result.contains(r#"<meta property="book-series-index" content="2"/>"#));
    }
}
