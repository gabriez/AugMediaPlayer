use std::sync::OnceLock;

use serde::Serialize;

pub mod media;

pub static ENV_VARS: OnceLock<ServerEnvVars> = OnceLock::new();

#[derive(Serialize)]
pub struct ServerResponse<T> {
    pub status: bool,
    pub message: String,
    pub data: T,
}

impl<T> ServerResponse<T> {
    pub fn new(status: bool, message: String, data: T) -> Self {
        Self {
            status,
            message,
            data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServerEnvVars {
    pub media_storage_path: String,
    pub port: u16,
}

impl ServerEnvVars {
    /// This method will only work if dotenv.ok() has been called at the start of the program
    /// The .env variables will have a default value
    pub fn build() -> Self {
        let media_storage_path =
            std::env::var("MEDIA_STORAGE_PATH").unwrap_or_else(|_| "./media_files".to_string());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .unwrap_or(3000);

        Self {
            media_storage_path,
            port,
        }
    }
}

pub fn create_dir_if_not_exists(path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    if !path.as_ref().exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
