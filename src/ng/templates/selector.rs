use crate::ng::templates::scanner::ElementUsage;

/// One alternative of an Angular selector (`button[fixBtn].primary`).
#[derive(Debug, Clone)]
pub struct SimpleSelector {
    pub element: Option<String>,
    pub attributes: Vec<String>,
    pub classes: Vec<String>,
}

/// Parses an Angular selector list (`fix-btn, button[fixBtn]:not([link])`).
/// `:not(...)` parts are ignored (conservative: may over-match, never
/// under-matches — safer for unused-code detection).
pub fn parse_selector(selector: &str) -> Vec<SimpleSelector> {
    selector
        .split(',')
        .map(|alternative| parse_alternative(alternative.trim()))
        .filter(|s| s.element.is_some() || !s.attributes.is_empty() || !s.classes.is_empty())
        .collect()
}

fn parse_alternative(selector: &str) -> SimpleSelector {
    let mut result = SimpleSelector {
        element: None,
        attributes: Vec::new(),
        classes: Vec::new(),
    };

    let bytes = selector.as_bytes();
    let mut i = 0;

    // Leading element name.
    let element_start = i;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_')
    {
        i += 1;
    }
    if i > element_start {
        result.element = Some(selector[element_start..i].to_string());
    }

    while i < bytes.len() {
        match bytes[i] {
            b'[' => {
                let end = selector[i..]
                    .find(']')
                    .map(|off| i + off)
                    .unwrap_or(bytes.len());
                let inner = &selector[i + 1..end];
                // [attr=value] — only the name matters for presence matching.
                let name = inner.split('=').next().unwrap_or(inner).trim();
                if !name.is_empty() {
                    result.attributes.push(name.to_string());
                }
                i = end + 1;
            }
            b'.' => {
                let start = i + 1;
                let mut j = start;
                while j < bytes.len()
                    && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'-' || bytes[j] == b'_')
                {
                    j += 1;
                }
                if j > start {
                    result.classes.push(selector[start..j].to_string());
                }
                i = j;
            }
            b':' => {
                // :not(...) — skip the pseudo-class with its argument.
                let mut j = i + 1;
                while j < bytes.len() && bytes[j] != b'(' && bytes[j] != b'[' && bytes[j] != b'.' {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'(' {
                    let mut depth = 1;
                    j += 1;
                    while j < bytes.len() && depth > 0 {
                        match bytes[j] {
                            b'(' => depth += 1,
                            b')' => depth -= 1,
                            _ => {}
                        }
                        j += 1;
                    }
                }
                i = j;
            }
            _ => i += 1,
        }
    }

    result
}

/// Does any alternative of the selector match this element?
pub fn matches(selectors: &[SimpleSelector], element: &ElementUsage) -> bool {
    selectors.iter().any(|selector| {
        if let Some(tag) = &selector.element {
            if tag != &element.tag {
                return false;
            }
        } else if selector.attributes.is_empty() && selector.classes.is_empty() {
            return false;
        }

        selector
            .attributes
            .iter()
            .all(|attr| element.attributes.contains(attr))
            && selector
                .classes
                .iter()
                .all(|class| element.classes.contains(class))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn element(tag: &str, attrs: &[&str]) -> ElementUsage {
        ElementUsage {
            tag: tag.to_string(),
            attributes: attrs.iter().map(|s| s.to_string()).collect(),
            classes: HashSet::new(),
        }
    }

    #[test]
    fn element_selector_matches_tag() {
        let sel = parse_selector("fix-button");
        assert!(matches(&sel, &element("fix-button", &[])));
        assert!(!matches(&sel, &element("fix-badge", &[])));
    }

    #[test]
    fn attribute_selector_matches_any_tag() {
        let sel = parse_selector("[uiTooltip]");
        assert!(matches(&sel, &element("div", &["uiTooltip"])));
        assert!(matches(&sel, &element("span", &["uiTooltip", "other"])));
        assert!(!matches(&sel, &element("div", &["other"])));
    }

    #[test]
    fn compound_selector_requires_both() {
        let sel = parse_selector("button[fixBtn]");
        assert!(matches(&sel, &element("button", &["fixBtn"])));
        assert!(!matches(&sel, &element("button", &[])));
        assert!(!matches(&sel, &element("a", &["fixBtn"])));
    }

    #[test]
    fn comma_alternatives_match_independently() {
        let sel = parse_selector("fix-a, [fixB]");
        assert!(matches(&sel, &element("fix-a", &[])));
        assert!(matches(&sel, &element("div", &["fixB"])));
        assert!(!matches(&sel, &element("div", &[])));
    }

    #[test]
    fn not_pseudo_class_is_ignored_conservatively() {
        let sel = parse_selector("input:not([type=checkbox])");
        assert!(matches(&sel, &element("input", &[])));
    }

    #[test]
    fn attr_with_value_matches_on_presence() {
        let sel = parse_selector("[fixVariant=primary]");
        assert!(matches(&sel, &element("div", &["fixVariant"])));
    }
}
