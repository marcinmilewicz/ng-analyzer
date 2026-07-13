use crate::ng::models::ng_base::NgBaseInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgServiceInfo {
    #[serde(flatten)]
    pub base: NgBaseInfo,
    /// `providedIn` value; None for `@Injectable()` without it
    /// (provided via a providers array somewhere).
    pub provided_in: Option<String>,
}
