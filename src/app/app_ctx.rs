use std::sync::Arc;

use rust_extensions::AppStates;

use crate::settings_reader::SettingsModel;
use crate::storage::ContainersStorage;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct AppContext {
    pub containers_storage: ContainersStorage,
    pub settings: Arc<SettingsModel>,
    pub states: Arc<AppStates>,
}

impl AppContext {
    pub fn new(settings: Arc<SettingsModel>) -> Self {
        let containers_path = settings.get_containers_path().to_string();

        Self {
            containers_storage: ContainersStorage::new(containers_path),
            settings,
            states: Arc::new(AppStates::create_initialized()),
        }
    }
}
