//! CSS parser - extract selectors from CSS content

use std::collections::HashSet;

/// A parsed CSS selector
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CssSelector {
    /// Element selector (e.g., `div`, `p`, `h1`)
    Element(String),
    /// Class selector (e.g., `.btn`, `.container`)
    Class(String),
    /// ID selector (e.g., `#header`, `#main`)
    Id(String),
}

/// Parse CSS content and extract selectors
pub fn parse_css_selectors(css_content: &str) -> Vec<CssSelector> {
    let mut selectors = HashSet::new();
    
    // Remove comments
    let css_no_comments = remove_css_comments(css_content);
    
    // Extract selectors from rule blocks
    extract_selectors_from_rules(&css_no_comments, &mut selectors);
    
    // Also extract selectors from @rules
    extract_selectors_from_at_rules(&css_no_comments, &mut selectors);
    
    selectors.into_iter().collect()
}

/// Remove CSS comments
fn remove_css_comments(content: &str) -> String {
    let mut result = String::new();
    let mut chars = content.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            // Skip comment
            chars.next(); // skip *
            while let Some(c) = chars.next() {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next(); // skip /
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }
    
    result
}

/// Extract selectors from CSS rule blocks
fn extract_selectors_from_rules(css: &str, selectors: &mut HashSet<CssSelector>) {
    let mut pos = 0;
    
    while pos < css.len() {
        // Skip whitespace
        while pos < css.len() && css[pos..].chars().next().is_none_or(|c| c.is_whitespace()) {
            pos += 1;
        }
        
        if pos >= css.len() {
            break;
        }
        
        // Check if this is a selector start
        let first_char = css[pos..].chars().next().unwrap_or('\0');
        
        if first_char == '#' || first_char == '.' || first_char == '[' || first_char.is_alphabetic() {
            if let Some(end) = find_rule_end(css, pos) {
                let rule_content = &css[pos..end];
                let selectors_str = extract_selectors_from_rule_content(rule_content);
                
                for sel in parse_selectors(&selectors_str) {
                    selectors.insert(sel);
                }
                
                pos = end;
            } else {
                pos += 1;
            }
        } else {
            pos += 1;
        }
    }
}

/// Find the end of a rule block
fn find_rule_end(css: &str, start: usize) -> Option<usize> {
    // Find the opening brace
    let brace_pos = css[start..].find('{')?;
    let brace_pos = start + brace_pos;
    
    // Find the matching closing brace
    let closing_pos = find_matching_brace(css, brace_pos)?;
    
    Some(closing_pos + 1)
}

/// Find matching closing brace
fn find_matching_brace(content: &str, open_pos: usize) -> Option<usize> {
    let mut depth: i32 = 1;
    let chars: Vec<char> = content.chars().collect();

    for (i, brace) in chars.iter().enumerate().skip(open_pos + 1) {
        match brace {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    
    None
}

/// Parse selector string into individual selectors
fn parse_selectors(selector_str: &str) -> Vec<CssSelector> {
    let mut selectors = Vec::new();
    
    // Split by comma to handle multiple selectors
    for sel in selector_str.split(',') {
        let sel = sel.trim();
        if sel.is_empty() {
            continue;
        }
        
        // Parse the selector
        if let Some(parsed) = parse_single_selector(sel) {
            selectors.push(parsed);
        }
    }
    
    selectors
}

/// Parse a single selector
fn parse_single_selector(selector: &str) -> Option<CssSelector> {
    let selector = selector.trim();
    
    if selector.is_empty() {
        return None;
    }
    
    // ID selector
    if let Some(id_text) = selector.strip_prefix('#') {
        let id = id_text.trim_start_matches(|c: char| !c.is_alphanumeric() && c != '-');
        return Some(CssSelector::Id(id.to_string()));
    }
    
    // Class selector
    if let Some(class_text) = selector.strip_prefix('.') {
        let class = class_text.trim_start_matches(|c: char| !c.is_alphanumeric() && c != '-');
        return Some(CssSelector::Class(class.to_string()));
    }
    
    // Element selector (default)
    Some(CssSelector::Element(selector.to_string()))
}

/// Extract selectors from rule content
fn extract_selectors_from_rule_content(rule: &str) -> String {
    if let Some(brace) = rule.find('{') {
        rule[..brace].trim().to_string()
    } else {
        rule.trim().to_string()
    }
}

/// Extract selectors from @rules (media queries, keyframes, etc.)
fn extract_selectors_from_at_rules(css: &str, selectors: &mut HashSet<CssSelector>) {
    // Look for @media blocks
    let at_media = "@media";
    let mut pos = 0;
    
    while let Some(at_pos) = css[pos..].find(at_media) {
        let at_pos = pos + at_pos;
        
        // Find opening brace
        if let Some(brace_pos) = css[at_pos..].find('{') {
            let brace_pos = at_pos + brace_pos;
            
            // Find matching closing brace
            if let Some(end_pos) = find_matching_brace(css, brace_pos) {
                // Extract content between braces and parse
                let content = &css[brace_pos + 1..end_pos];
                extract_selectors_from_rules(content, selectors);
                
                pos = end_pos + 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_class_selector() {
        let css = ".btn { color: red; }";
        let selectors = parse_css_selectors(css);
        
        assert_eq!(selectors.len(), 1);
        assert!(matches!(selectors[0], CssSelector::Class(ref s) if s == "btn"));
    }

    #[test]
    fn test_parse_id_selector() {
        let css = "#header { background: blue; }";
        let selectors = parse_css_selectors(css);
        
        assert_eq!(selectors.len(), 1);
        assert!(matches!(selectors[0], CssSelector::Id(ref s) if s == "header"));
    }

    #[test]
    fn test_parse_element_selector() {
        let css = "div { margin: 0; }";
        let selectors = parse_css_selectors(css);
        
        assert_eq!(selectors.len(), 1);
        assert!(matches!(selectors[0], CssSelector::Element(ref s) if s == "div"));
    }

    #[test]
    fn test_parse_multiple_selectors() {
        let css = ".btn, #header, div { color: red; }";
        let selectors = parse_css_selectors(css);
        
        assert_eq!(selectors.len(), 3);
    }

    #[test]
    fn test_parse_with_comments() {
        let css = "/* comment */ .btn { color: red; }";
        let selectors = parse_css_selectors(css);
        
        assert_eq!(selectors.len(), 1);
        assert!(matches!(selectors[0], CssSelector::Class(ref s) if s == "btn"));
    }

    #[test]
    fn test_parse_attribute_selector() {
        let css = "[type=\"text\"] { border: 1px; }";
        let selectors = parse_css_selectors(css);
        
        // Attribute selectors are treated as elements
        assert!(!selectors.is_empty());
    }
}
