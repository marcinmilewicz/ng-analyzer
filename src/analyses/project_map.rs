use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub root: PathBuf,
    pub tags: Vec<String>,
    pub project_type: String,
    /// Directories whose every script file is a framework entry point
    /// (expo-router `app/` dirs — file-based routing).
    #[serde(skip)]
    pub entry_dirs: Vec<PathBuf>,
}

/// Maps files to the NX project that owns them (longest matching root wins —
/// nested projects are attributed correctly).
pub struct ProjectCatalog {
    projects: Vec<ProjectInfo>,
}

impl ProjectCatalog {
    pub fn new(mut projects: Vec<ProjectInfo>) -> Self {
        // Longest roots first so nested projects match before their parents.
        projects.sort_by_key(|p| std::cmp::Reverse(p.root.as_os_str().len()));
        Self { projects }
    }

    pub fn projects(&self) -> impl Iterator<Item = &ProjectInfo> {
        self.projects.iter()
    }

    pub fn project_of(&self, file: &Path) -> Option<&ProjectInfo> {
        self.projects
            .iter()
            .find(|project| file.starts_with(&project.root))
    }

    pub fn by_name(&self, name: &str) -> Option<&ProjectInfo> {
        self.projects.iter().find(|project| project.name == name)
    }

    /// File lives in a file-based-routing directory (expo-router) —
    /// the framework consumes it without imports.
    pub fn is_framework_entry(&self, file: &Path) -> bool {
        self.projects
            .iter()
            .flat_map(|project| project.entry_dirs.iter())
            .any(|dir| file.starts_with(dir))
    }
}

/// Test sources use production code without making it "production-used".
pub fn is_test_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    name.contains(".spec.") || name.contains(".test.") || name.ends_with("_test.ts")
}

/// Next.js App Router file conventions: the framework consumes these files'
/// exports (default component, metadata, generateStaticParams, …) without
/// any import statement.
const NEXT_CONVENTION_STEMS: &[&str] = &[
    "page",
    "layout",
    "route",
    "template",
    "loading",
    "error",
    "global-error",
    "not-found",
    "default",
    "middleware",
    // Next.js 16 renamed middleware.ts → proxy.ts.
    "proxy",
    "instrumentation",
    "opengraph-image",
    "twitter-image",
    "icon",
    "apple-icon",
    "sitemap",
    "robots",
    "manifest",
];

/// Application entry points are reachability roots, never dead code:
/// bundler entries, framework file conventions and tool config files.
pub fn is_entry_file(path: &Path) -> bool {
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) else {
        return false;
    };

    if name == "main.ts" || name == "main.tsx" || name == "polyfills.ts" {
        return true;
    }

    // *.config.ts / *.config.js — consumed by tools; *.setup.ts and
    // global-setup/teardown — Playwright/Jest hooks referenced by string;
    // *.stories.* — loaded by Storybook via glob, not imports.
    if name.contains(".config.") || name.contains(".setup.") || name.contains(".stories.") {
        return true;
    }
    if name.starts_with("global-setup.") || name.starts_with("global-teardown.") {
        return true;
    }

    let is_script = path
        .extension()
        .is_some_and(|ext| ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx");
    if !is_script {
        return false;
    }
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };
    NEXT_CONVENTION_STEMS.contains(&stem)
}
