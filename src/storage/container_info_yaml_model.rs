use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Content of `{containers_path}/{container_name}/container-info.yaml`.
/// This file is the only metadata storage of the service — there is no database.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContainerInfoYamlModel {
    pub container: String,
    pub tags: BTreeMap<String, ContainerTagYamlModel>,
}

impl ContainerInfoYamlModel {
    pub fn new(container_name: &str) -> Self {
        Self {
            container: container_name.to_string(),
            tags: BTreeMap::new(),
        }
    }

    /// True when at least one tag still points at the given hash.
    pub fn has_reference_to_hash(&self, hash: &str) -> bool {
        self.tags.values().any(|itm| itm.hash == hash)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContainerTagYamlModel {
    pub hash: String,
    pub size: u64,
    pub uploaded_at: String,
    pub uploaded_by: String,
}
