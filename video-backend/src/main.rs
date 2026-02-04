use axum::{Json, Router, response::IntoResponse, routing::get};
use dotenv::dotenv;
use video_backend::{ENV_VARS, ServerEnvVars, ServerResponse, media::media_routes};

#[tokio::main]
async fn main() {
    dotenv().ok();
    gstreamer::init().expect("Unable to initialize GStreamer");

    let env_vars: ServerEnvVars = ServerEnvVars::build();
    ENV_VARS
        .set(env_vars.clone())
        .expect("Failed to set ENV_VARS");

    let app = Router::new()
        .route("/", get(default_response))
        .merge(media_routes());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", env_vars.port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub async fn default_response() -> impl IntoResponse {
    Json(ServerResponse::new(
        true,
        "Video Backend is running".to_string(),
        (),
    ))
}
