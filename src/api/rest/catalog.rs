use crate::protocol::{Catalog, ModelCatalogItem};

use super::{ApiError, KimiClient};

impl KimiClient {
    pub fn list_models(&self) -> Result<Catalog<ModelCatalogItem>, ApiError> {
        self.get("/models")
    }
}
