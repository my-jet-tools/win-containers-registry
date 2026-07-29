use std::sync::Arc;

use crate::app::AppContext;
use crate::models::ContainersError;
use crate::storage::ContainerListItemModel;

pub async fn get_containers_list(
    app: &Arc<AppContext>,
) -> Result<Vec<ContainerListItemModel>, ContainersError> {
    Ok(app.containers_storage.get_containers().await)
}
