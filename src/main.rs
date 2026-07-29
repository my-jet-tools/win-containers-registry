use std::sync::Arc;

use app::AppContext;

mod app;
mod consts;
mod flows;
mod http_server;
mod models;
mod scripts;
mod settings_reader;
mod storage;

#[tokio::main]
async fn main() {
    let settings = settings_reader::read_settings().await;

    let settings = Arc::new(settings);

    let app = Arc::new(AppContext::new(settings));

    crate::http_server::start_up::setup_server(&app);

    app.states.wait_until_shutdown().await;
}
