use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
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

#[wasm_bindgen]
pub fn resize_metadata(
    val: JsValue,
    new_width: u32,
    new_height: u32,
) -> Result<JsValue, JsError> {
    let metadata: Vec<MediaFileMetadata> = serde_wasm_bindgen::from_value(val)?;

    let resized: Vec<MediaFileMetadata> = metadata
        .iter()
        .map(|data| MediaFileMetadata {
            timestamp: data.timestamp,
            x: (data.x as f64 * new_width as f64 / data.width as f64) as u32,
            y: (data.y as f64 * new_height as f64 / data.height as f64) as u32,
            width: new_width,
            height: new_height,
        })
        .collect();

    Ok(serde_wasm_bindgen::to_value(&resized)?)
}
