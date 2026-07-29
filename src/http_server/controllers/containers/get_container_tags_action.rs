use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(MyHttpInput)]
pub struct GetContainerTagsInputModel {
    #[http_path(
        name = "containerName",
        description = "Container name. Example: mt4-bridge"
    )]
    pub container_name: String,

    // default = "" so that a missing header comes back as 401, not as a 400 validation error
    #[http_header(name = "X-API-Key", description = "Api key", default = "")]
    pub api_key: String,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure)]
pub struct ContainerTagHttpModel {
    pub tag: String,
    pub hash: String,
    pub size: u64,
    pub uploaded_at: String,
    pub uploaded_by: String,
}

#[http_route(
    method: "GET",
    route: "/api/containers/v1/tags/{containerName}",
    controller: "Containers",
    summary: "Get container tags",
    description: "Returns every tag of the container with the hash it points at, taken from container-info.yaml",
    input_data: "GetContainerTagsInputModel",
    result: [
        {status_code: 200, description: "Container tags", model: "Vec<ContainerTagHttpModel>"},
        {status_code: 400, description: "Invalid container name"},
        {status_code: 401, description: "Invalid or missing api key"},
        {status_code: 404, description: "Container not found"},
    ]
)]
pub struct GetContainerTagsAction {
    app: Arc<AppContext>,
}

impl GetContainerTagsAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetContainerTagsAction,
    input_data: GetContainerTagsInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let tags = crate::flows::get_container_tags(
        &action.app,
        input_data.container_name.as_str(),
        input_data.api_key.as_str(),
    )
    .await?;

    let response: Vec<ContainerTagHttpModel> = tags
        .into_iter()
        .map(|itm| ContainerTagHttpModel {
            tag: itm.tag,
            hash: itm.hash,
            size: itm.size,
            uploaded_at: itm.uploaded_at,
            uploaded_by: itm.uploaded_by,
        })
        .collect();

    HttpOutput::as_json(response).into_ok_result(true)
}
