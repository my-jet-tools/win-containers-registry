use std::sync::Arc;

use crate::app::AppContext;
use crate::models::ContainersError;
use crate::storage::DeletedContainerTagModel;

/// `container` is `{container_name}:{tag}` — e.g. `mt4-bridge:0.1.0`
pub async fn delete_container_tag(
    app: &Arc<AppContext>,
    container: &str,
    api_key: &str,
) -> Result<DeletedContainerTagModel, ContainersError> {
    crate::scripts::check_api_key(app, api_key)?;

    let container_id = crate::scripts::parse_container_id(container)?;

    app.containers_storage
        .delete_tag(
            container_id.container_name.as_str(),
            container_id.tag.as_str(),
        )
        .await
}
