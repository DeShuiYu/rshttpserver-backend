use crate::state::AppState;
use crate::utils::format_bytes;
use axum::Json;
use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::{HeaderMap, header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::SeekFrom;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio_util::io::ReaderStream;

#[derive(Serialize)]
pub(crate) struct ApiResponse {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[derive(Serialize)]
pub(crate) struct EntryInfo {
    ename: String,
    eppath: String,
    epath: String,
    etype: String,
    emodified: u64,
    eaccessed: u64,
    ecreated: u64,
}

pub(crate) async fn list_entry_info_handler(
    State(state): State<AppState>,
    entrypath: Option<Path<String>>,
) -> impl IntoResponse {
    let r_entry_path = if let Some(Path(p)) = entrypath {
        PathBuf::from(p)
    } else {
        PathBuf::from("")
    };

    let a_entry_path = match state.config.root_dirpath.join(&r_entry_path).canonicalize() {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    code: 404,
                    message: format!("{} not found", &r_entry_path.display()),
                    data: None,
                }),
            );
        }
    };

    let strip_prefix = format!("{}/", &state.config.root_dirpath.display());

    if a_entry_path.is_file() || a_entry_path.is_symlink() {
        let ename = a_entry_path
            .file_name()
            .and_then(|f| Some(f.to_string_lossy().to_string()))
            .unwrap_or_else(|| "Unknown".to_string());
        let etype = if a_entry_path.is_file() {
            "f".to_string()
        } else if a_entry_path.is_symlink() {
            "l".to_string()
        } else {
            "u".to_string()
        };

        let eppath = a_entry_path
            .parent()
            .and_then(|p| Some(p.to_string_lossy().to_string()))
            .unwrap_or_else(|| "".to_string())
            .strip_prefix(&strip_prefix)
            .map_or_else(|| "".to_string(), |p| p.to_string());

        let epath = a_entry_path
            .to_string_lossy()
            .to_string()
            .strip_prefix(&strip_prefix)
            .map_or_else(|| "".to_string(), |p| p.to_string());
        let emodified = a_entry_path
            .metadata()
            .and_then(|m| {
                m.modified().map(|m| {
                    m.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or(std::time::Duration::from_secs(0))
                        .as_secs()
                })
            })
            .unwrap_or_else(|_| 0);
        let eaccessed = a_entry_path
            .metadata()
            .and_then(|m| {
                m.accessed().map(|m| {
                    m.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or(std::time::Duration::from_secs(0))
                        .as_secs()
                })
            })
            .unwrap_or_else(|_| 0);
        let ecreated = a_entry_path
            .metadata()
            .and_then(|m| {
                m.created().map(|m| {
                    m.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or(std::time::Duration::from_secs(0))
                        .as_secs()
                })
            })
            .unwrap_or_else(|_| 0);
        return (
            StatusCode::OK,
            Json(ApiResponse {
                code: 200,
                message: "OK".to_string(),
                data: Some(json!(vec![EntryInfo {
                    ename,
                    eppath,
                    epath,
                    etype,
                    emodified,
                    eaccessed,
                    ecreated,
                }])),
            }),
        );
    } else if a_entry_path.is_dir() {
        let mut entries_info = vec![];
        match std::fs::read_dir(&a_entry_path) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let ename = entry.file_name().to_string_lossy().to_string();
                        let etype = entry
                            .file_type()
                            .and_then(|ft| {
                                if ft.is_file() {
                                    Ok("f")
                                } else if ft.is_dir() {
                                    Ok("d")
                                } else {
                                    Ok("u")
                                }
                            })
                            .map_or_else(|_| "u".to_string(), |s| s.to_string());
                        let eppath = entry
                            .path()
                            .parent()
                            .and_then(|p| Some(p.to_string_lossy().to_string()))
                            .unwrap_or_else(|| "".to_string())
                            .strip_prefix(&strip_prefix)
                            .map_or_else(|| "".to_string(), |p| p.to_string());

                        let epath = entry
                            .path()
                            .to_string_lossy()
                            .to_string()
                            .strip_prefix(&strip_prefix)
                            .map_or_else(|| "".to_string(), |p| p.to_string());

                        let emodified = entry
                            .metadata()
                            .and_then(|m| {
                                m.modified().map(|m| {
                                    m.duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or(std::time::Duration::from_secs(0))
                                        .as_secs()
                                })
                            })
                            .unwrap_or_else(|_| 0);
                        let eaccessed = entry
                            .metadata()
                            .and_then(|m| {
                                m.accessed().map(|m| {
                                    m.duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or(std::time::Duration::from_secs(0))
                                        .as_secs()
                                })
                            })
                            .unwrap_or_else(|_| 0);

                        let ecreated = entry
                            .metadata()
                            .and_then(|m| {
                                m.created().map(|m| {
                                    m.duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or(std::time::Duration::from_secs(0))
                                        .as_secs()
                                })
                            })
                            .unwrap_or_else(|_| 0);

                        entries_info.push(EntryInfo {
                            ename,
                            eppath,
                            epath,
                            etype,
                            emodified,
                            eaccessed,
                            ecreated,
                        })
                    }
                }
                return (
                    StatusCode::OK,
                    Json(ApiResponse {
                        code: 200,
                        message: "OK".to_string(),
                        data: Some(json!(entries_info)),
                    }),
                );
            }
            Err(_) => {}
        }
    }

    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse {
            code: 404,
            message: format!("{} not found", &r_entry_path.display()),
            data: None,
        }),
    )
}

pub(crate) async fn delete_entry_handler(
    Path(epath): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let a_entry_path = match state.config.root_dirpath.join(&epath).canonicalize() {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    code: 404,
                    message: format!("{} not found", &epath),
                    data: None,
                }),
            );
        }
    };
    if a_entry_path.is_file() || a_entry_path.is_symlink() {
        if std::fs::remove_file(&a_entry_path).is_err() {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    code: 404,
                    message: format!("{} not found", &epath),
                    data: None,
                }),
            );
        }
    } else if a_entry_path.is_dir() {
        if std::fs::remove_dir_all(&a_entry_path).is_err() {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    code: 404,
                    message: format!("{} not found", &epath),
                    data: None,
                }),
            );
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 200,
            message: format!("succcess remove {}", &epath),
            data: None,
        }),
    )
}

#[derive(Deserialize)]
pub(crate) struct RenameEntryBody {
    newname: String,
}
pub(crate) async fn rename_entry_handler(
    Path(epath): Path<String>,
    State(state): State<AppState>,
    Json(body): Json<RenameEntryBody>,
) -> impl IntoResponse {
    let o_a_entry_path = match state.config.root_dirpath.join(&epath).canonicalize() {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    code: 404,
                    message: format!("{} not found", &epath),
                    data: None,
                }),
            );
        }
    };
    let o_a_entry_ppath = o_a_entry_path.parent().unwrap();

    if std::fs::rename(&o_a_entry_path, o_a_entry_ppath.join(&body.newname)).is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                code: 404,
                message: format!("error {} to {} ", &epath, &body.newname),
                data: None,
            }),
        );
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 200,
            message: format!("success {} to {}", &epath, &body.newname),
            data: None,
        }),
    )
}

pub(crate) async fn create_entry_handler(
    Path(entrypath): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let a_entry_path = state.config.root_dirpath.join(&entrypath);
    if std::fs::create_dir_all(&a_entry_path).is_err() {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                code: 404,
                message: format!("error create {} ", &entrypath),
                data: None,
            }),
        );
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 200,
            message: format!("success create {}", &entrypath),
            data: None,
        }),
    )
}

pub(crate) async fn upload_entry_handler(
    entrypath: Option<Path<String>>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let r_entry_path = if let Some(Path(p)) = entrypath {
        PathBuf::from(p)
    } else {
        PathBuf::from("")
    };

    let a_entry_path = match state.config.root_dirpath.join(&r_entry_path).canonicalize() {
        Ok(p) => p,
        Err(_) => {
            tracing::warn!(">>> {} not found", &r_entry_path.display());
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    code: 404,
                    message: format!("{} not found", &r_entry_path.display()),
                    data: None,
                }),
            );
        }
    };

    let mut total_bytes = 0;

    while let Ok(Some(mut field)) = multipart.next_field().await {
        if let Some(file_name) = field.file_name() {
            let file_name = PathBuf::from(&file_name)
                .file_name()
                .map_or_else(|| "unknow".to_string(), |m| m.to_string_lossy().to_string());
            let save_path = a_entry_path.join(&file_name);
            tracing::info!(">>> start save {} to {:?}", &file_name, &save_path);

            match tokio::fs::File::create(&save_path).await {
                Ok(file) => {
                    let mut stream_writer = tokio::io::BufWriter::new(file);
                    let mut total_chunk_bytes = 0;
                    while let Ok(Some(chunk)) = field.chunk().await {
                        total_chunk_bytes += chunk.len();
                        // 关键优化 3: 流式读取 (Chunked)
                        // 只要网络还在传数据，这个循环就会继续。内存中永远只保留当前的一个 chunk。
                        let _ = stream_writer.write_all(&chunk).await;

                        // 必须 flush 确保缓冲区的数据全部落盘
                        let _ = stream_writer.flush().await;
                    }
                    total_bytes += total_chunk_bytes;
                    tracing::info!(
                        "success save file: {:?}, size: {}",
                        &save_path,
                        format_bytes(total_chunk_bytes as u64)
                    );
                }
                Err(e) => {
                    tracing::error!(">>> create {:?} error: {}", &save_path, e);
                    continue;
                }
            }
        } else {
            tracing::warn!(">>> no name or file_name and skip");
        }
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            code: 200,
            message: format!(
                "success upload to {} and total {}",
                &r_entry_path.display(),
                format_bytes(total_bytes as u64)
            ),
            data: None,
        }),
    )
}

pub(crate) async fn download_entry_handler(
    Path(entrypath): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response,(StatusCode, Json<ApiResponse>)> {
    let a_entry_path = match state.config.root_dirpath.join(&entrypath).canonicalize() {
        Ok(p) => p,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiResponse {
                    code: 404,
                    message: format!("{} not found", &entrypath),
                    data: None,
                })),
            );
        }
    };

    if a_entry_path.is_file() {
        let mut file = match tokio::fs::File::open(&a_entry_path).await {
            Ok(f) => f,
            Err(e) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse {
                        code: 404,
                        message: format!("{},err:{}", &entrypath, e),
                        data: None,
                    }),
                ));
            }
        };

        let emeta = match a_entry_path.metadata() {
            Ok(m) => m,
            Err(_) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        code: 500,
                        message: format!("{} not get meta", &entrypath),
                        data: None,
                    }),
                ));
            }
        };
        let ename = a_entry_path
            .file_name()
            .map_or_else(|| "unknow".to_string(), |m| m.to_string_lossy().to_string());
        let esize = emeta.len();
        let mime_type = mime_guess::from_path(&a_entry_path).first_or_octet_stream();
        // 解析 Range
        let mut start = 0;
        let mut end = esize - 1;
        let mut is_partial = false;

        if let Some(range_header) = headers.get(header::RANGE) {
            if let Ok(range_str) = range_header.to_str() {
                if let Some((s, e)) = parse_range(range_str, esize) {
                    start = s;
                    end = e;
                    is_partial = true;
                } else {
                    return Err((
                        StatusCode::RANGE_NOT_SATISFIABLE,
                        Json(ApiResponse {
                            code: 416,
                            message: format!("bytes */{}", esize),
                            data: None,
                        }),
                    ));
                }
            }
        }
        let content_length = end - start + 1;
        // 6. 核心逻辑：Seek + Take (利用标准库的高性能实现)
        if start > 0 {
            if file.seek(SeekFrom::Start(start)).await.is_err() {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse {
                        code: 500,
                        message: format!("{} seek error", &entrypath),
                        data: None,
                    }),
                ));
            }
        }
        // 关键点：file.take(len) 会限制读取长度，并且所有权被移交给 ReaderStream
        let stream = ReaderStream::new(file.take(content_length));
        let body = Body::from_stream(stream);
        // 构建响应头
        let mut response_headers = HeaderMap::new();
        response_headers.insert(header::CONTENT_TYPE, mime_type.as_ref().parse().unwrap());
        response_headers.insert(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", ename)
                .parse()
                .unwrap(),
        );
        response_headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
        response_headers.insert(
            header::CONTENT_LENGTH,
            content_length.to_string().parse().unwrap(),
        );

        if is_partial {
            response_headers.insert(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, esize).parse().unwrap());
            Ok((StatusCode::PARTIAL_CONTENT, response_headers, body).into_response())
        } else {
            Ok((StatusCode::OK, response_headers, body).into_response())
        }
    } else {
        todo!()
    }

}// 解析 Range 的辅助函数 (保持简单有效)
fn parse_range(range: &str, size: u64) -> Option<(u64, u64)> {
    let range = range.strip_prefix("bytes=")?;
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 { return None; }

    let start = parts[0].parse::<u64>().ok()?;
    let end = if parts[1].is_empty() {
        size - 1
    } else {
        parts[1].parse::<u64>().ok()?.min(size - 1)
    };

    if start > end { return None; }
    Some((start, end))
}


