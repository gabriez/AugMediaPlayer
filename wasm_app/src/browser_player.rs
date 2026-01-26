#[wasm_bindgen]

pub struct BrowserMediaPlayer {
    /// Source URL of the media
    src: String,
    /// Is video playing right now?
    playing: bool,
    /// Is seeking enabled for this media?
    seek_enabled: bool,
    /// How long does this media last, in nanoseconds
    duration: Option<f64>,

    /// Is user seeking the media?
    user_is_seeking: bool,
    /// Current volume (0.0 to 1.0)
    volume: f64,
    /// Is the player muted?
    muted: bool,
    /// Volume before muting (to restore when unmuting)
    volume_before_mute: f64,
}

#[wasm_bindgen]

impl BrowserMediaPlayer {
    pub fn build(src: impl AsRef<str>) -> Self {
        Self {
            src: src.as_ref().to_string(),
            playing: false,
            seek_enabled: false,
            duration: None,
            user_is_seeking: false,
            volume: 0.5,
            muted: false,
            volume_before_mute: 0.5,
        }
    }
}
