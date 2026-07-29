use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(MyHttpInput)]
pub struct GetContainerHashInputModel {
    #[http_path(
        name = "container",
        description = "{container_name}:{tag}. Example: mt4-bridge:0.1.0"
    )]
    pub container: String,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure)]
pub struct GetContainerHashHttpResponse {
    pub container: String,
    pub tag: String,
    pub hash: String,
    pub size: u64,
    pub uploaded_at: String,
    pub uploaded_by: String,
}

#[http_route(
    method: "GET",
    route: "/api/containers/v1/hash/{container}",
    controller: "Containers",
    summary: "Get the hash of a container tag",
    description: "Resolves the tag into the hash of the stored zip, without downloading it",
    input_data: "GetContainerHashInputModel",
    result: [
        {status_code: 200, description: "Container tag info", model: "GetContainerHashHttpResponse"},
        {status_code: 400, description: "Invalid container id"},
        {status_code: 404, description: "Container or tag not found"},
    ]
)]
pub struct GetContainerHashAction {
    app: Arc<AppContext>,
}

impl GetContainerHashAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetContainerHashAction,
    input_data: GetContainerHashInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result =
        crate::flows::get_container_hash(&action.app, input_data.container.as_str()).await?;

    HttpOutput::as_json(GetContainerHashHttpResponse {
        container: result.container_name,
        tag: result.tag,
        hash: result.hash,
        size: result.size,
        uploaded_at: result.uploaded_at,
        uploaded_by: result.uploaded_by,
    })
    .into_ok_result(true)
}
