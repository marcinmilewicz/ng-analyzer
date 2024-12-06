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

/// Normalizes a file path by:
/// - Resolving '..' (parent directory) references
/// - Removing '.' (current directory) references
/// - Maintaining relative/absolute path status
///
/// # Examples:
/// ```
/// let path = "./../ddd-hrm/libs/shared/ui/src/./lib/badge/badge.component.ts";
/// let normalized = normalize_path(path);
/// assert_eq!(normalized.to_str().unwrap(), "../ddd-hrm/libs/shared/ui/src/lib/badge/badge.component.ts");
/// ```
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                if !components.is_empty()
                    && components.last() != Some(&std::path::Component::ParentDir)
                {
                    components.pop();
                } else {
                    components.push(component);
                }
            }
            std::path::Component::CurDir => {
                // Skip '.' components
                continue;
            }
            _ => components.push(component),
        }
    }

    components
        .iter()
        .fold(PathBuf::new(), |mut result, &component| {
            result.push(component);
            result
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        let test_cases = vec![
            (
                "./../ddd-hrm/libs/shared/ui/src/./lib/badge/badge.component.ts",
                "../ddd-hrm/libs/shared/ui/src/lib/badge/badge.component.ts",
            ),
            ("./test/./path/../file.txt", "test/file.txt"),
            ("../../../test.txt", "../../../test.txt"),
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
