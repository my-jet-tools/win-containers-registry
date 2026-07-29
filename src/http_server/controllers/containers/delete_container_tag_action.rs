use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(MyHttpInput)]
pub struct DeleteContainerTagInputModel {
    #[http_path(
        name = "container",
        description = "{container_name}:{tag}. Example: mt4-bridge:0.1.0"
    )]
    pub container: String,

    // default = "" so that a missing header comes back as 401, not as a 400 validation error
    #[http_header(name = "X-API-Key", description = "Write api key", default = "")]
    pub api_key: String,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure)]
pub struct DeleteContainerTagHttpResponse {
    pub hash: String,
    /// True when no other tag referenced the hash and its zip was deleted.
    pub orphan_deleted: bool,
}

#[http_route(
    method: "DELETE",
    route: "/api/containers/v1/tag/{container}",
    controller: "Containers",
    summary: "Delete a container tag",
    description: "Removes the tag from container-info.yaml. The zip is deleted as well, unless another tag still points at the same hash.",
    input_data: "DeleteContainerTagInputModel",
    result: [
        {status_code: 200, description: "Tag is deleted", model: "DeleteContainerTagHttpResponse"},
        {status_code: 400, description: "Invalid container id"},
        {status_code: 401, description: "Invalid or missing api key"},
        {status_code: 404, description: "Container or tag not found"},
    ]
)]
pub struct DeleteContainerTagAction {
    app: Arc<AppContext>,
}

impl DeleteContainerTagAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &DeleteContainerTagAction,
    input_data: DeleteContainerTagInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = crate::flows::delete_container_tag(
        &action.app,
        input_data.container.as_str(),
        input_data.api_key.as_str(),
    )
    .await?;

    HttpOutput::as_json(DeleteContainerTagHttpResponse {
        hash: result.hash,
        orphan_deleted: result.orphan_deleted,
    })
    .into_ok_result(true)
}
