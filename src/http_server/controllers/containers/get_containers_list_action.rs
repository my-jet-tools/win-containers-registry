use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(MyHttpInput)]
pub struct GetContainersListInputModel {
    // default = "" so that a missing header comes back as 401, not as a 400 validation error
    #[http_header(name = "X-API-Key", description = "Api key", default = "")]
    pub api_key: String,
}

#[derive(Serialize, Deserialize, MyHttpObjectStructure)]
pub struct ContainerListItemHttpModel {
    pub container: String,
    pub tags_amount: usize,
}

#[http_route(
    method: "GET",
    route: "/api/containers/v1/list",
    controller: "Containers",
    summary: "Get containers list",
    description: "Returns every container found in the storage folder",
    input_data: "GetContainersListInputModel",
    result: [
        {status_code: 200, description: "Containers list", model: "Vec<ContainerListItemHttpModel>"},
        {status_code: 401, description: "Invalid or missing api key"},
    ]
)]
pub struct GetContainersListAction {
    app: Arc<AppContext>,
}

impl GetContainersListAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetContainersListAction,
    input_data: GetContainersListInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let containers =
        crate::flows::get_containers_list(&action.app, input_data.api_key.as_str()).await?;

    let response: Vec<ContainerListItemHttpModel> = containers
        .into_iter()
        .map(|itm| ContainerListItemHttpModel {
            container: itm.container,
            tags_amount: itm.tags_amount,
        })
        .collect();

    HttpOutput::as_json(response).into_ok_result(true)
}
