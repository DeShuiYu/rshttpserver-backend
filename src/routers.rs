use axum::extract::DefaultBodyLimit;
use crate::handlers::{get_files_info_handler, delete_files_handler, upload_files_handler, create_folder_handler, download_files_handler};
use crate::state::AppState;
use axum::Router;
use axum::routing::{get, post,delete};
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::decompression::RequestDecompressionLayer;
pub(crate) fn create_global_router(app_state: AppState) -> Router {
    Router::new()
        .route("/files/info", get(get_files_info_handler))
        .route("/files/info/{*filepath}", get(get_files_info_handler))
        .route("/files/delete/{*filepath}", delete(delete_files_handler))
        .route("/files/upload",post(upload_files_handler))
        .route("/files/upload/{*filepath}",post(upload_files_handler))
        .route("/files/create/folder/{*filepath}",get(create_folder_handler))
        .route("/files/download/{*filepath}",get(download_files_handler))
        .layer(
            ServiceBuilder::new()
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(RequestDecompressionLayer::new())
                .layer(CompressionLayer::new())
                .layer(DefaultBodyLimit::max(1024usize * 1024 * 1024 * 1024)),
        )
        .with_state(app_state)
}
