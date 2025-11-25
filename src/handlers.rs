use crate::state::AppState;
use crate::utils::format_bytes;
use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::io::{AsyncWriteExt, BufWriter};

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    code: i32,
    message: String,
    data: Option<Value>,
}

pub(crate) async fn get_files_info_handler(
    path: Option<Path<String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let relative_path = if let Some(Path(filepath)) = path {
        PathBuf::from(filepath)
    } else {
        PathBuf::from(".")
    };
    let absolute_path = if let Ok(path) = state
        .config
        .root_dirpath
        .join(&relative_path)
        .canonicalize()
    {
        path
    } else {
        tracing::warn!(">>> {:?} not found", &relative_path);
        PathBuf::from(&state.config.root_dirpath)
    };

    let cpath = absolute_path
        .strip_prefix(&state.config.root_dirpath)
        .map_or_else(
            |_| "".to_string(),
            |path| path.to_string_lossy().to_string(),
        );

    let mut files_info = vec![];
    if absolute_path.is_file() || absolute_path.is_symlink() {
        let name = absolute_path
            .file_name()
            .map_or_else(|| "".to_string(), |name| name.to_string_lossy().to_string());
        let path = &cpath;
        let size = absolute_path
            .metadata()
            .map_or_else(|_| 0, |metadata| metadata.len());
        let modified = absolute_path
            .metadata()
            .and_then(|metadata| metadata.modified())
            .map_or_else(
                |_| 0,
                |modified| {
                    modified
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .map_or_else(|_| 0, |duration| duration.as_secs())
                },
            );
        let created = absolute_path
            .metadata()
            .and_then(|metadata| metadata.created())
            .map_or_else(
                |_| 0,
                |created| {
                    created
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .map_or_else(|_| 0, |duration| duration.as_secs())
                },
            );

        files_info.push(json!( {
            "name": name,
            "path": path,
            "size": size,
            "modified": modified,
            "created":created,
        }));
    } else if absolute_path.is_dir() {
        let f_vec = std::fs::read_dir(&absolute_path)
            .unwrap()
            .into_iter()
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                let path = &cpath;
                let size = entry
                    .metadata()
                    .map_or_else(|_| 0, |metadata| metadata.len());
                let modified = entry
                    .metadata()
                    .and_then(|metadata| metadata.modified())
                    .map_or_else(
                        |_| 0,
                        |modified| {
                            modified
                                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                .map_or_else(|_| 0, |duration| duration.as_secs())
                        },
                    );
                let created = entry
                    .metadata()
                    .and_then(|metadata| metadata.created())
                    .map_or_else(
                        |_| 0,
                        |created| {
                            created
                                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                .map_or_else(|_| 0, |duration| duration.as_secs())
                        },
                    );
                json!( {
                    "name": name,
                    "path": path,
                    "size": size,
                    "modified": modified,
                    "created":created,
                })
            })
            .collect::<Vec<_>>();
        files_info.extend(f_vec);
    }
    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 200,
            message: format!("{} success", cpath),
            data: Some(json!({"files":files_info})),
        }),
    )
}

pub(crate) async fn delete_files_handler(
    Path(filepath): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let relative_path = if !filepath.is_empty() {
        PathBuf::from(filepath)
    } else {
        PathBuf::from(".")
    };
    let absolute_path = if let Ok(path) = state
        .config
        .root_dirpath
        .join(&relative_path)
        .canonicalize()
    {
        path
    } else {
        tracing::warn!(">>> {:?} not found", &relative_path);
        PathBuf::from(&state.config.root_dirpath)
    };

    let cpath = absolute_path
        .strip_prefix(&state.config.root_dirpath)
        .map_or_else(
            |_| "".to_string(),
            |path| path.to_string_lossy().to_string(),
        );

    if (absolute_path.is_file() || absolute_path.is_symlink())
        && let Ok(_) = std::fs::remove_file(&absolute_path)
    {
        return (
            StatusCode::OK,
            Json(ApiResponse {
                code: 200,
                message: format!("success delete {}", &cpath),
                data: None,
            }),
        );
    } else if absolute_path.is_dir()
        && let Ok(_) = std::fs::remove_dir_all(&absolute_path)
    {
        return (
            StatusCode::OK,
            Json(ApiResponse {
                code: 200,
                message: format!("success delete {}", &cpath),
                data: None,
            }),
        );
    }

    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse {
            code: 404,
            message: format!("{} not found", &cpath),
            data: None,
        }),
    )
}

pub(crate) async fn upload_files_handler(
    path: Option<Path<String>>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let relative_path = if let Some(Path(filepath)) = path {
        PathBuf::from(filepath)
    } else {
        PathBuf::from(".")
    };
    let absolute_path = if let Ok(path) = state
        .config
        .root_dirpath
        .join(&relative_path)
        .canonicalize()
    {
        path
    } else {
        tracing::warn!(">>> {:?} not found", &relative_path);
        PathBuf::from(&state.config.root_dirpath)
    };

    let cpath = absolute_path
        .strip_prefix(&state.config.root_dirpath)
        .map_or_else(
            |_| "".to_string(),
            |path| path.to_string_lossy().to_string(),
        );
    while let Ok(Some(mut field)) = multipart.next_field().await {
        if let Some(name) = field.name()
            && let Some(file_name) = field.file_name()
        {
            tracing::info!(">>> name:{} file_name:{}", &name, &file_name);
            let file_name = PathBuf::from(&file_name)
                .file_name()
                .map_or_else(|| "unknow".to_string(), |m| m.to_string_lossy().to_string());
            let save_path = absolute_path.join(&file_name);
            tracing::info!(">>> start save {} to {:?}", &file_name, &save_path);

            if let Ok(file) = tokio::fs::File::create(&save_path).await {
                let mut stream_writer = BufWriter::new(file);
                let mut total_bytes = 0;
                while let Ok(Some(chunk)) = field.chunk().await {
                    total_bytes += chunk.len();
                    // 关键优化 3: 流式读取 (Chunked)
                    // 只要网络还在传数据，这个循环就会继续。内存中永远只保留当前的一个 chunk。
                    let _ = stream_writer.write_all(&chunk).await;

                    // 必须 flush 确保缓冲区的数据全部落盘
                    let _ = stream_writer.flush().await;
                }
                tracing::info!(
                    "success save file: {:?}, size: {}",
                    &save_path,
                    format_bytes(total_bytes as u64)
                );
            } else {
                tracing::error!(">>> create {:?} error", &save_path);
            }
        } else {
            tracing::warn!(">>> no name or file_name");
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 200,
            message: format!("success upload {}", &cpath),
            data: None,
        }),
    )
}

pub(crate) async fn create_folder_handler(
    Path(filepath): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let relative_path = PathBuf::from(filepath);
    let absolute_path = state.config.root_dirpath.join(&relative_path);
    let cpath = absolute_path
        .strip_prefix(&state.config.root_dirpath)
        .map_or_else(
            |_| "".to_string(),
            |path| path.to_string_lossy().to_string(),
        );

    if !absolute_path.starts_with(&state.config.root_dirpath) {
        (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                code: 403,
                message: "forbidden".to_string(),
                data: None,
            }),
        )
    } else {
        if let Ok(_) = tokio::fs::create_dir_all(&absolute_path).await {
            (
                StatusCode::OK,
                Json(ApiResponse {
                    code: 200,
                    message: format!("success create {} folder", cpath),
                    data: None,
                }),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    code: 500,
                    message: format!("create {} folder error", cpath),
                    data: None,
                }),
            )
        }
    }
}

pub(crate) async fn download_files_handler(
    Path(filepath): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let absolute_path = if let Ok(path) = state.config.root_dirpath.join(&filepath).canonicalize() {
        path
    }else {
        return Err((StatusCode::NOT_FOUND, Json(ApiResponse{
            code: 404,
            message: format!("{} not found",&filepath),
            data: None,
        })));
    };
    let filename = absolute_path
        .file_name()
        .map_or_else(
            || "unknow".to_string(),
            |m| m.to_string_lossy().to_string(),
        );
    let file = match tokio::fs::File::open(&absolute_path).await {
        Ok(file) => file,
        Err(e) => {
            return Err((StatusCode::NOT_FOUND, Json(ApiResponse{
                code: 404,
                message: format!("{}", e.to_string()),
                data: None,
            })));
        },
    };
    // 设置下载时显示的文件名（关键！）
    let headers = [
        (header::CONTENT_DISPOSITION, format!(r#"attachment; filename="{}""#, filename)),
        (header::CONTENT_TYPE, "application/octet-stream".to_string()),
    ];
    let stream = tokio_util::io::ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);
    Ok((headers, body))
}
