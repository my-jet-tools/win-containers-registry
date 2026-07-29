use std::sync::Arc;

use my_http_server::macros::*;
use my_http_server::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(Serialize, Deserialize, MyHttpObjectStructure)]
pub struct PingHttpResponse {
    pub result: String,
}

#[http_route(
    method: "GET",
    route: "/api/system/v1/ping",
    controller: "System",
    summary: "Liveness probe",
    description: "Returns 'pong' when the service is up",
    result: [
        {status_code: 200, description: "Service is alive", model: PingHttpResponse},
    ]
)]
pub struct PingAction {
    _app: Arc<AppContext>,
}

impl PingAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { _app: app }
    }
}

async fn handle_request(
    _action: &PingAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    HttpOutput::as_json(PingHttpResponse {
        result: "pong".to_string(),
    })
    .into_ok_result(true)
}
