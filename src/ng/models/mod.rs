mod ng_base;
mod ng_component;
pub mod ng_directive;
pub mod ng_element;
mod ng_module;
pub mod ng_pipe;
pub mod ng_references;
pub mod ng_results;
pub mod ng_service;
pub mod ng_template;

pub use ng_component::NgComponentInfo;
pub use ng_module::NgModuleInfo;

pub use ng_results::NgAnalysisResults;
pub use ng_service::NgServiceInfo;
