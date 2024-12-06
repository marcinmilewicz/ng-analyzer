use crate::ng::models::{NgAnalysisResults, NgComponentInfo, NgServiceInfo};

pub struct NgReporter;

impl NgReporter {
    pub fn print_analysis(results: &NgAnalysisResults) {
        println!("🔍 Analyzing Angular project...\n");

        println!("\n📊 Analysis Results:");

        for component in &results.components {
            Self::print_component(component);
        }

        for service in &results.services {
            Self::print_service(service);
        }

        println!("Components found: {}", results.components.len());
        println!("\nServices found: {}", results.services.len());
        println!("\nModules found: {}", results.modules.len());
        println!("\nDirectives found: {}", results.directives.len());
        println!("\nPipes found: {}", results.pipes.len());
    }

    fn print_component(component: &NgComponentInfo) {
        println!("  - {} ({})", component.base.name, component.base.relative_path);
        println!("    Selector: {}", component.selector);
        println!("    Package: {}", component.base.package_name);
        println!("    Template: {}", component.template_path);


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
