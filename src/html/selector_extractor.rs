//! Selector extractor - extract used selectors from HTML/XHTML content

use std::collections::HashSet;

/// Extracted used selectors from HTML
#[derive(Debug, Default)]
pub struct UsedSelectors {
    pub classes: HashSet<String>,
    pub ids: HashSet<String>,
    pub elements: HashSet<String>,
}

/// Extract used selectors from HTML content
pub fn extract_used_selectors(html_content: &str) -> UsedSelectors {
    let mut used = UsedSelectors::default();
    
    // Extract classes
    extract_classes(html_content, &mut used.classes);
    
    // Extract IDs
    extract_ids(html_content, &mut used.ids);
    
    // Extract element names
    extract_elements(html_content, &mut used.elements);
    
    used
}

/// Extract all class names from HTML
fn extract_classes(html: &str, classes: &mut HashSet<String>) {
    // Find all class attributes
    let class_prefix = "class=\"";
    let class_prefix2 = "class='";
    
    let mut pos = 0;
    
    while let Some(start) = html[pos..].find(class_prefix) {
        let start = pos + start + class_prefix.len();
        if let Some(end) = html[start..].find('"') {
            let class_value = &html[start..start + end];
            for class in class_value.split_whitespace() {
                classes.insert(class.to_string());
            }
            pos = start + end;
        } else {
            break;
        }
    }
    
    // Single quotes
    pos = 0;
    while let Some(start) = html[pos..].find(class_prefix2) {
        let start = pos + start + class_prefix2.len();
        if let Some(end) = html[start..].find('\'') {
            let class_value = &html[start..start + end];
            for class in class_value.split_whitespace() {
                classes.insert(class.to_string());
            }
            pos = start + end;
        } else {
            break;
        }
    }
}

/// Extract all ID names from HTML
fn extract_ids(html: &str, ids: &mut HashSet<String>) {
    let id_prefix = "id=\"";
    let id_prefix2 = "id='";
    
    let mut pos = 0;
    
    while let Some(start) = html[pos..].find(id_prefix) {
        let start = pos + start + id_prefix.len();
        if let Some(end) = html[start..].find('"') {
            let id_value = &html[start..start + end];
            ids.insert(id_value.to_string());
            pos = start + end;
        } else {
            break;
        }
    }
    
    // Single quotes
    pos = 0;
    while let Some(start) = html[pos..].find(id_prefix2) {
        let start = pos + start + id_prefix2.len();
        if let Some(end) = html[start..].find('\'') {
            let id_value = &html[start..start + end];
            ids.insert(id_value.to_string());
            pos = start + end;
        } else {
            break;
        }
    }
}

/// Extract all element names from HTML
fn extract_elements(html: &str, elements: &mut HashSet<String>) {
    // Look for opening tags
    let mut pos = 0;
    
    while let Some(start) = html[pos..].find('<') {
        let start = pos + start + 1;
        
        // Skip closing tags
        if html[start..].starts_with('/') {
            pos = start;
            continue;
        }
        
        // Skip comments, doctypes, etc.
        if html[start..].starts_with('!') || html[start..].starts_with('?') {
            pos = start;
            continue;
        }
        
        // Find the tag name
        let tag_end = html[start..].find(|c: char| c.is_whitespace() || c == '>' || c == '/').unwrap_or(0);
        
        if tag_end > 0 {
            let tag_name = &html[start..start + tag_end];
            
            // Only add if it looks like a valid tag name
            if tag_name.chars().all(|c| c.is_alphanumeric() || c == '-') {
                elements.insert(tag_name.to_string());
            }
        }
        
        pos = start + tag_end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_classes() {
        let html = r#"<div class="btn primary">Hello</div><span class="text-center">World</span>"#;
        let mut classes = HashSet::new();
        extract_classes(html, &mut classes);
        
        assert!(classes.contains("btn"));
        assert!(classes.contains("primary"));
        assert!(classes.contains("text-center"));
    }

    #[test]
    fn test_extract_ids() {
        let html = r#"<div id="main">Content</div>"#;
        let mut ids = HashSet::new();
        extract_ids(html, &mut ids);
        
        assert!(ids.contains("main"));
    }

    #[test]
    fn test_extract_elements() {
        let html = r#"<div><p>Hello</p><h1>Title</h1></div>"#;
        let mut elements = HashSet::new();
        extract_elements(html, &mut elements);
        
        assert!(elements.contains("div"));
        assert!(elements.contains("p"));
        assert!(elements.contains("h1"));
    }

    #[test]
    fn test_extract_used_selectors() {
        let html = r#"
            <div id="container" class="main-content">
                <p class="text-center">Hello</p>
                <h1 id="title">Title</h1>
            </div>
        "#;
        
        let used = extract_used_selectors(html);
        
        assert!(used.classes.contains("main-content"));
        assert!(used.classes.contains("text-center"));
        assert!(used.ids.contains("container"));
        assert!(used.ids.contains("title"));
        assert!(used.elements.contains("div"));
        assert!(used.elements.contains("p"));
        assert!(used.elements.contains("h1"));
    }

    #[test]
    fn test_extract_classes_with_single_quotes() {
        let html = r#"<div class='btn primary'>Hello</div>"#;
        let mut classes = HashSet::new();
        extract_classes(html, &mut classes);
        
        assert!(classes.contains("btn"));
        assert!(classes.contains("primary"));
    }
}
