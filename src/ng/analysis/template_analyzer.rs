use regex::Regex;
use std::collections::HashSet;

use crate::ng::models::ng_template::TemplateUsage;

pub struct TemplateParser;

impl TemplateParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_template(&self, template: &str) -> TemplateUsage {
        let components = self.find_components(template);
        let directives = self.find_directives(template);
        let pipes = self.find_pipes(template);

        TemplateUsage {
            components: components.into_iter().collect(),
            pipes: pipes.into_iter().collect(),
            directives: directives.into_iter().collect(),
        }
    }

    fn find_components(&self, template: &str) -> HashSet<String> {
        let component_re = Regex::new(r"<([a-z][a-z0-9]*-[a-z0-9-]+)").unwrap();
        component_re
            .captures_iter(template)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn find_directives(&self, template: &str) -> HashSet<String> {
        let mut directives = HashSet::new();

        let structural_directive_regex = Regex::new(r"\*ng([A-Z][a-zA-Z]*)").unwrap();
        directives.extend(
            structural_directive_regex
                .captures_iter(template)
                .filter_map(|cap| cap.get(1))
                .map(|m| format!("ng{}", m.as_str())),
        );

        let attribute_directive_regex = Regex::new(r"\[(ng[A-Z][a-zA-Z]*)\]").unwrap();
        directives.extend(
            attribute_directive_regex
                .captures_iter(template)
                .filter_map(|cap| cap.get(1))
                .map(|m| m.as_str().to_string()),
        );

        directives
    }

    fn find_pipes(&self, template: &str) -> HashSet<String> {
        let pipe_regex = Regex::new(r"\|\s*([a-zA-Z][a-zA-Z0-9]*)").unwrap();
        pipe_regex
            .captures_iter(template)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_parsing() {
        let template = r#"
            <app-header [title]="'Welcome'">
                <div *ngIf="showContent">
                    {{ value | uppercase | date }}
                    <custom-element [ngClass]="classes">
                        <ng-container *ngFor="let item of items">
                            {{ item.name | titlecase }}
                        </ng-container>
                    </custom-element>
                </div>
            </app-header>
        "#;

        let parser = TemplateParser::new();
        let usage = parser.parse_template(template);

        assert!(usage.components.contains(&"app-header".to_string()));
        assert!(usage.components.contains(&"custom-element".to_string()));
        assert!(usage.directives.contains(&"ngIf".to_string()));
        assert!(usage.directives.contains(&"ngFor".to_string()));
        assert!(usage.directives.contains(&"ngClass".to_string()));
        assert!(usage.pipes.contains(&"uppercase".to_string()));
        assert!(usage.pipes.contains(&"titlecase".to_string()));
        assert!(usage.pipes.contains(&"date".to_string()));
    }
}
