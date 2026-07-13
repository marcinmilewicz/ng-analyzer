use std::collections::HashSet;

/// One element occurrence in a template with everything selector matching
/// needs: tag name, normalized attribute names, classes.
#[derive(Debug)]
pub struct ElementUsage {
    pub tag: String,
    pub attributes: HashSet<String>,
    pub classes: HashSet<String>,
}

#[derive(Debug, Default)]
pub struct TemplateScan {
    pub elements: Vec<ElementUsage>,
    pub pipes: HashSet<String>,
}

/// Lightweight Angular-template scanner. Handles binding sugar (`[prop]`,
/// `(event)`, `[(model)]`, `*structural`), interpolations and the new control
/// flow blocks (`@if`, `@for` are plain text to this scanner — elements inside
/// them are still found).
pub fn scan_template(html: &str) -> TemplateScan {
    let mut scan = TemplateScan::default();
    let bytes = html.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if html[i..].starts_with("<!--") {
            i = html[i..]
                .find("-->")
                .map(|end| i + end + 3)
                .unwrap_or(bytes.len());
        } else if bytes[i] == b'<' {
            if html[i..].starts_with("</") || html[i..].starts_with("<!") {
                i = html[i..]
                    .find('>')
                    .map(|end| i + end + 1)
                    .unwrap_or(bytes.len());
            } else {
                i = parse_element(html, i, &mut scan);
            }
        } else {
            // Text node: scan interpolations and control-flow block
            // conditions (`@if (expr | pipe)`) for pipes.
            let end = html[i..]
                .find('<')
                .map(|off| i + off)
                .unwrap_or(bytes.len());
            scan_interpolations(&html[i..end], &mut scan.pipes);
            scan_control_flow_expressions(&html[i..end], &mut scan.pipes);
            i = end;
        }
    }

    scan
}

/// Parses `<tag attr=... >` starting at `start` (which points at '<').
/// Returns the index just past the closing '>'.
fn parse_element(html: &str, start: usize, scan: &mut TemplateScan) -> usize {
    let bytes = html.as_bytes();
    let mut i = start + 1;

    let tag_start = i;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_')
    {
        i += 1;
    }
    let tag = html[tag_start..i].to_string();
    if tag.is_empty() {
        // Stray '<' — treat as text.
        return start + 1;
    }

    let mut element = ElementUsage {
        tag,
        attributes: HashSet::new(),
        classes: HashSet::new(),
    };

    while i < bytes.len() && bytes[i] != b'>' {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'>' {
            break;
        }
        if bytes[i] == b'/' {
            i += 1;
            continue;
        }

        // Attribute name — until whitespace, '=', '/' or '>'.
        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'>'
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let raw_name = &html[name_start..i];

        // Attribute value (optional).
        let mut value = None;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i];
                i += 1;
                let value_start = i;
                while i < bytes.len() && bytes[i] != quote {
                    i += 1;
                }
                value = Some(&html[value_start..i]);
                i += 1; // closing quote
            } else {
                let value_start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
                    i += 1;
                }
                value = Some(&html[value_start..i]);
            }
        }

        process_attribute(raw_name, value, &mut element, &mut scan.pipes);
    }

    scan.elements.push(element);
    i + 1
}

/// Normalizes binding sugar and records the attribute. Binding values are
/// expressions — scanned for pipes.
fn process_attribute(
    raw_name: &str,
    value: Option<&str>,
    element: &mut ElementUsage,
    pipes: &mut HashSet<String>,
) {
    if raw_name.is_empty() {
        return;
    }

    let (normalized, is_binding) = normalize_attribute_name(raw_name);

    if let Some(name) = normalized {
        if name == "class" {
            if let Some(value) = value {
                for class in value.split_ascii_whitespace() {
                    element.classes.insert(class.to_string());
                }
            }
        }
        element.attributes.insert(name);
    }

    if let Some(value) = value {
        if is_binding {
            extract_pipes(value, pipes);
        } else {
            scan_interpolations(value, pipes);
        }
    }
}

/// `[prop]` / `(event)` / `[(model)]` / `*structural` / `attr` →
/// (normalized name, is the value an expression).
fn normalize_attribute_name(raw: &str) -> (Option<String>, bool) {
    if let Some(rest) = raw.strip_prefix("[(").and_then(|r| r.strip_suffix(")]")) {
        return (Some(rest.to_string()), true);
    }
    if let Some(rest) = raw.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        // [attr.x], [style.x], [class.x] are not directive inputs.
        let rest = rest.strip_prefix("attr.").unwrap_or(rest);
        return (Some(rest.to_string()), true);
    }
    if let Some(rest) = raw.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
        return (Some(rest.to_string()), true);
    }
    if let Some(rest) = raw.strip_prefix('*') {
        return (Some(rest.to_string()), true);
    }
    if raw.starts_with('#') || raw.starts_with('@') {
        return (None, false);
    }
    (Some(raw.to_string()), false)
}

/// Angular control flow blocks carry expressions in parentheses:
/// `@if (items | uiHas)`, `@for (item of list | uiSort; track item)`,
/// `@switch (mode | uiMap)`. Extracts pipes from those expressions.
fn scan_control_flow_expressions(text: &str, pipes: &mut HashSet<String>) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'@' {
            i += 1;
            continue;
        }
        // @keyword
        let mut j = i + 1;
        while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
            j += 1;
        }
        if j == i + 1 {
            i += 1;
            continue;
        }
        // Optional whitespace, then a parenthesized expression.
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b'(' {
            i = j;
            continue;
        }
        let expr_start = j + 1;
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
        let expr_end = if depth == 0 { j - 1 } else { j };
        extract_pipes(&text[expr_start..expr_end], pipes);
        i = j;
    }
}

/// Finds `{{ expr }}` interpolations in text and extracts pipes from them.
fn scan_interpolations(text: &str, pipes: &mut HashSet<String>) {
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            break;
        };
        extract_pipes(&after[..end], pipes);
        rest = &after[end + 2..];
    }
}

/// Extracts `| pipeName` occurrences from an expression, skipping `||`.
fn extract_pipes(expr: &str, pipes: &mut HashSet<String>) {
    let bytes = expr.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'|' {
            let double_before = i > 0 && bytes[i - 1] == b'|';
            let double_after = i + 1 < bytes.len() && bytes[i + 1] == b'|';
            if double_before || double_after {
                i += 1;
                continue;
            }
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            let name_start = j;
            while j < bytes.len()
                && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'$')
            {
                j += 1;
            }
            if j > name_start {
                pipes.insert(expr[name_start..j].to_string());
            }
            i = j;
        } else {
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_elements_and_attributes() {
        let scan = scan_template(
            r#"<div class="row main"><ui-button [uiTooltip]="tip" (click)="go()" *uiIf="ok">X</ui-button></div>"#,
        );
        assert_eq!(scan.elements.len(), 2);
        let button = &scan.elements[1];
        assert_eq!(button.tag, "ui-button");
        assert!(button.attributes.contains("uiTooltip"));
        assert!(button.attributes.contains("click"));
        assert!(button.attributes.contains("uiIf"));

        let div = &scan.elements[0];
        assert!(div.classes.contains("row"));
        assert!(div.classes.contains("main"));
    }

    #[test]
    fn scans_pipes_in_interpolations_and_bindings() {
        let scan = scan_template(
            r#"<span [title]="price | uiCurrency:'PLN'">{{ items | uiFilter | async }}</span>
               <p>{{ a || b }}</p>"#,
        );
        assert!(scan.pipes.contains("uiCurrency"));
        assert!(scan.pipes.contains("uiFilter"));
        assert!(scan.pipes.contains("async"));
        assert!(!scan.pipes.contains("b"), "|| must not produce a pipe");
    }

    #[test]
    fn survives_control_flow_blocks_and_comments() {
        let scan = scan_template(
            r#"<!-- comment with <fake-tag> -->
            @if (show) { <fix-badge label="x" /> } @else { <p>none</p> }
            @for (item of items; track item.id) { <fix-row [data]="item" /> }"#,
        );
        let tags: Vec<&str> = scan.elements.iter().map(|e| e.tag.as_str()).collect();
        assert!(tags.contains(&"fix-badge"));
        assert!(tags.contains(&"fix-row"));
        assert!(tags.contains(&"p"));
        assert!(!tags.contains(&"fake-tag"));
    }

    #[test]
    fn structural_directive_value_pipes_are_found() {
        let scan = scan_template(r#"<li *ngFor="let u of users | uiSort"></li>"#);
        assert!(scan.pipes.contains("uiSort"));
    }
}
