use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TSConfig {
    #[serde(rename = "compilerOptions")]
    pub compiler_options: Option<CompilerOptions>,
    pub extends: Option<String>,
}


impl fmt::Display for TSConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "TSConfig {{")?;
        if let Some(extends) = &self.extends {
            writeln!(f, "  extends: {}", extends)?;
        }
        if let Some(options) = &self.compiler_options {
            write!(f, "{}", options)?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompilerOptions {
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    pub paths: Option<HashMap<String, Vec<String>>>,
}

impl fmt::Display for CompilerOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  compilerOptions {{")?;
        if let Some(base_url) = &self.base_url {
            writeln!(f, "    baseUrl: {}", base_url)?;
        }
        if let Some(paths) = &self.paths {
            writeln!(f, "    paths: {{")?;
            for (key, values) in paths {
                writeln!(f, "      {}: [", key)?;
                for value in values {
                    writeln!(f, "        {},", value)?;
                }
                writeln!(f, "      ]")?;
            }
            writeln!(f, "    }}")?;
        }
        write!(f, "  }}")
    }
}
