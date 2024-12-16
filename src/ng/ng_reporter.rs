use crate::ng::models::{NgAnalysisResults, NgComponentInfo, NgServiceInfo};

pub struct NgReporter;

impl NgReporter {
    pub fn print_analysis(results: &NgAnalysisResults) {
        println!("🔍 Analyzing Angular project...\n");
        println!("Components found: {}", results.components.len());
        println!("\nServices found: {}", results.services.len());
        println!("\nModules found: {}", results.modules.len());
        println!("\nDirectives found: {}", results.directives.len());
        println!("\nPipes found: {}", results.pipes.len());
    }

    pub fn print_filtered_components(results: &NgAnalysisResults, component_names: &[String]) {
        println!("\n🔍 Analyzing specific components...\n");

        let filtered_components: Vec<&NgComponentInfo> = results
            .components
            .iter()
            .filter(|comp| component_names.contains(&comp.base.name))
            .collect();

        println!(
            "📊 Found Components ({}/{}):",
            filtered_components.len(),
            component_names.len()
        );

        for component in filtered_components {
            Self::print_component(component);
        }

        let not_found: Vec<&String> = component_names
            .iter()
            .filter(|name| !results.components.iter().any(|c| &c.base.name == *name))
            .collect();

        if !not_found.is_empty() {
            println!("\n⚠️ Components not found:");
            for name in not_found {
                println!("  - {}", name);
            }
        }
    }

    fn print_component(component: &NgComponentInfo) {
        println!(
            "  - {} ({})",
            component.base.name, component.base.relative_path
        );
        println!("    Selector: {}", component.selector);
        println!("    Package: {}", component.base.package_name);
        println!("    Template: {}", component.template_path);
        println!(
            "    Usage in templates: {}",
            component.base.references.used_in_templates.len()
        );

        if !component.base.imports.is_empty() {
            println!("    Imports:");
            for import in &component.base.imports {
                println!("      - {}", import);
            }
        }
    }

    fn print_service(service: &NgServiceInfo) {
        println!("  - {} ({})", service.base.name, service.base.relative_path);
        println!("    Package: {}", service.base.package_name);

        if !service.base.imports.is_empty() {
            println!("    Imports:");
            for import in &service.base.imports {
                println!("      - {}", import);
            }
        }
    }
}
