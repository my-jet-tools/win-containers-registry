use std::sync::Arc;

use my_http_server::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: &Arc<AppContext>) -> ControllersMiddleware {
    let mut result = ControllersMiddleware::new(None, None);

    result.register_get_action(Arc::new(super::system::PingAction::new(app.clone())));

    result.register_post_action(Arc::new(super::containers::UploadContainerAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::containers::DownloadContainerAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::containers::GetContainersListAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::containers::GetContainerTagsAction::new(
        app.clone(),
    )));

    result.register_get_action(Arc::new(super::containers::GetContainerHashAction::new(
        app.clone(),
    )));

    result.register_delete_action(Arc::new(super::containers::DeleteContainerTagAction::new(
        app.clone(),
    )));

    result
}
