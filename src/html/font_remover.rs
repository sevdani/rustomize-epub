//! Font remover - remove embedded fonts
/// Remove fonts from CSS files (memory-based)
pub fn remove_fonts_from_css_content(css_content: &str) -> (String, usize) {
    let mut fonts_removed = 0;
    let mut result = css_content.to_string();
    
    // Remove @font-face rules
    let at_font_face_start = "@font-face";
    while let Some(start) = result.find(at_font_face_start) {
        if let Some(end) = result[start..].find('}') {
            result.drain(start..start + end + 1);
            fonts_removed += 1;
        } else {
            break;
        }
    }
    
    (result, fonts_removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_fonts_from_css_content_empty() {
        let input = "";
        let (output, removed) = remove_fonts_from_css_content(input);
        
        assert_eq!(output, "");
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_remove_fonts_from_css_content_no_fonts() {
        let input = "body { color: red; }";
        let (output, removed) = remove_fonts_from_css_content(input);
        
        assert_eq!(output, "body { color: red; }");
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_remove_fonts_from_css_content_single_font() {
        let input = "@font-face { font-family: TestFont; src: url(test.woff); }\nbody { color: red; }";
        let (output, removed) = remove_fonts_from_css_content(input);
        
        assert!(output.contains("body"));
        assert!(output.contains("color"));
        assert!(!output.contains("@font-face"));
        assert_eq!(removed, 1);
    }

    #[test]
    fn test_remove_fonts_from_css_content_multiple_fonts() {
        let input = "@font-face { font-family: A; src: url(a.woff); }\n@font-face { font-family: B; src: url(b.woff); }\nbody { color: red; }";
        let (output, removed) = remove_fonts_from_css_content(input);
        
        assert!(output.contains("body"));
        assert!(!output.contains("@font-face"));
        assert_eq!(removed, 2);
    }

    #[test]
    fn test_remove_fonts_from_css_content_preserves_other_content() {
        let input = "@font-face { font-family: Test; src: url(test.woff); }\nbody { color: red; }\n.container { margin: 0; }";
        let (output, removed) = remove_fonts_from_css_content(input);
        
        assert!(output.contains("body"));
        assert!(output.contains("container"));
        assert!(output.contains("margin"));
        assert_eq!(removed, 1);
    }
}
