use axum::extract::DefaultBodyLimit;
use crate::state::AppState;
use axum::Router;
use axum::routing::{get, post, delete, put};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::decompression::RequestDecompressionLayer;
use crate::handlers::{create_entry_handler, delete_entry_handler, download_entry_handler, list_entry_info_handler, rename_entry_handler, upload_entry_handler};

pub(crate) fn create_global_router(app_state: AppState) -> Router {
    Router::new()
        .route("/info/", get(list_entry_info_handler))
        .route("/info/{*epath}", get(list_entry_info_handler))
        .route("/delete/{*epath}", delete(delete_entry_handler))
        .route("/rename/{*oepath}", put(rename_entry_handler))
        .route("/create/{*epath}", get(create_entry_handler))
        .route("/upload/",post(upload_entry_handler))
        .route("/upload/{*epath}",post(upload_entry_handler))
        .route("/download/{*epath}",get(download_entry_handler))
        .layer(
            ServiceBuilder::new()
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(RequestDecompressionLayer::new())
                .layer(CompressionLayer::new())
                .layer(DefaultBodyLimit::max(1024usize * 1024 * 1024 * 1024)),
        )
        .with_state(app_state)
}
