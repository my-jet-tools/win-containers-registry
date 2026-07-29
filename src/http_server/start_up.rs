use std::net::SocketAddr;
use std::sync::Arc;

use my_http_server::MyHttpServer;
use my_http_server::controllers::swagger::SwaggerMiddleware;

use crate::app::AppContext;

pub fn setup_server(app: &Arc<AppContext>) {
    let http_port = SocketAddr::from(([0, 0, 0, 0], crate::consts::HTTP_PORT));

    println!("Starting HTTP server at Tcp({:?})", http_port);

    let mut http_server = MyHttpServer::new(http_port);

    let controllers = Arc::new(crate::http_server::controllers::builder::build(app));

    let swagger_middleware = Arc::new(SwaggerMiddleware::new(
        controllers.clone(),
        crate::app::APP_NAME.to_string(),
        crate::app::APP_VERSION.to_string(),
    ));

    http_server.add_middleware(swagger_middleware);
    http_server.add_middleware(controllers);

    http_server.start(app.states.clone(), my_logger::LOGGER.clone());
}
