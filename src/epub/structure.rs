//! EPUB structure in-memory operations

use std::collections::HashMap;
use std::path::Path;

// Re-export types for test compatibility
pub use crate::types::ContentFiles;

/// Find content files from OPF content (memory-based)
pub fn find_content_files_from_content(opf_content: &str, opf_path: &str) -> ContentFiles {
    let mut content_files = ContentFiles::default();
    
    // Parse manifest items
    parse_opf_manifest(opf_content, opf_path, &mut content_files);
    
    content_files
}

/// Parse OPF manifest and collect content files
fn parse_opf_manifest(content: &str, opf_path: &str, files: &mut ContentFiles) {
    // Look for <item> elements in manifest
    // Format: <item id="..." href="..." media-type="..." />
    
    let manifest_start = content.find("<manifest");
    let manifest_end = manifest_start.and_then(|start| {
        content[start..].find("</manifest>")
    });
    
    if let (Some(start), Some(end)) = (manifest_start, manifest_end) {
        let manifest_content = &content[start..start + end];
        
        // Parse each item
        let item_start = "<item";
        let item_end = "/>";
        
        let mut pos = 0;
        while let Some(item_start_pos) = manifest_content[pos..].find(item_start) {
            let item_end_pos = manifest_content[item_start_pos + pos..].find(item_end);
            
            if let Some(end_pos) = item_end_pos {
                let item = &manifest_content[pos + item_start_pos..pos + item_start_pos + end_pos + item_end.len()];
                
                // Extract href
                if let Some(href) = extract_attribute(item, "href") {
                    let media_type = extract_attribute(item, "media-type").unwrap_or_default();
                    
                    // Build full path relative to OPF location
                    let full_path = build_full_path(opf_path, &href);
                    
                    // Classify by media type
                    match media_type.as_str() {
                        "application/xhtml+xml" | "application/xml" => {
                            files.xhtml.push(full_path);
                        }
                        "text/css" => {
                            files.css.push(full_path);
                        }
                        "image/jpeg" | "image/png" | "image/gif" | "image/svg+xml" => {
                            files.images.push(full_path);
                        }
                        "font/otf" | "font/ttf" | "application/x-font-otf" | "application/x-font-ttf" => {
                            files.fonts.push(full_path);
                        }
                        "application/x-dtbncx+xml" => {
                            files.ncx.push(full_path);
                        }
                        "application/oebps-package+xml" => {
                            files.opf.push(full_path);
                        }
                        _ => {
                            files.other.push(full_path);
                        }
                    }
                }
                
                pos += item_start_pos + end_pos + item_end.len();
            } else {
                break;
            }
        }
    }
}

/// Build full path from OPF base and relative href
fn build_full_path(opf_path: &str, href: &str) -> String {
    if opf_path.is_empty() {
        return href.to_string();
    }
    
    let opf_path = opf_path.trim_end_matches('/');
    let href = href.trim_start_matches('/');
    
    if href.is_empty() {
        opf_path.to_string()
    } else {
        format!("{}/{}", opf_path, href)
    }
}

/// Extract an attribute value from XML element
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

/// Update OPF manifest entries when images are renamed (memory-based)
pub fn update_opf_content(opf_content: &str, rename_map: &HashMap<String, String>) -> String {
    let mut new_content = opf_content.to_string();
    
    for (old_path, new_path) in rename_map {
        // Replace href values
        let old_href = format!("href=\"{}\"", old_path);
        let new_href = format!("href=\"{}\"", new_path);
        
        new_content = new_content.replace(&old_href, &new_href);
        
        // Also replace image references in tags
        let old_tag = format!("href=\"{}", old_path);
        let new_tag = format!("href=\"{}", new_path);
        
        new_content = new_content.replace(&old_tag, &new_tag);
    }
    
    new_content
}

/// Update XHTML content references (memory-based)
pub fn update_xhtml_content(xhtml_content: &str, rename_map: &HashMap<String, String>) -> String {
    let mut new_content = xhtml_content.to_string();
    
    for (old_path, new_path) in rename_map {
        // Replace image references in <img> tags
        let old_src = format!("src=\"{}\"", old_path);
        let new_src = format!("src=\"{}\"", new_path);
        
        new_content = new_content.replace(&old_src, &new_src);
        
        // Also handle relative path variations
        let old_filename = old_path.split('/').next_back().unwrap_or(old_path);
        let new_filename = new_path.split('/').next_back().unwrap_or(new_path);
        
        let old_format = format!("src=\"{}\"", old_filename);
        let new_format = format!("src=\"{}\"", new_filename);
        new_content = new_content.replace(&old_format, &new_format);
    }
    
    new_content
}

/// Update CSS content references (memory-based)
pub fn update_css_content(css_content: &str, rename_map: &HashMap<String, String>) -> String {
    let mut new_content = css_content.to_string();
    
    for (old_path, new_path) in rename_map {
        let old_url = format!("url(\"{}\")", old_path);
        let new_url = format!("url(\"{}\")", new_path);
        
        new_content = new_content.replace(&old_url, &new_url);
        
        let old_url_noquote = format!("url({})", old_path);
        let new_url_noquote = format!("url({})", new_path);
        
        new_content = new_content.replace(&old_url_noquote, &new_url_noquote);
    }
    
    new_content
}

/// Fix SVG covers - wrap SVG images in image tags if needed (memory-based)
pub fn fix_svg_covers_content(opf_content: &str) -> (String, usize) {
    let mut fixed_count = 0;
    let mut new_content = opf_content.to_string();
    
    // Look for SVG items in manifest
    let svg_pattern = "media-type=\"image/svg+xml\"";
    
    for (start, _) in opf_content.match_indices(svg_pattern) {
        // Find the item tag
        let item_start = opf_content[..start].rfind("<item").unwrap_or(0);
        let item_end = opf_content[start..].find("/>").map(|e| start + e + 2).unwrap_or(start);
        
        if item_start > 0 && item_end > item_start {
            let item_tag = &opf_content[item_start..item_end];
            
            // Check if it's already wrapped in a meta tag
            let meta_start = opf_content[..item_start].rfind("<meta").unwrap_or(0);
            let meta_end = opf_content[meta_start..].find(">").map(|e| meta_start + e + 1).unwrap_or(0);
            
            if meta_end > meta_start && opf_content[meta_start..meta_end].contains("cover") {
                // Already wrapped, skip
                continue;
            }
            
            // Need to add a cover reference
            fixed_count += 1;
            
            // Extract href
            if let Some(href) = extract_attribute(item_tag, "href") {
                // Add meta tag for cover
                let meta_tag = format!("<meta name=\"cover\" content=\"{}\" />", href);
                
                // Insert before manifest end
                if let Some(manifest_end) = opf_content.find("</manifest>") {
                    new_content.insert_str(manifest_end, &format!("\n    {}", meta_tag));
                }
            }
        }
    }
    
    (new_content, fixed_count)
}

/// Result of TOC fixing
#[derive(Debug, Clone)]
pub struct TocFixResult {
    pub is_valid: bool,
    pub status: String,
    pub ncx_exists: bool,
    pub nav_exists: bool,
    pub nav_point_count: usize,
}

/// Fix or regenerate TOC - check if TOC exists (memory-based, for backward compatibility)
pub fn fix_toc_content(opf_content: &str) -> (bool, String) {
    // Check if NCX exists
    let ncx_exists = opf_content.contains("application/x-dtbncx+xml");
    
    if ncx_exists {
        // TOC exists
        if let Some(_ncx_path) = find_ncx_path_from_content(opf_content) {
            return (true, "Existing TOC is valid".to_string());
        }
    }
    
    (false, "No valid TOC found".to_string())
}

/// Find NCX path from OPF content
fn find_ncx_path_from_content(opf_content: &str) -> Option<String> {
    let manifest_start = opf_content.find("<manifest")?;
    let manifest_end = opf_content[manifest_start..].find("</manifest>")?;
    let manifest_content = &opf_content[manifest_start..manifest_start + manifest_end];
    
    if let Some(ncx_pos) = manifest_content.find("application/x-dtbncx+xml") {
        // Find the item tag containing NCX
        let item_start = manifest_content[..ncx_pos].rfind("<item")?;
        let item_end = manifest_content[item_start..].find("/>")?;
        let item_tag = &manifest_content[item_start..item_start + item_end + 2];
        
        extract_attribute(item_tag, "href")
    } else {
        None
    }
}

/// Add cover reference to OPF manifest
pub fn add_cover_to_opf_content(opf_content: &str, cover_path: &str) -> String {
    let mut new_content = opf_content.to_string();
    
    // Get the directory path from cover_path for the media-type
    let cover_filename = Path::new(cover_path)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("cover.jpg");
    
    let cover_extension = Path::new(cover_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");
    
    let media_type = match cover_extension.to_lowercase().as_str() {
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/jpeg",
    };
    
    // Add cover image to manifest
    let manifest_insert = format!("\n    <item id=\"cover\" href=\"{}\" media-type=\"{}\"/>", cover_path, media_type);
    
    // Find the manifest section and add the cover item before </manifest>
    if let Some(manifest_end) = new_content.find("</manifest>") {
        new_content.insert_str(manifest_end, &manifest_insert);
    }
    
    // Add cover meta tag if not exists
    if !new_content.contains("name=\"cover\"") {
        let meta_tag = format!("\n    <meta name=\"cover\" content=\"{}\"/>", cover_filename);
        
        // Find manifest end again (positions may have changed)
        if let Some(manifest_end) = new_content.find("</manifest>") {
            new_content.insert_str(manifest_end, &meta_tag);
        }
    }
    
    new_content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_attribute_basic() {
        let element = "id=\"test\" href=\"chapter1.xhtml\"";
        let href = extract_attribute(element, "href");
        assert_eq!(href, Some("chapter1.xhtml".to_string()));
    }

    #[test]
    fn test_extract_attribute_not_found() {
        let element = "id=\"test\" href=\"chapter1.xhtml\"";
        let value = extract_attribute(element, "src");
        assert_eq!(value, None);
    }

    #[test]
    fn test_build_full_path_empty() {
        let result = build_full_path("", "chapter1.xhtml");
        assert_eq!(result, "chapter1.xhtml");
    }

    #[test]
    fn test_build_full_path_with_base() {
        let result = build_full_path("OEBPS", "chapter1.xhtml");
        assert_eq!(result, "OEBPS/chapter1.xhtml");
    }

    #[test]
    fn test_build_full_path_trailing_slash() {
        let result = build_full_path("OEBPS/", "chapter1.xhtml");
        assert_eq!(result, "OEBPS/chapter1.xhtml");
    }

    #[test]
    fn test_parse_opf_manifest_basic() {
        let content = r#"<manifest>
            <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
            <item id="css" href="styles.css" media-type="text/css"/>
            <item id="img1" href="images/cover.jpg" media-type="image/jpeg"/>
        </manifest>"#;
        
        let mut files = ContentFiles::default();
        parse_opf_manifest(content, "OEBPS/content.opf", &mut files);
        
        assert_eq!(files.xhtml.len(), 0);
        assert_eq!(files.ncx.len(), 1);
        assert_eq!(files.css.len(), 1);
        assert_eq!(files.images.len(), 1);
    }

    #[test]
    fn test_find_content_files_from_content() {
        let opf_content = r#"<manifest>
            <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
        </manifest>"#;
        
        let files = find_content_files_from_content(opf_content, "content.opf");
        assert_eq!(files.ncx.len(), 1);
    }

    #[test]
    fn test_fix_toc_content_ncx_exists() {
        let opf_content = r#"<manifest>
            <item href="toc.ncx" media-type="application/x-dtbncx+xml"/>
        </manifest>"#;
        
        let (valid, status) = fix_toc_content(opf_content);
        assert!(valid);
        assert_eq!(status, "Existing TOC is valid");
    }

    #[test]
    fn test_fix_toc_content_no_ncx() {
        let opf_content = "<manifest></manifest>";
        let (valid, status) = fix_toc_content(opf_content);
        
        assert!(!valid);
        assert_eq!(status, "No valid TOC found");
    }

    #[test]
    fn test_find_ncx_path_from_content() {
        let opf_content = r#"<manifest>
            <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
        </manifest>"#;
        
        let path = find_ncx_path_from_content(opf_content);
        assert_eq!(path, Some("toc.ncx".to_string()));
    }

    #[test]
    fn test_update_opf_content_image_rename() {
        let opf_content = "<manifest>\n<item href=\"old.jpg\"/>\n</manifest>";
        let mut rename_map = HashMap::new();
        rename_map.insert("old.jpg".to_string(), "new.jpg".to_string());
        
        let result = update_opf_content(opf_content, &rename_map);
        assert!(result.contains("new.jpg"));
        assert!(!result.contains("old.jpg"));
    }

    #[test]
    fn test_update_xhtml_content_image_rename() {
        let xhtml_content = "<img src=\"old.jpg\"/>";
        let mut rename_map = HashMap::new();
        rename_map.insert("old.jpg".to_string(), "new.jpg".to_string());
        
        let result = update_xhtml_content(xhtml_content, &rename_map);
        assert!(result.contains("new.jpg"));
    }

    #[test]
    fn test_update_css_content_image_rename() {
        let css_content = "background: url(\"old.jpg\")";
        let mut rename_map = HashMap::new();
        rename_map.insert("old.jpg".to_string(), "new.jpg".to_string());
        
        let result = update_css_content(css_content, &rename_map);
        assert!(result.contains("new.jpg"));
    }

    #[test]
    fn test_fix_svg_covers_content() {
        let opf_content = r#"<manifest>
            <item href="cover.svg" media-type="image/svg+xml"/>
        </manifest>"#;
        
        let (result, count) = fix_svg_covers_content(opf_content);
        assert!(count > 0 || result.contains("meta"));
    }
    
    #[test]
    fn test_add_cover_to_opf_content_jpeg() {
        let opf_content = r#"<manifest>
            <item href="chapter1.xhtml" media-type="application/xhtml+xml"/>
        </manifest>"#;
        
        let result = add_cover_to_opf_content(opf_content, "OEBPS/images/cover.jpg");
        assert!(result.contains("cover.jpg"));
        assert!(result.contains("media-type=\"image/jpeg\""));
        assert!(result.contains("<meta name=\"cover\""));
    }
    
    #[test]
    fn test_add_cover_to_opf_content_png() {
        let opf_content = r#"<manifest>
            <item href="chapter1.xhtml" media-type="application/xhtml+xml"/>
        </manifest>"#;
        
        let result = add_cover_to_opf_content(opf_content, "OEBPS/images/cover.png");
        assert!(result.contains("cover.png"));
        assert!(result.contains("media-type=\"image/png\""));
    }
    
    #[test]
    fn test_add_cover_to_opf_content_meta_tag_added() {
        let opf_content = r#"<manifest>
            <item href="chapter1.xhtml" media-type="application/xhtml+xml"/>
        </manifest>"#;
        
        let result = add_cover_to_opf_content(opf_content, "OEBPS/images/cover.jpg");
        assert!(result.contains("name=\"cover\""));
    }
    
    #[test]
    fn test_add_cover_to_opf_content_with_existing_meta() {
        let opf_content = r#"<manifest>
            <item href="chapter1.xhtml" media-type="application/xhtml+xml"/>
            <meta name="cover" content="old-cover.jpg"/>
        </manifest>"#;
        
        let result = add_cover_to_opf_content(opf_content, "OEBPS/images/cover.jpg");
        // Should preserve existing meta but still add manifest entry
        assert!(result.contains("name=\"cover\""));
    }
    
    #[test]
    fn test_add_cover_to_opf_content_empty_manifest() {
        let opf_content = "<manifest></manifest>";
        
        let result = add_cover_to_opf_content(opf_content, "OEBPS/images/cover.jpg");
        assert!(result.contains("cover.jpg"));
    }
}
