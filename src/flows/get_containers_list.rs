use std::sync::Arc;

use crate::app::AppContext;
use crate::models::ContainersError;
use crate::storage::ContainerListItemModel;

pub async fn get_containers_list(
    app: &Arc<AppContext>,
    api_key: &str,
) -> Result<Vec<ContainerListItemModel>, ContainersError> {
    crate::scripts::check_api_key(app, api_key)?;

    Ok(app.containers_storage.get_containers().await)
}
