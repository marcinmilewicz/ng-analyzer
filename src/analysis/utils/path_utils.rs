pub fn get_relative_path(file_path: &Path, project_root: &Path) -> String {
    file_path
        .strip_prefix(project_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string()
}

pub fn is_node_modules(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "node_modules")
}

use std::path::{Path, PathBuf};

pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path_str = path.as_ref().to_string_lossy();
    let mut parts: Vec<&str> = path_str.split('/').collect();
    let mut result: Vec<String> = Vec::new();

    let needs_dot = path_str.starts_with("../") && !path_str.starts_with("./");
    if needs_dot {
        result.push(".".to_string());
    }

    let mut i = 0;
    while i < parts.len() {
        match parts[i] {
            "." => {
                if i == 0 && !needs_dot {
                    result.push(".".to_string());
                }
            }
            ".." => {
                if !result.is_empty()
                    && result.last() != Some(&".".to_string())
                    && result.last() != Some(&"..".to_string())
                {
                    result.pop();
                } else {
                    result.push("..".to_string());
                }
            }
            "" => {
                if i == 0 && !needs_dot {
                    result.push("".to_string());
                }
            }
            part => result.push(part.to_string()),
        }
        i += 1;
    }

    PathBuf::from(result.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        let test_cases = vec![
            (
                "./../ddd-hrm/libs/shared/ui/src/./lib/badge/badge.component.ts",
                "./../ddd-hrm/libs/shared/ui/src/lib/badge/badge.component.ts",
            ),
            ("./test/./path/../file.txt", "./test/file.txt"),
            ("../../../test.txt", "./../../../test.txt"),
        ];

        for (input, expected) in test_cases {
            let result = normalize_path(input);
            assert_eq!(result.to_str().unwrap(), expected);
        }
    }

    #[test]
    fn test_get_relative_path() {
        let test_cases = vec![
            (
                "/root/project/src/app/component.ts",
                "/root/project",
                "src/app/component.ts",
            ),
            (
                "/root/project/packages/app/src/component.ts",
                "/root/project/packages/app",
                "src/component.ts",
            ),
            (
                "/root/other/src/component.ts",
                "/root/project",
                "/root/other/src/component.ts",
            ),
            (
                "/root/project/component.ts",
                "/root/project",
                "component.ts",
            ),
            (
                "/root/project/app/module.ts",
                "/root/project",
                "app/module.ts",
            ),
        ];

        for (file_path, project_root, expected) in test_cases {
            let file_path = PathBuf::from(file_path);
            let project_root = PathBuf::from(project_root);

            assert_eq!(
                get_relative_path(&file_path, &project_root),
                expected,
                "Failed for file_path: {:?}, project_root: {:?}",
                file_path,
                project_root
            );
        }
    }
}
