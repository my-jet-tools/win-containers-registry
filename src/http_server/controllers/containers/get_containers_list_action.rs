use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

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
    result: [
        {status_code: 200, description: "Containers list", model: "Vec<ContainerListItemHttpModel>"},
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
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let containers = crate::flows::get_containers_list(&action.app).await?;

    let response: Vec<ContainerListItemHttpModel> = containers
        .into_iter()
        .map(|itm| ContainerListItemHttpModel {
            container: itm.container,
            tags_amount: itm.tags_amount,
        })
        .collect();

    HttpOutput::as_json(response).into_ok_result(true)
}
