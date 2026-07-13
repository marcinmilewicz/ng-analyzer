/// Real-world tsconfig.json / project.json files are JSONC: comments and
/// trailing commas are the norm. Strips both so serde_json can parse the
/// result. String contents (including escapes) are preserved untouched.
pub fn strip_jsonc(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;

    let mut in_string = false;
    while i < bytes.len() {
        let c = bytes[i] as char;

        if in_string {
            out.push(c);
            if c == '\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1] as char);
                i += 2;
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match c {
            '"' => {
                in_string = true;
                out.push(c);
                i += 1;
            }
            '/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
            }
            ',' => {
                // Trailing comma: next non-whitespace (skipping comments)
                // is '}' or ']'.
                let mut j = i + 1;
                loop {
                    while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                        j += 1;
                    }
                    if j + 1 < bytes.len() && bytes[j] == b'/' && bytes[j + 1] == b'/' {
                        while j < bytes.len() && bytes[j] != b'\n' {
                            j += 1;
                        }
                        continue;
                    }
                    if j + 1 < bytes.len() && bytes[j] == b'/' && bytes[j + 1] == b'*' {
                        j += 2;
                        while j + 1 < bytes.len() && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
                            j += 1;
                        }
                        j = (j + 2).min(bytes.len());
                        continue;
                    }
                    break;
                }
                if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                    // drop the comma
                } else {
                    out.push(',');
                }
                i += 1;
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_comments_and_trailing_commas() {
        let input = r#"{
  // line comment
  "a": 1, /* block */ "b": "text with // no comment",
  "c": [1, 2, /* mid */ 3,],
}"#;
        let value: serde_json::Value = serde_json::from_str(&strip_jsonc(input)).unwrap();
        assert_eq!(value["a"], 1);
        assert_eq!(value["b"], "text with // no comment");
        assert_eq!(value["c"][2], 3);
    }

    #[test]
    fn preserves_escaped_quotes_in_strings() {
        let input = r#"{ "path": "say \"hi\" // not a comment" }"#;
        let value: serde_json::Value = serde_json::from_str(&strip_jsonc(input)).unwrap();
        assert_eq!(value["path"], "say \"hi\" // not a comment");
    }

    #[test]
    fn plain_json_passes_through() {
        let input = r#"{ "a": [1, 2], "b": { "c": true } }"#;
        assert_eq!(strip_jsonc(input), input);
    }
}
