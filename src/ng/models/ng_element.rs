use crate::ng::models::ng_directive::NgDirectiveInfo;
use crate::ng::models::ng_pipe::NgPipeInfo;
use crate::ng::models::{NgComponentInfo, NgModuleInfo, NgServiceInfo};

#[derive(Debug, Clone)]
pub enum NgElement {
    Component(NgComponentInfo),
    Directive(NgDirectiveInfo),
    Pipe(NgPipeInfo),
    Module(NgModuleInfo),
    Service(NgServiceInfo),
}
