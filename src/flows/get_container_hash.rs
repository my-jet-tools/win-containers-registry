use std::sync::Arc;

use crate::app::AppContext;
use crate::models::ContainersError;
use crate::storage::ContainerTagModel;

/// `container` is `{container_name}:{tag}` — e.g. `mt4-bridge:0.1.0`
pub async fn get_container_hash(
    app: &Arc<AppContext>,
    container: &str,
) -> Result<ContainerTagModel, ContainersError> {
    let container_id = crate::scripts::parse_container_id(container)?;

    app.containers_storage
        .get_tag(
            container_id.container_name.as_str(),
            container_id.tag.as_str(),
        )
        .await
}
