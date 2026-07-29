use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;

use crate::app::AppContext;

#[derive(MyHttpInput)]
pub struct DownloadContainerInputModel {
    #[http_path(
        name = "container",
        description = "{container_name}:{tag}. Example: mt4-bridge:0.1.0"
    )]
    pub container: String,
}

#[http_route(
    method: "GET",
    route: "/api/containers/v1/download/{container}",
    controller: "Containers",
    summary: "Download a container",
    description: "Resolves the tag into a hash and returns the {hash}.zip content as a file",
    input_data: "DownloadContainerInputModel",
    result: [
        {status_code: 200, description: "Zip archive of the container tag"},
        {status_code: 400, description: "Invalid container id"},
        {status_code: 404, description: "Container or tag not found"},
    ]
)]
pub struct DownloadContainerAction {
    app: Arc<AppContext>,
}

impl DownloadContainerAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &DownloadContainerAction,
    input_data: DownloadContainerInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result =
        crate::flows::download_container(&action.app, input_data.container.as_str()).await?;

    HttpOutput::as_file(result.file_name, result.content).into_ok_result(true)
}
