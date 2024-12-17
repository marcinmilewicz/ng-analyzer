use crate::analysis::models::import::ResolvedImport;
use crate::ng::models::ng_base::NgBaseInfo;
use crate::ng::models::ng_directive::NgDirectiveInfo;
use crate::ng::models::ng_other::NgOtherInfo;
use crate::ng::models::ng_pipe::NgPipeInfo;
use crate::ng::models::ng_spec::NgTestSpecInfo;
use crate::ng::models::{NgComponentInfo, NgModuleInfo, NgServiceInfo};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum NgElement {
    Component(NgComponentInfo),
    Directive(NgDirectiveInfo),
    Pipe(NgPipeInfo),
    Module(NgModuleInfo),
    Service(NgServiceInfo),
    TestSpec(NgTestSpecInfo),
    Other(NgOtherInfo),
}

impl NgElement {
    pub fn get_base(&self) -> &NgBaseInfo {
        match self {
            NgElement::Component(c) => &c.base,
            NgElement::Directive(d) => &d.base,
            NgElement::Pipe(p) => &p.base,
            NgElement::Module(m) => &m.base,
            NgElement::Service(s) => &s.base,
            NgElement::TestSpec(t) => &t.base,
            NgElement::Other(o) => &o.base,
        }
    }

    pub fn get_base_mut(&mut self) -> &mut NgBaseInfo {
        match self {
            NgElement::Component(c) => &mut c.base,
            NgElement::Directive(d) => &mut d.base,
            NgElement::Pipe(p) => &mut p.base,
            NgElement::Module(m) => &mut m.base,
            NgElement::Service(s) => &mut s.base,
            NgElement::TestSpec(t) => &mut t.base,
            NgElement::Other(o) => &mut o.base,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.get_base().name
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.get_base().source_path
    }

    pub fn get_imports(&self) -> &Vec<ResolvedImport> {
        &self.get_base().imports
    }

    pub fn get_relative_path(&self) -> &str {
        &self.get_base().relative_path
    }

    pub fn get_package_name(&self) -> &str {
        &self.get_base().package_name
    }

    pub fn is_component(&self) -> bool {
        matches!(self, NgElement::Component(_))
    }

    pub fn is_directive(&self) -> bool {
        matches!(self, NgElement::Directive(_))
    }

    pub fn is_pipe(&self) -> bool {
        matches!(self, NgElement::Pipe(_))
    }

    pub fn is_module(&self) -> bool {
        matches!(self, NgElement::Module(_))
    }

    pub fn is_service(&self) -> bool {
        matches!(self, NgElement::Service(_))
    }

    pub fn as_component(&self) -> Option<&NgComponentInfo> {
        match self {
            NgElement::Component(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_directive(&self) -> Option<&NgDirectiveInfo> {
        match self {
            NgElement::Directive(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_pipe(&self) -> Option<&NgPipeInfo> {
        match self {
            NgElement::Pipe(p) => Some(p),
            _ => None,
        }
    }

    pub fn as_module(&self) -> Option<&NgModuleInfo> {
        match self {
            NgElement::Module(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_service(&self) -> Option<&NgServiceInfo> {
        match self {
            NgElement::Service(s) => Some(s),
            _ => None,
        }
    }
}
