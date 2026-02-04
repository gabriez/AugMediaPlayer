use std::path::PathBuf;

use axum::{
    Json,
    extract::{Multipart, Path},
    http::StatusCode,
    response::IntoResponse,
};
use gstreamer::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{ENV_VARS, ServerResponse, create_dir_if_not_exists};

// Storage path local route
// env_var/media_files.json
// env_var/media_files/{id}/file.mp4
// env_var/media_files/{id}/metadata.json

const MEDIA_PATH: &'static str = "media_files";
const MEDIA_PATH_JSON: &'static str = "media_files.json";
const METADATA_JSON: &'static str = "metadata.json";

#[derive(Serialize, Deserialize, Clone)]
pub struct MediaFiles {
    pub id: String,
    pub filename: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MediaFileMetadata {
    // Second in which we'll be extracting the frame
    pub timestamp: f64,
    // Position to put the frame in the video in X axis
    pub x: u32,
    // Position to put the frame in the video in Y axis
    pub y: u32,
    // Width that the frame will occupy
    pub width: u32,
    // Height that the frame will occupy
    pub height: u32,
}

pub async fn get_media_files() -> impl IntoResponse {
    let env_vars = ENV_VARS.get().expect("ENV_VARS not set");
    let media_storage_path =
        std::path::Path::new(env_vars.media_storage_path.as_str()).join(MEDIA_PATH_JSON);

    tokio::fs::read_to_string(media_storage_path)
        .await
        .map(|data| {
            let media_files: Vec<MediaFiles> = serde_json::from_str(&data).unwrap_or_default();
            (
                StatusCode::OK,
                Json(ServerResponse {
                    status: true,
                    message: "Media files retrieved successfully".to_string(),
                    data: media_files,
                }),
            )
        })
        .unwrap_or_else(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ServerResponse {
                    status: false,
                    message: "Failed to read media files".to_string(),
                    data: vec![],
                }),
            )
        })
}

pub async fn stream_media_file(Path(id): Path<u32>) -> impl IntoResponse {
    StatusCode::INTERNAL_SERVER_ERROR
}

pub async fn upload_media_file(multipart: Multipart) -> impl IntoResponse {
    let env_vars = ENV_VARS.get().expect("ENV_VARS not set");
    let root_dir = std::path::Path::new(env_vars.media_storage_path.as_str()).to_path_buf();

    let (media_file, video_path) = match store_video_file(multipart, &root_dir).await {
        Ok(path) => path,
        Err(err_response) => return err_response,
    };

    // This shouldn't panic at this point
    let absolute_path = video_path.canonicalize().unwrap();

    let metadata = match process_video_with_gstreamer(&absolute_path) {
        Ok(metadata) => metadata,
        Err(err_msg) => {
            println!("Error procesing video: {}", err_msg);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ServerResponse::<()> {
                    status: false,
                    message: "Failed to process video".to_string(),
                    data: (),
                }),
            );
        }
    };

    if let Err(err_msg) =
        store_metadata(&video_path.parent().unwrap().to_path_buf(), metadata).await
    {
        println!("Error storing metadata: {}", err_msg);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ServerResponse::<()> {
                status: false,
                message: "Failed to process video".to_string(),
                data: (),
            }),
        );
    }

    if let Err(err_msg) = store_mediafile_data(&root_dir, media_file).await {
        println!("Error storing media file data: {}", err_msg);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ServerResponse::<()> {
                status: false,
                message: "Failed to process video".to_string(),
                data: (),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(ServerResponse::<()> {
            status: true,
            message: "Success".to_string(),
            data: (),
        }),
    )
}

pub async fn get_media_file_metadata(Path(id): Path<u32>) -> impl IntoResponse {
    let env_vars = ENV_VARS.get().expect("ENV_VARS not set");
    let media_storage_path = std::path::Path::new(env_vars.media_storage_path.as_str())
        .join(MEDIA_PATH)
        .join(id.to_string())
        .join(METADATA_JSON);

    tokio::fs::read_to_string(media_storage_path)
        .await
        .map(|data| {
            let metadata: Vec<MediaFileMetadata> = serde_json::from_str(&data).unwrap_or_default();
            (
                StatusCode::OK,
                Json(ServerResponse {
                    status: true,
                    message: "Meta data file retrieved successfully".to_string(),
                    data: metadata,
                }),
            )
        })
        .unwrap_or_else(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ServerResponse {
                    status: false,
                    message: "Failed to read metadata file".to_string(),
                    data: vec![],
                }),
            )
        })
}

pub fn media_routes() -> axum::Router {
    axum::Router::new()
        .route("/media_files", axum::routing::get(get_media_files))
        .route(
            "/media_files/{id}/stream",
            axum::routing::get(stream_media_file),
        )
        .route(
            "/media_files/upload",
            axum::routing::post(upload_media_file),
        )
        .route(
            "/media_files/{id}/metadata",
            axum::routing::get(get_media_file_metadata),
        )
}

async fn store_video_file(
    mut multipart: Multipart,
    root_dir: &PathBuf,
) -> Result<(MediaFiles, PathBuf), (StatusCode, Json<ServerResponse<()>>)> {
    while let Ok(Some(mut file)) = multipart.next_field().await {
        let mut data = Vec::new();

        let file_name = {
            let file_name = file.file_name();
            let field_name = file.name();

            if file_name.is_none() || field_name.is_none() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ServerResponse::<()> {
                        status: false,
                        message: "File name or field name is missing".to_string(),
                        data: (),
                    }),
                ));
            };

            if field_name.unwrap() != "media_file" {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ServerResponse::<()> {
                        status: false,
                        message: "File name or field name is missing".to_string(),
                        data: (),
                    }),
                ));
            }
            file_name.unwrap().to_string()
        };

        while let Ok(Some(chunk)) = file.chunk().await {
            data.extend_from_slice(&chunk);
        }

        let media_file = MediaFiles {
            id: uuid::Uuid::new_v4().to_string(),
            filename: file_name,
        };

        let media_storage_path = root_dir
            .join(MEDIA_PATH)
            .join(media_file.id.clone())
            .join(media_file.filename.clone());

        if let Err(err) = create_dir_if_not_exists(media_storage_path.parent().unwrap()) {
            println!("Error creating directories: {}", err);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ServerResponse::<()> {
                    status: false,
                    message: "Failed to store video".to_string(),
                    data: (),
                }),
            ));
        }

        let mut file = match File::create(&media_storage_path).await {
            Err(err) => {
                println!("Error creating file: {}", err);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ServerResponse::<()> {
                        status: false,
                        message: "Failed to store video".to_string(),
                        data: (),
                    }),
                ));
            }
            Ok(file) => file,
        };

        if let Err(_) = file.write(&data).await {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ServerResponse::<()> {
                    status: false,
                    message: "Failed to store video".to_string(),
                    data: (),
                }),
            ));
        };
        return Ok((media_file, media_storage_path));
    }
    Err((
        StatusCode::BAD_REQUEST,
        Json(ServerResponse::<()> {
            status: false,
            message: "No file found in the request".to_string(),
            data: (),
        }),
    ))
}

fn process_video_with_gstreamer(video_path: &PathBuf) -> Result<Vec<MediaFileMetadata>, String> {
    use gstreamer as gst;
    use std::sync::{Arc, Mutex};

    println!("file://{}", video_path.to_str().unwrap());

    // Build the pipeline
    let pipeline = gst::parse::launch(&format!(
        "filesrc location={} ! decodebin ! videoconvert ! appsink name=sink",
        video_path.to_str().unwrap()
    ))
    .map_err(|e| format!("Failed to create pipeline: {}", e))?;

    let pipeline = pipeline
        .dynamic_cast::<gst::Pipeline>()
        .map_err(|_| "Failed to cast to Pipeline")?;

    // Get the appsink element
    let appsink = pipeline
        .by_name("sink")
        .ok_or("Failed to get appsink")?
        .dynamic_cast::<gstreamer_app::AppSink>()
        .map_err(|_| "Failed to cast to AppSink")?;

    // Shared metadata collection
    let metadata_list = Arc::new(Mutex::new(Vec::new()));
    let metadata_list_clone = Arc::clone(&metadata_list);

    let mut last_second = -1.0;

    // Set up callbacks for the appsink
    appsink.set_callbacks(
        gstreamer_app::AppSinkCallbacks::builder()
            .new_sample(move |appsink| {
                let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Error)?;
                let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                let caps = sample.caps().ok_or(gst::FlowError::Error)?;

                // Get timestamp
                let pts = buffer.pts();
                if pts.is_none() {
                    return Ok(gst::FlowSuccess::Ok);
                }

                let timestamp_ns = pts.unwrap().nseconds();
                let timestamp_sec = timestamp_ns as f64 / 1_000_000_000.0;

                // Only collect metadata once per second
                let current_second = timestamp_sec.floor();
                if current_second > last_second {
                    last_second = current_second;

                    // Extract video dimensions from caps
                    // Actually, width and height will come from AI algorithm later, not from here
                    let structure = caps.structure(0).ok_or(gst::FlowError::Error)?;
                    let width = structure.get::<i32>("width").ok().unwrap_or(0) as u32;
                    let height = structure.get::<i32>("height").ok().unwrap_or(0) as u32;

                    let metadata = MediaFileMetadata {
                        timestamp: current_second,
                        width,
                        height,
                        x: 0,
                        y: 0,
                    };

                    let mut list = metadata_list_clone.lock().unwrap();
                    list.push(metadata);
                }

                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    // Start pipeline
    pipeline
        .set_state(gst::State::Playing)
        .map_err(|e| format!("Failed to set pipeline to playing: {}", e))?;

    // Wait for EOS or error
    let bus = pipeline.bus().ok_or("Failed to get bus")?;
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null).ok();
                return Err(format!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                ));
            }
            _ => (),
        }
    }

    // Clean up
    pipeline
        .set_state(gst::State::Null)
        .map_err(|e| format!("Failed to stop pipeline: {}", e))?;

    let metadata = metadata_list.lock().unwrap().clone();

    Ok(metadata)
}

async fn store_metadata(path: &PathBuf, metadata: Vec<MediaFileMetadata>) -> Result<(), String> {
    let metadata_path = path.join(METADATA_JSON);

    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

    tokio::fs::write(metadata_path, metadata_json)
        .await
        .map_err(|e| format!("Failed to write metadata file: {}", e))?;

    Ok(())
}

async fn store_mediafile_data(root_dir: &PathBuf, file_data: MediaFiles) -> Result<(), String> {
    let media_files_json_path = root_dir.join(MEDIA_PATH_JSON);

    let mut media_files: Vec<MediaFiles> = if media_files_json_path.exists() {
        let data = tokio::fs::read_to_string(&media_files_json_path)
            .await
            .map_err(|e| format!("Failed to read media files JSON: {}", e))?;
        serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse media files JSON: {}", e))?
    } else {
        Vec::new()
    };

    media_files.push(file_data);

    let updated_json = serde_json::to_string_pretty(&media_files)
        .map_err(|e| format!("Failed to serialize media files: {}", e))?;

    tokio::fs::write(media_files_json_path, updated_json)
        .await
        .map_err(|e| format!("Failed to write media files JSON: {}", e))?;

    Ok(())
}
