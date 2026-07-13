use crate::analyses::project_map::ProjectCatalog;
use crate::analyses::stats::ProjectDependencyInfo;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// NX-style tag boundary rules, read from `nx-analyzer.json` at the workspace
/// root (legacy name `ng-analyzer.json` still accepted):
///
/// ```json
/// {
///   "boundaries": [
///     { "sourceTag": "type:ui", "allowedTags": ["type:ui", "type:util"] }
///   ]
/// }
/// ```
#[derive(Debug, Deserialize, Default)]
pub struct AnalyzerConfig {
    #[serde(default)]
    pub boundaries: Vec<BoundaryRule>,
}

#[derive(Debug, Deserialize)]
pub struct BoundaryRule {
    #[serde(rename = "sourceTag")]
    pub source_tag: String,
    #[serde(rename = "allowedTags")]
    pub allowed_tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BoundaryViolation {
    pub from: String,
    pub to: String,
    pub source_tag: String,
    pub allowed_tags: Vec<String>,
    pub to_tags: Vec<String>,
}

pub fn load_config(workspace_root: &Path) -> AnalyzerConfig {
    // Legacy pre-rename config name kept as a fallback.
    let Some((path, content)) = ["nx-analyzer.json", "ng-analyzer.json"]
        .iter()
        .find_map(|name| {
            let path = workspace_root.join(name);
            std::fs::read_to_string(&path).ok().map(|c| (path, c))
        })
    else {
        return AnalyzerConfig::default();
    };
    match serde_json::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("⚠️ Invalid {}: {}", path.display(), e);
            AnalyzerConfig::default()
        }
    }
}

/// Checks every cross-project dependency against the tag rules. A project may
/// match several rules (one per matching source tag) — each is enforced.
pub fn check_boundaries(
    dependencies: &[ProjectDependencyInfo],
    catalog: &ProjectCatalog,
    config: &AnalyzerConfig,
) -> Vec<BoundaryViolation> {
    if config.boundaries.is_empty() {
        return Vec::new();
    }

    let mut violations = Vec::new();

    for dep in dependencies {
        let (Some(from), Some(to)) = (catalog.by_name(&dep.from), catalog.by_name(&dep.to)) else {
            continue;
        };

        for rule in &config.boundaries {
            if !from.tags.contains(&rule.source_tag) {
                continue;
            }
            let allowed = rule
                .allowed_tags
                .iter()
                .any(|allowed| allowed == "*" || to.tags.contains(allowed));
            if !allowed {
                violations.push(BoundaryViolation {
                    from: from.name.clone(),
                    to: to.name.clone(),
                    source_tag: rule.source_tag.clone(),
                    allowed_tags: rule.allowed_tags.clone(),
                    to_tags: to.tags.clone(),
                });
            }
        }
    }

    violations
        .sort_by(|a, b| (&a.from, &a.to, &a.source_tag).cmp(&(&b.from, &b.to, &b.source_tag)));
    violations
}
