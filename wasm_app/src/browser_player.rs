use {
    common_media::media_player::{MediaPlayerErrors, PlaybackControl, SeekControl, VolumeControl},
    wasm_bindgen::prelude::*,
};

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
    pub fn build(src: String) -> Self {
        Self {
            src,
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

// /// TODO: still to be implemented
impl PlaybackControl for BrowserMediaPlayer {
    fn play(&self) -> Result<(), MediaPlayerErrors> {
        todo!("Still to implement")
    }

    fn pause(&self) -> Result<(), MediaPlayerErrors> {
        todo!("Still to implement");
    }

    fn playing(&self) -> bool {
        self.playing
    }
    fn stop(&self) -> Result<(), MediaPlayerErrors> {
        todo!("Still to implement")
    }
}

impl SeekControl for BrowserMediaPlayer {
    fn can_seek(&self) -> bool {
        self.seek_enabled
    }
    fn duration(&self) -> Option<f64> {
        todo!("Still to implement")
    }
    fn position(&self) -> Result<f64, MediaPlayerErrors> {
        todo!("Still to implement")
    }
    fn seek_backward(&self) -> Result<(), MediaPlayerErrors> {
        todo!("Still to implement")
    }
    fn seek_forward(&self) -> Result<(), MediaPlayerErrors> {
        todo!("Still to implement")
    }
    fn seek_to(&self, _position: f64) -> Result<(), MediaPlayerErrors> {
        todo!("Still to implement")
    }
    fn set_user_is_seeking(&mut self, _user_is_seeking: bool) {
        todo!("Still to implement")
    }
    fn user_is_seeking(&self) -> bool {
        todo!("Still to implement")
    }
}

impl VolumeControl for BrowserMediaPlayer {
    fn get_volume(&self) -> f64 {
        todo!("Still to implement")
    }
    fn is_muted(&self) -> bool {
        todo!("Still to implement")
    }
    fn set_volume(&mut self, _volume: f64) {
        todo!("Still to implement")
    }
    fn toggle_mute(&mut self) {
        todo!("Still to implement")
    }
}

// // Maybe I'll delete this trait in the future. I created it this way to group all the other traits, but maybe is unnecesary
// impl MediaPlayerControl for BrowserMediaPlayer {}
