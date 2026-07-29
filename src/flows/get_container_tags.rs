use std::sync::Arc;

use crate::app::AppContext;
use crate::models::ContainersError;
use crate::storage::ContainerTagModel;

pub async fn get_container_tags(
    app: &Arc<AppContext>,
    container_name: &str,
    api_key: &str,
) -> Result<Vec<ContainerTagModel>, ContainersError> {
    crate::scripts::check_api_key(app, api_key)?;

    crate::scripts::validate_name("containerName", container_name)?;

    app.containers_storage.get_tags(container_name).await
}
