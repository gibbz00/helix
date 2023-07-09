use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub enum DebugConfigCompletion {
    Named(String),
    Advanced(AdvancedCompletion),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AdvancedCompletion {
    pub name: Option<String>,
    pub completion: Option<String>,
    pub default: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged)]
pub enum DebugArgumentValue {
    String(String),
    Array(Vec<String>),
    Boolean(bool),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DebugTemplate {
    pub name: String,
    pub request: String,
    pub completion: Vec<DebugConfigCompletion>,
    pub args: HashMap<String, DebugArgumentValue>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DebugAdapterConfig {
    pub name: String,
    pub transport: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub port_arg: Option<String>,
    pub templates: Vec<DebugTemplate>,
    #[serde(default)]
    pub quirks: DebuggerQuirks,
}

// Different workarounds for adapters' differences
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize)]
pub struct DebuggerQuirks {
    #[serde(default)]
    pub absolute_paths: bool,
}
