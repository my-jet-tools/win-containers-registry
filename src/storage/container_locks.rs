use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

/// Hands out one lock per container name.
///
/// Every mutation of a container (writing the zip blob + updating
/// container-info.yaml + deleting an orphan blob) happens under the container's
/// own lock, so the blob and the yaml can never disagree. Reads take the same
/// lock — otherwise a download could resolve a hash which a concurrent upload
/// is about to delete as an orphan.
pub struct ContainerLocks {
    locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl ContainerLocks {
    pub fn new() -> Self {
        Self {
            locks: Mutex::new(HashMap::new()),
        }
    }

    pub async fn get_lock(&self, container_name: &str) -> Arc<Mutex<()>> {
        let mut locks = self.locks.lock().await;

        if let Some(lock) = locks.get(container_name) {
            return lock.clone();
        }

        let lock = Arc::new(Mutex::new(()));
        locks.insert(container_name.to_string(), lock.clone());
        lock
    }
}
