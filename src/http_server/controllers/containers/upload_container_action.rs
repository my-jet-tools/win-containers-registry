use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;
use crate::flows::UploadContainerParams;

#[derive(MyHttpInput)]
pub struct UploadContainerInputModel {
    #[http_path(
        name = "container",
        description = "{container_name}:{tag}. Example: mt4-bridge:0.1.0"
    )]
    pub container: String,

    // default = "" so that a missing header comes back as 401, not as a 400 validation error
    #[http_header(name = "X-API-Key", description = "Write api key", default = "")]
    pub api_key: String,

    #[http_body_raw(description = "Zip archive with the windows service")]
    pub content: Vec<u8>,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure)]
pub struct UploadContainerHttpResponse {
    pub container: String,
    pub tag: String,
    pub hash: String,
    pub size: u64,
    /// Hash this tag was pointing at before the upload, if it was reassigned.
    pub replaced_hash: Option<String>,
    /// True when the replaced hash became unreferenced and its zip was deleted.
    pub orphan_deleted: bool,
}

#[http_route(
    method: "POST",
    route: "/api/containers/v1/upload/{container}",
    controller: "Containers",
    summary: "Upload a container",
    description: "Body is the zip archive itself. Hash of the content becomes the file name, the tag becomes a pointer to that hash. Re-uploading an existing tag overwrites it.",
    input_data: "UploadContainerInputModel",
    result: [
        {status_code: 200, description: "Container tag is stored", model: "UploadContainerHttpResponse"},
        {status_code: 400, description: "Invalid container id or body"},
        {status_code: 401, description: "Invalid or missing api key"},
    ]
)]
pub struct UploadContainerAction {
    app: Arc<AppContext>,
}

impl UploadContainerAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &UploadContainerAction,
    input_data: UploadContainerInputModel,
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = crate::flows::upload_container(
        &action.app,
        UploadContainerParams {
            container: input_data.container,
            api_key: input_data.api_key,
            uploaded_by: ctx.request.get_ip().get_real_ip_as_string(),
            content: input_data.content,
        },
    )
    .await?;

    HttpOutput::as_json(UploadContainerHttpResponse {
        container: result.container_name,
        tag: result.tag,
        hash: result.hash,
        size: result.size,
        replaced_hash: result.replaced_hash,
        orphan_deleted: result.orphan_deleted,
    })
    .into_ok_result(true)
}
