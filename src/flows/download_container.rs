use std::sync::Arc;

use crate::app::AppContext;
use crate::models::ContainersError;
use crate::storage::DownloadedContainerTagModel;

/// `container` is `{container_name}:{tag}` — e.g. `mt4-bridge:0.1.0`
pub async fn download_container(
    app: &Arc<AppContext>,
    container: &str,
) -> Result<DownloadedContainerTagModel, ContainersError> {
    let container_id = crate::scripts::parse_container_id(container)?;

    app.containers_storage
        .download(
            container_id.container_name.as_str(),
            container_id.tag.as_str(),
        )
        .await
}
