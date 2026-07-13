use crate::ng::models::ng_base::NgBaseInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgPipeInfo {
    #[serde(flatten)]
    pub base: NgBaseInfo,
    pub name: String,
    pub pure: bool,
    pub standalone: bool,
}
