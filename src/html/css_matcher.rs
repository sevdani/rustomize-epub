//! CSS matcher - match CSS selectors against used selectors from HTML

use crate::html::selector_extractor::UsedSelectors;

/// Check if a CSS selector matches any used selector
pub fn selector_matches(selector: &str, used: &UsedSelectors) -> bool {
    let selector = selector.trim();

    // ID selector
    if let Some(ide_text) = selector.strip_prefix('#') {
        let id_name = ide_text.trim();
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

    // Complex selectors - extract the first component
    for component in selector.split([' ', '>', '+', '~']) {
        let component = component.trim();
        if let Some(component_text) = component.strip_prefix('#') {
            let id_name = component_text.trim();
            if used.ids.contains(id_name) {
                return true;
            }
        } else if let Some(class_text) = component.strip_prefix('#') {
            let class_name = class_text.trim();
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
    fn test_selector_matches_class() {
        let selector = ".btn";
        let used = UsedSelectors {
            classes: vec!["btn".to_string()].into_iter().collect(),
            ids: HashSet::new(),
            elements: HashSet::new(),
        };

        assert!(selector_matches(selector, &used));
    }

    #[test]
    fn test_selector_matches_id() {
        let selector = "#header";
        let used = UsedSelectors {
            classes: HashSet::new(),
            ids: vec!["header".to_string()].into_iter().collect(),
            elements: HashSet::new(),
        };

        assert!(selector_matches(selector, &used));
    }

    #[test]
    fn test_selector_matches_element() {
        let selector = "div";
        let used = UsedSelectors {
            classes: HashSet::new(),
            ids: HashSet::new(),
            elements: vec!["div".to_string()].into_iter().collect(),
        };

        assert!(selector_matches(selector, &used));
    }

    #[test]
    fn test_selector_matches_unused() {
        let selector = ".unused";
        let used = UsedSelectors {
            classes: vec!["btn".to_string()].into_iter().collect(),
            ids: HashSet::new(),
            elements: HashSet::new(),
        };

        assert!(!selector_matches(selector, &used));
    }
}
