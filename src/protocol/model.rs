use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Catalog<T> {
    pub items: Vec<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelCatalogItem {
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub max_context_size: u64,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub support_efforts: Vec<String>,
    #[serde(default)]
    pub default_effort: Option<String>,
}

impl ModelCatalogItem {
    pub fn label(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.model)
    }
}
