//! CSS cleaner - remove unused rules

use crate::html::selector_extractor::extract_used_selectors;

/// Remove unused CSS rules from content
///
/// This function:
/// 1. Extracts used selectors from all provided HTML/XHTML content
/// 2. Scans CSS rules
/// 3. Removes rules that don't match any used selector
/// 4. Returns cleaned CSS and count of removed rules
pub fn cleanup_css_content(css_content: &str, html_content: &str) -> (String, usize) {
    // Extract used selectors from HTML
    let used = extract_used_selectors(html_content);

    // Parse CSS and filter unused rules
    filter_css_content(css_content, &used)
}

/// Count CSS rules in content
fn count_css_rules(content: &str) -> usize {
    // Count CSS blocks (groups between curly braces)
    let mut count = 0;
    let mut in_block = false;

    for ch in content.chars() {
        if ch == '{' {
            in_block = true;
        } else if ch == '}' && in_block {
            count += 1;
            in_block = false;
        }
    }

    count
}

/// Filter CSS content to remove unused rules
fn filter_css_content(
    css_content: &str,
    used: &crate::html::selector_extractor::UsedSelectors,
) -> (String, usize) {
    let original_rules = count_css_rules(css_content);

    // Remove comments first
    let css_no_comments = remove_css_comments(css_content);

    // Simple approach: use regex to find and filter rules
    let mut result = String::new();

    // Split by closing brace to get rules
    let rules: Vec<&str> = css_no_comments
        .split('}')
        .filter(|s| !s.trim().is_empty())
        .collect();

    for rule in rules {
        let rule = rule.trim();
        if rule.is_empty() {
            continue;
        }

        // Extract selector part (before the first {)
        if let Some(brace_pos) = rule.find('{') {
            let selector_part = rule[..brace_pos].trim();

            // Check if any selector in this rule matches used selectors
            let should_keep = should_keep_rule(selector_part, used);

            if should_keep {
                result.push_str(rule);
                result.push_str("}\n");
            }
        } else {
            // Keep rules without braces (like @import)
            result.push_str(rule);
            result.push('\n');
        }
    }

    let new_rules = count_css_rules(&result);
    let rules_removed = original_rules - new_rules;

    (result, rules_removed)
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

/// Check if a rule should be kept based on its selectors
fn should_keep_rule(
    selector_str: &str,
    used: &crate::html::selector_extractor::UsedSelectors,
) -> bool {
    // Split by comma for multiple selectors
    let selectors: Vec<&str> = selector_str.split(',').map(|s| s.trim()).collect();

    eprintln!(
        "should_keep_rule: selector_str='{}', selectors={:?}",
        selector_str, selectors
    );
    eprintln!("used.classes={:?}", used.classes);
    eprintln!("used.ids={:?}", used.ids);
    eprintln!("used.elements={:?}", used.elements);

    // Check if ANY selector matches used selectors
    for selector in selectors {
        if selector.is_empty() {
            eprintln!("  skipping empty selector");
            continue;
        }

        eprintln!("  checking selector='{}'", selector);
        if selector_matches(selector, used) {
            eprintln!("  MATCH!");
            return true;
        }
    }

    eprintln!("  no match");
    false
}

/// Check if a CSS selector matches any used selector
fn selector_matches(selector: &str, used: &crate::html::selector_extractor::UsedSelectors) -> bool {
    let selector = selector.trim();

    // ID selector
    if let Some(id_text) = selector.strip_prefix('#') {
        let id_name = id_text.trim();
        return used.ids.contains(id_name);
    }

    // Class selector
    if let Some(class_text) = selector.strip_prefix('.') {
        let class_name = class_text.trim();
        return used.classes.contains(class_name);
    }

    // Element selector (simple case)
    if selector.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return used.elements.contains(selector);
    }

    // Attribute selector - consider as potentially used
    if selector.contains('[') {
        return true;
    }

    // Complex selectors - extract the first component
    // e.g., "div.container" -> check if "div" or ".container" matches
    for component in selector.split([' ', '>', '+', '~', '.']) {
        let component = component.trim();
        if let Some(component_text) = component.strip_prefix('#') {
            let id_name = component_text.trim();
            if used.ids.contains(id_name) {
                return true;
            }
        } else if let Some(component_text) = component.strip_prefix('.') {
            let class_name = component_text.trim();
            if used.classes.contains(class_name) {
                return true;
            }
        } else if component.chars().all(|c| c.is_alphanumeric() || c == '-')
            && used.elements.contains(component)
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_cleanup_css_content_empty() {
        let css_input = "";
        let html_input = "<body></body>";
        let (output, rules_removed) = cleanup_css_content(css_input, html_input);

        assert_eq!(output, "");
        assert_eq!(rules_removed, 0);
    }

    #[test]
    fn test_cleanup_css_content_with_used_rules() {
        let css_input = ".btn { color: red; }\nbody { color: blue; }";
        let html_input = r#"<body><div class="btn">Hello</div></body>"#;
        let (output, rules_removed) = cleanup_css_content(css_input, html_input);

        assert!(output.contains(".btn"));
        assert!(output.contains("body"));
        assert!(output.contains("red"));
        assert!(output.contains("blue"));
        assert!(rules_removed == 0);
    }

    #[test]
    fn test_cleanup_css_content_with_unused_rules() {
        let css_input = r#"
.btn { color: red; }
#header { background: blue; }
.unused-class { display: none; }
div { margin: 0; }
        "#;
        let html_input = r#"<div class="btn">Hello</div>"#;
        let (output, rules_removed) = cleanup_css_content(css_input, html_input);

        assert!(output.contains(".btn"));
        assert!(output.contains("div"));
        assert!(!output.contains(".unused-class"));
        assert!(!output.contains("#header"));
        assert!(rules_removed > 0);
    }

    #[test]
    fn test_cleanup_css_content_with_comments() {
        let css_input = "/* comment */\n.btn {\n  color: red;\n}";
        let html_input = r#"<body><div class="btn">Hello</div></body>"#;
        let (output, rules_removed) = cleanup_css_content(css_input, html_input);

        eprintln!("CSS input: {}", css_input);
        eprintln!("HTML input: {}", html_input);
        eprintln!("Output: {}", output);

        assert!(output.contains(".btn"));
        assert!(output.contains("color"));
        assert!(output.contains("red"));
        assert_eq!(rules_removed, 0);
    }

    #[test]
    fn test_cleanup_css_content_whitespace_handling() {
        let css_input = "   \n  .btn {\n  color: red;\n  }\n   ";
        let html_input = r#"<body><div class="btn">Hello</div></body>"#;
        let (output, rules_removed) = cleanup_css_content(css_input, html_input);

        assert!(output.contains(".btn"));
        assert!(output.contains("color"));
        assert_eq!(rules_removed, 0);
    }

    #[test]
    fn test_cleanup_css_content_all_unused() {
        let css_input = r#"
.unused1 { color: red; }
.unused2 { background: blue; }
        "#;
        let html_input = r#"<div class="used">Hello</div>"#;
        let (output, rules_removed) = cleanup_css_content(css_input, html_input);

        assert!(!output.contains("unused1"));
        assert!(!output.contains("unused2"));
        assert!(rules_removed > 0);
    }

    #[test]
    fn test_cleanup_css_content_with_id() {
        let css_input = r#"
#header { color: red; }
#unused { background: blue; }
        "#;
        let html_input = r#"<div id="header">Hello</div>"#;
        let (output, rules_removed) = cleanup_css_content(css_input, html_input);

        assert!(output.contains("#header"));
        assert!(!output.contains("#unused"));
        assert!(rules_removed > 0);
    }

    #[test]
    fn test_selector_matches_class() {
        let css_selector = ".btn";
        let used = crate::html::selector_extractor::UsedSelectors {
            classes: vec!["btn".to_string()].into_iter().collect(),
            ids: HashSet::new(),
            elements: HashSet::new(),
        };

        assert!(selector_matches(css_selector, &used));
    }

    #[test]
    fn test_selector_matches_id() {
        let css_selector = "#header";
        let used = crate::html::selector_extractor::UsedSelectors {
            classes: HashSet::new(),
            ids: vec!["header".to_string()].into_iter().collect(),
            elements: HashSet::new(),
        };

        assert!(selector_matches(css_selector, &used));
    }

    #[test]
    fn test_selector_matches_element() {
        let css_selector = "div";
        let used = crate::html::selector_extractor::UsedSelectors {
            classes: HashSet::new(),
            ids: HashSet::new(),
            elements: vec!["div".to_string()].into_iter().collect(),
        };

        assert!(selector_matches(css_selector, &used));
    }

    #[test]
    fn test_selector_matches_unused() {
        let css_selector = ".unused";
        let used = crate::html::selector_extractor::UsedSelectors {
            classes: vec!["btn".to_string()].into_iter().collect(),
            ids: HashSet::new(),
            elements: HashSet::new(),
        };

        assert!(!selector_matches(css_selector, &used));
    }
}
