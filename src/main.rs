

use std::sync::Arc;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::routers::create_global_router;

mod routers;
mod handlers;
mod config;
mod state;
mod error;
mod utils;



#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace,tower_http=trace,axum=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_line_number( true)
                .with_ansi( true)
                .with_thread_ids( true)
                .with_thread_names( true)
        )
        .init();



    let app_config = config::AppConfig::new();
    tracing::info!(">>> {:?}", app_config);
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", &app_config.host, &app_config.port))
        .await
        .expect("Failed to bind to port");

    let app_state = state::AppState::new(Arc::new(app_config));
    let app_router = create_global_router(app_state);
    let app_service = app_router.into_make_service_with_connect_info::<std::net::SocketAddr>();

    tracing::info!(">>> listening on {}", listener.local_addr().expect("Failed to get local address"));
    axum::serve(listener, app_service).await.expect("Failed to start server");
}
