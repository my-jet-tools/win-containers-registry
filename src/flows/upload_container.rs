use std::sync::Arc;

use crate::app::AppContext;
use crate::models::ContainersError;
use crate::storage::{UploadContainerBlobParams, UploadedContainerTagModel};

const ZIP_SIGNATURE: &[u8] = b"PK";

pub struct UploadContainerParams {
    /// `{container_name}:{tag}` — e.g. `mt4-bridge:0.1.0`
    pub container: String,
    pub api_key: String,
    /// Ip of the uploader — goes into container-info.yaml as `uploaded_by`.
    pub uploaded_by: String,
    pub content: Vec<u8>,
}

pub async fn upload_container(
    app: &Arc<AppContext>,
    params: UploadContainerParams,
) -> Result<UploadedContainerTagModel, ContainersError> {
    crate::scripts::check_api_key(app, params.api_key.as_str())?;

    let container_id = crate::scripts::parse_container_id(params.container.as_str())?;

    if params.content.is_empty() {
        return Err(ContainersError::EmptyContent);
    }

    if !params.content.starts_with(ZIP_SIGNATURE) {
        return Err(ContainersError::NotZipArchive);
    }

    let hash = crate::scripts::calc_sha256_hex(params.content.as_slice());

    let result = app
        .containers_storage
        .upload(UploadContainerBlobParams {
            container_name: container_id.container_name,
            tag: container_id.tag,
            hash,
            content: params.content,
            uploaded_by: params.uploaded_by,
        })
        .await;

    Ok(result)
}
