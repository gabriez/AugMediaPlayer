use thiserror::Error;

#[derive(Error, Debug)]
pub enum MediaPlayerErrors {
    #[error(
        "Unable to seek to the specified position with this media. Check if you're using a stream"
    )]
    ErrorSeeking(String),
    #[error("Seek is unavailable for this media")]
    ErrorSeekingUnavailable,

    #[error("Unable to get position in this media")]
    ErrorGettingPosition,

    #[error("Error playing media")]
    ErrorPlaying(String),
    #[error("Error stopping media")]
    Errorstopping(String),
    #[error("Error pausing media")]
    ErrorPausing(String),
}

pub trait MediaPlayerControl: PlaybackControl + SeekControl + VolumeControl {
    //placeholder
}

/// Trait for playback control
pub trait PlaybackControl {
    /// Play the media
    fn play(&self) -> Result<(), MediaPlayerErrors>;

    /// Pause the media
    fn pause(&self) -> Result<(), MediaPlayerErrors>;

    /// Stop the media
    fn stop(&self) -> Result<(), MediaPlayerErrors>;

    /// Check if the media is playing
    fn playing(&self) -> bool;
}

/// Trait for seek control
pub trait SeekControl {
    /// Move the media forward by 10 seconds
    fn seek_forward(&self) -> Result<(), MediaPlayerErrors>;

    /// Move the media backward by 10 seconds
    fn seek_backward(&self) -> Result<(), MediaPlayerErrors>;

    /// Seek to a specific position in the media
    fn seek_to(&self, position: f64) -> Result<(), MediaPlayerErrors>;

    /// Returns media duration in seconds
    fn duration(&self) -> Option<f64>;

    /// Check if seeking is supported for this media
    fn can_seek(&self) -> bool;

    /// Get current playback position
    fn position(&self) -> Result<f64, MediaPlayerErrors>;

    /// Returns if the user is seeking the media
    fn user_is_seeking(&self) -> bool;

    /// Set if the user is seeking the media
    fn set_user_is_seeking(&mut self, user_is_seeking: bool);
}

/// Trait for volume control
pub trait VolumeControl {
    /// Set the volume (0.0 to 1.0)
    fn set_volume(&mut self, volume: f64);

    /// Get the current volume (0.0 to 1.0)
    fn get_volume(&self) -> f64;

    /// Toggle mute/unmute
    fn toggle_mute(&mut self);

    /// Check if the player is muted
    fn is_muted(&self) -> bool;
}
