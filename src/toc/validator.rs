//! TOC validator - validate NCX (EPUB2) and NAV (EPUB3) TOC files

use std::collections::HashSet;

/// Result of TOC validation
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<ValidationError>,
    pub errors: Vec<ValidationError>,
    pub nav_point_count: usize,
}

/// Error level for validation issues
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorLevel {
    Warning,
    Error,
    Fatal,
}

/// A validation error
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub level: ErrorLevel,
    pub message: String,
    pub location: Option<String>,
}

/// Validate NCX (EPUB2) TOC content
pub fn validate_ncx(ncx_content: &str, xhtml_files: &[String]) -> ValidationResult {
    let mut result = ValidationResult {
        is_valid: true,
        warnings: Vec::new(),
        errors: Vec::new(),
        nav_point_count: 0,
    };

    // Check basic XML structure
    if !ncx_content.contains("<ncx") || !ncx_content.contains("</ncx>") {
        result.errors.push(ValidationError {
            level: ErrorLevel::Error,
            message: "NCX is missing root <ncx> element".to_string(),
            location: None,
        });
        result.is_valid = false;
        return result;
    }

    // Check required elements
    if !ncx_content.contains("<head>") {
        result.errors.push(ValidationError {
            level: ErrorLevel::Error,
            message: "NCX is missing <head> element".to_string(),
            location: None,
        });
        result.is_valid = false;
    }

    if !ncx_content.contains("<docTitle>") {
        result.errors.push(ValidationError {
            level: ErrorLevel::Error,
            message: "NCX is missing <docTitle> element".to_string(),
            location: None,
        });
        result.is_valid = false;
    }

    if !ncx_content.contains("<docAuthor>") {
        result.errors.push(ValidationError {
            level: ErrorLevel::Error,
            message: "NCX is missing <docAuthor> element".to_string(),
            location: None,
        });
        result.is_valid = false;
    }

    if !ncx_content.contains("<navMap>") {
        result.errors.push(ValidationError {
            level: ErrorLevel::Error,
            message: "NCX is missing <navMap> element".to_string(),
            location: None,
        });
        result.is_valid = false;
        return result;
    }

    // Count navigation points
    let nav_points = extract_nav_points(ncx_content);
    result.nav_point_count = nav_points.len();

    if result.nav_point_count == 0 {
        result.errors.push(ValidationError {
            level: ErrorLevel::Error,
            message: "NCX <navMap> contains no navigation points".to_string(),
            location: None,
        });
        result.is_valid = false;
        return result;
    }

    // Validate navigation points
    let mut seen_ids = HashSet::new();
    for point in nav_points.iter() {
        // Check for duplicate IDs
        if seen_ids.contains(&point.id) {
            result.errors.push(ValidationError {
                level: ErrorLevel::Error,
                message: format!("Duplicate navPoint ID: {}", point.id),
                location: Some(point.id.clone()),
            });
            result.is_valid = false;
        }
        seen_ids.insert(point.id.clone());

        // Check for content reference
        if point.content_src.is_empty() {
            result.errors.push(ValidationError {
                level: ErrorLevel::Error,
                message: format!("navPoint '{}' has no content reference", point.id),
                location: Some(point.id.clone()),
            });
            result.is_valid = false;
        }

        // Check if content reference points to valid file
        if !point.content_src.is_empty()
            && !xhtml_files.iter().any(|f| {
                // Check if the path matches
                f.ends_with(&point.content_src) || f.contains(&point.content_src)
            })
        {
            result.warnings.push(ValidationError {
                level: ErrorLevel::Warning,
                message: format!(
                    "navPoint '{}' references non-existent file: {}",
                    point.id, point.content_src
                ),
                location: Some(point.id.clone()),
            });
        }
    }

    // Check playOrder sequence
    let play_orders: Vec<i32> = nav_points.iter().map(|p| p.play_order).collect();
    for i in 1..play_orders.len() {
        if play_orders[i] <= play_orders[i - 1] {
            result.warnings.push(ValidationError {
                level: ErrorLevel::Warning,
                message: format!(
                    "Non-sequential playOrder at navPoint index {}: {} <= {}",
                    i,
                    play_orders[i],
                    play_orders[i - 1]
                ),
                location: None,
            });
        }
    }

    result
}

/// Validate NAV (EPUB3) TOC content
pub fn validate_nav(nav_content: &str, xhtml_files: &[String]) -> ValidationResult {
    let mut result = ValidationResult {
        is_valid: true,
        warnings: Vec::new(),
        errors: Vec::new(),
        nav_point_count: 0,
    };

    // Check for required structure
    if !nav_content.contains("epub:type=\"toc\"") {
        result.errors.push(ValidationError {
            level: ErrorLevel::Error,
            message: "NAV is missing epub:type=\"toc\" attribute".to_string(),
            location: None,
        });
        result.is_valid = false;
        return result;
    }

    if !nav_content.contains("<ol>") {
        result.errors.push(ValidationError {
            level: ErrorLevel::Error,
            message: "NAV is missing <ol> element".to_string(),
            location: None,
        });
        result.is_valid = false;
        return result;
    }

    // Count navigation items
    let nav_items = extract_nav_items(nav_content);
    result.nav_point_count = nav_items.len();

    if result.nav_point_count == 0 {
        result.errors.push(ValidationError {
            level: ErrorLevel::Error,
            message: "NAV <ol> contains no navigation items".to_string(),
            location: None,
        });
        result.is_valid = false;
        return result;
    }

    // Validate navigation items
    for item in &nav_items {
        if item.href.is_empty() {
            result.errors.push(ValidationError {
                level: ErrorLevel::Error,
                message: "NAV item has no href attribute".to_string(),
                location: None,
            });
            result.is_valid = false;
        }

        // Check if href points to valid file
        if !item.href.is_empty() {
            // Normalize href (remove fragment if present)
            let href = item.href.split('#').next().unwrap_or(&item.href);
            if !xhtml_files.iter().any(|f| {
                f.ends_with(href) || f.contains(href) || f.ends_with(&format!("/{}", href))
            }) {
                result.warnings.push(ValidationError {
                    level: ErrorLevel::Warning,
                    message: format!("NAV item references non-existent file: {}", item.href),
                    location: Some(item.href.clone()),
                });
            }
        }
    }

    result
}

/// A navigation point in NCX
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct NavPoint {
    id: String,
    content_src: String,
    play_order: i32,
    label: String,
}

/// Extract navigation points from NCX
fn extract_nav_points(ncx_content: &str) -> Vec<NavPoint> {
    let mut points = Vec::new();

    // Find all navPoint elements
    let nav_point_start = "<navPoint";
    let nav_point_end = "</navPoint>";

    let mut pos = 0;
    while let Some(start) = ncx_content[pos..].find(nav_point_start) {
        let start = pos + start;

        // Find closing tag
        if let Some(end) = ncx_content[start..].find(nav_point_end) {
            let end = start + end + nav_point_end.len();
            let point_content = &ncx_content[start..end];

            // Extract id
            let id = extract_attribute(point_content, "id").unwrap_or_default();

            // Extract content src
            let content_src = extract_attribute(point_content, "src").unwrap_or_default();

            // Extract playOrder
            let play_order = extract_attribute(point_content, "playOrder")
                .and_then(|s| s.parse().ok())
                .unwrap_or(points.len() as i32 + 1);

            // Extract navLabel text
            let label = extract_nav_label(point_content).unwrap_or_default();

            points.push(NavPoint {
                id,
                content_src,
                play_order,
                label,
            });

            pos = end;
        } else {
            break;
        }
    }

    points
}

/// A navigation item in NAV
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct NavItem {
    href: String,
    text: String,
}

/// Extract navigation items from NAV
fn extract_nav_items(nav_content: &str) -> Vec<NavItem> {
    let mut items = Vec::new();

    // Find all <li> elements
    let li_start = "<li";
    let li_end = "</li>";

    let mut pos = 0;
    while let Some(start) = nav_content[pos..].find(li_start) {
        let start = pos + start;

        // Find closing tag
        if let Some(end) = nav_content[start..].find(li_end) {
            let end = start + end + li_end.len();
            let item_content = &nav_content[start..end];

            // Extract href from <a href="...">
            let href = extract_attribute(item_content, "href").unwrap_or_default();

            // Extract link text
            let text = extract_link_text(item_content).unwrap_or_default();

            items.push(NavItem { href, text });

            pos = end;
        } else {
            break;
        }
    }

    items
}

/// Extract attribute value from XML element
fn extract_attribute(element: &str, attr_name: &str) -> Option<String> {
    let attr_pattern = format!("{}=\"", attr_name);
    if let Some(start) = element.find(&attr_pattern) {
        let start = start + attr_pattern.len();
        if let Some(end) = element[start..].find('"') {
            return Some(element[start..start + end].to_string());
        }
    }
    None
}

/// Extract navLabel text from NCX navPoint
fn extract_nav_label(point_content: &str) -> Option<String> {
    // Look for <navLabel>...</navLabel>
    if let Some(label_start) = point_content.find("<navLabel>") {
        if let Some(text_start) = point_content[label_start..].find("<text>") {
            let text_start = label_start + text_start + "<text>".len();
            if let Some(text_end) = point_content[text_start..].find("</text>") {
                return Some(
                    point_content[text_start..text_start + text_end]
                        .trim()
                        .to_string(),
                );
            }
        }
    }
    None
}

/// Extract link text from NAV <a> element
fn extract_link_text(item_content: &str) -> Option<String> {
    // Look for content after <a...> and before </a>
    if let Some(a_start) = item_content.find("<a") {
        // Find the end of the opening tag
        if let Some(tag_end) = item_content[a_start..].find('>') {
            let text_start = a_start + tag_end + 1;
            if let Some(text_end) = item_content[text_start..].find("</a>") {
                return Some(
                    item_content[text_start..text_start + text_end]
                        .trim()
                        .to_string(),
                );
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ncx_empty() {
        let ncx = "";
        let xhtml = vec!["chapter1.xhtml".to_string()];
        let result = validate_ncx(ncx, &xhtml);

        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_ncx_valid() {
        let ncx = r#"<?xml version="1.0"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="test"/>
  </head>
  <docTitle><text>Test Book</text></docTitle>
  <docAuthor><text>Test Author</text></docAuthor>
  <navMap>
    <navPoint id="navpoint-1" playOrder="1">
      <navLabel><text>Chapter 1</text></navLabel>
      <content src="xhtml/chapter1.xhtml"/>
    </navPoint>
  </navMap>
</ncx>"#;
        let xhtml = vec!["xhtml/chapter1.xhtml".to_string()];
        let result = validate_ncx(ncx, &xhtml);

        assert!(result.is_valid);
        assert_eq!(result.nav_point_count, 1);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_ncx_missing_navmap() {
        let ncx = r#"<?xml version="1.0"?>
<ncx>
  <head><meta name="dtb:uid" content="test"/></head>
  <docTitle><text>Test</text></docTitle>
  <docAuthor><text>Author</text></docAuthor>
</ncx>"#;
        let xhtml = vec![];
        let result = validate_ncx(ncx, &xhtml);

        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.message.contains("navMap")));
    }

    #[test]
    fn test_validate_ncx_no_navpoints() {
        let ncx = r#"<?xml version="1.0"?>
<ncx>
  <head><meta name="dtb:uid" content="test"/></head>
  <docTitle><text>Test</text></docTitle>
  <docAuthor><text>Author</text></docAuthor>
  <navMap>
  </navMap>
</ncx>"#;
        let xhtml = vec![];
        let result = validate_ncx(ncx, &xhtml);

        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("navigation points")));
    }

    #[test]
    fn test_validate_nav_valid() {
        let nav = r#"<?xml version="1.0"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head>
  <title>Table of Contents</title>
</head>
<body>
  <nav epub:type="toc">
    <h1>Table of Contents</h1>
    <ol>
      <li><a href="xhtml/chapter1.xhtml">Chapter 1</a></li>
    </ol>
  </nav>
</body>
</html>"#;
        let xhtml = vec!["xhtml/chapter1.xhtml".to_string()];
        let result = validate_nav(nav, &xhtml);

        assert!(result.is_valid);
        assert_eq!(result.nav_point_count, 1);
    }

    #[test]
    fn test_validate_nav_missing_epub_type() {
        let nav = r#"<?xml version="1.0"?>
<html>
<body>
  <nav>
    <ol><li><a href="chapter1.xhtml">Chapter</a></li></ol>
  </nav>
</body>
</html>"#;
        let xhtml = vec![];
        let result = validate_nav(nav, &xhtml);

        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.message.contains("epub:type=\"toc\"")));
    }

    #[test]
    fn test_extract_nav_points() {
        let ncx = r#"<navMap>
  <navPoint id="navpoint-1" playOrder="1">
    <navLabel><text>Chapter 1</text></navLabel>
    <content src="chapter1.xhtml"/>
  </navPoint>
  <navPoint id="navpoint-2" playOrder="2">
    <navLabel><text>Chapter 2</text></navLabel>
    <content src="chapter2.xhtml"/>
  </navPoint>
</navMap>"#;

        let points = extract_nav_points(ncx);
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].id, "navpoint-1");
        assert_eq!(points[0].content_src, "chapter1.xhtml");
        assert_eq!(points[0].label, "Chapter 1");
    }

    #[test]
    fn test_extract_nav_items() {
        let nav = r#"<ol>
  <li><a href="chapter1.xhtml">Chapter 1</a></li>
  <li><a href="chapter2.xhtml">Chapter 2</a></li>
</ol>"#;

        let items = extract_nav_items(nav);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].href, "chapter1.xhtml");
        assert_eq!(items[0].text, "Chapter 1");
    }
}
