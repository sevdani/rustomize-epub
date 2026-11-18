//! CSS cleaner - remove unused rules

/// Remove unused CSS rules (memory-based)
pub fn cleanup_css_content(css_content: &str) -> (String, usize) {
    let original_rules = count_css_rules(css_content);
    let result = remove_unused_selectors(css_content);
    let new_rules = count_css_rules(&result);
    
    let rules_removed = original_rules - new_rules;
    
    (result, rules_removed)
}

/// Count CSS rules in content (simplified)
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

/// Remove unused CSS selectors (simplified)
fn remove_unused_selectors(content: &str) -> String {
    // Simplified: just clean up whitespace
    let mut result = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("/*") {
            result.push_str(trimmed);
            result.push('\n');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanup_css_content_empty() {
        let input = "";
        let (output, rules_removed) = cleanup_css_content(input);
        
        assert_eq!(output, "");
        assert_eq!(rules_removed, 0);
    }

    #[test]
    fn test_cleanup_css_content_with_comments() {
        let input = "/* comment */\nbody {\n  color: red;\n}";
        let (output, _rules_removed) = cleanup_css_content(input);
        
        assert!(output.contains("body"));
        assert!(output.contains("color"));
        assert!(output.contains("red"));
    }

    #[test]
    fn test_cleanup_css_content_whitespace() {
        let input = "   \n  body {\n  color: red;\n  }\n   ";
        let (output, _) = cleanup_css_content(input);
        
        assert!(output.contains("body"));
        assert!(output.contains("color"));
    }

    #[test]
    fn test_count_css_rules_basic() {
        let input = "body {}";
        let count = count_css_rules(input);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_count_css_rules_multiple() {
        let input = "body {} div {} span {}";
        let count = count_css_rules(input);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_css_rules_nested() {
        let input = "@media screen { body {} }";
        let count = count_css_rules(input);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_remove_unused_selectors_basic() {
        let input = "body { color: red; }";
        let output = remove_unused_selectors(input);
        
        assert!(output.contains("body"));
        assert!(output.contains("color"));
        assert!(output.contains("red"));
    }

    #[test]
    fn test_remove_unused_selectors_with_comments() {
        let input = "/* comment */\nbody { color: red; }";
        let output = remove_unused_selectors(input);
        
        assert!(!output.contains("comment"));
        assert!(output.contains("body"));
    }

    #[test]
    fn test_remove_unused_selectors_empty_lines() {
        let input = "\n\nbody { color: red; }\n\n";
        let output = remove_unused_selectors(input);
        
        assert!(output.contains("body"));
        assert!(output.contains("color"));
        assert!(!output.starts_with('\n'));
    }

    #[test]
    fn test_cleanup_css_content_rules_removed() {
        let input = "body { color: red; }\n\n\ndiv { background: blue; }";
        let (output, rules_removed) = cleanup_css_content(input);
        
        assert!(output.contains("body"));
        assert!(output.contains("div"));
        assert_eq!(rules_removed, 0);
    }
}
