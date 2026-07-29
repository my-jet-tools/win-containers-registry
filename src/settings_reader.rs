use rust_extensions::StrOrString;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SettingsModel {
    /// Root folder where containers are stored. Supports `~`.
    /// Layout: {ContainersPath}/{container_name}/container-info.yaml + {hash}.zip
    #[serde(rename = "ContainersPath")]
    containers_path: String,

    /// Api key required by the writes (upload / delete) and by the browsing
    /// endpoints (containers list / container tags). Download and hash lookup
    /// stay open — a target machine pulls a build without a key.
    /// Not configured means everything is open — same convention as my-files-storage.
    #[serde(rename = "ApiKey")]
    api_key: Option<String>,
}

impl SettingsModel {
    pub fn get_containers_path(&self) -> StrOrString<'_> {
        rust_extensions::file_utils::format_path(self.containers_path.as_str())
    }

    pub fn is_api_key_valid(&self, api_key: &str) -> bool {
        let Some(configured_api_key) = self.api_key.as_ref() else {
            return true;
        };

        configured_api_key == api_key
    }
}

pub async fn read_settings() -> SettingsModel {
    let file_name = rust_extensions::file_utils::format_path(crate::consts::SETTINGS_FILE_NAME);

    let file_content = tokio::fs::read(file_name.as_str()).await;

    if let Err(err) = &file_content {
        panic!(
            "Can't open settings file [{}]. Err: {}",
            file_name.as_str(),
            err
        );
    }

    let file_content = file_content.unwrap();

    serde_yaml::from_slice(file_content.as_slice()).unwrap_or_else(|err| {
        panic!(
            "Can't deserialize settings file [{}]. Err: {}",
            file_name.as_str(),
            err
        )
    })
}
