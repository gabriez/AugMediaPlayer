use {
    gstgtk4::PaintableSink,
    gstreamer::{prelude::*, *},
    gtk::glib,
    std::{
        cell::{RefCell, RefMut},
        rc::Rc,
    },
    thiserror::Error,
};
pub struct MediaPlayer {
    /// Our one and only element
    playbin: Element,
    /// Are we in the PLAYING state?
    playing: bool,

    /// Is seeking enabled for this media?
    seek_enabled: bool,
    /// How long does this media last, in nanoseconds
    duration: Option<ClockTime>,
    /// Video Widget
    gtk_video: PaintableSink,

    user_is_seeking: bool,
    /// Current volume (0.0 to 1.0)
    volume: f64,
    /// Is the player muted?
    muted: bool,
    /// Volume before muting (to restore when unmuting)
    volume_before_mute: f64,
}

impl MediaPlayer {
    pub fn build(uri: impl AsRef<str>) -> Self {
        let playbin = ElementFactory::make("playbin")
            .name("playbin")
            // Set the URI to play
            .property("uri", uri.as_ref())
            .build()
            .expect("Failed to create playbin element");

        let videosink = PaintableSink::new(Some("gtk4paintablesink"));

        playbin.set_property("video-sink", &videosink);
        
        // Set default volume to 50%
        playbin.set_property("volume", 0.5f64);

        Self {
            playbin,
            playing: false,
            seek_enabled: false,
            duration: ClockTime::NONE,
            gtk_video: videosink,
            user_is_seeking: false,
            volume: 0.5,
            muted: false,
            volume_before_mute: 0.5,
        }
    }
    // Getters

    pub fn user_is_seeking(&self) -> bool {
        self.user_is_seeking
    }

    pub fn duration(&self) -> Option<ClockTime> {
        self.duration
    }

    pub fn seek_enabled(&self) -> bool {
        self.seek_enabled
    }

    pub fn get_gtk_widget(&self) -> gtk::Widget {
        let paintable = self.gtk_video.property::<gtk::gdk::Paintable>("paintable");

        gtk::Picture::for_paintable(&paintable).upcast()
    }

    pub fn get_bus(&self) -> Bus {
        // This function most of the times will return a bus
        // If by any reason the bus is None, then something is wrong with gstreamer setup
        self.playbin.bus().unwrap()
    }

    pub fn playing(&self) -> bool {
        self.playing
    }

    pub fn get_position(&self) -> Result<ClockTime, MediaPlayerErrors> {
        self.playbin.query_position::<ClockTime>().map_or_else(
            || Err(MediaPlayerErrors::ErrorGettingPosition),
            |val| Ok(val),
        )
    }

    // Setters
    pub fn set_user_is_seeking(&mut self, user_is_seeking: bool) {
        self.user_is_seeking = user_is_seeking;
    }

    pub fn pause_player(&self) -> Result<(), MediaPlayerErrors> {
        self.playbin
            .set_state(State::Paused)
            .map_err(MediaPlayerErrors::ErrorPausing)
            .map(|_| ())
    }

    pub fn play_player(&self) -> Result<(), MediaPlayerErrors> {
        self.playbin
            .set_state(State::Playing)
            .map_err(MediaPlayerErrors::ErrorPlaying)
            .map(|_| ())
    }

    pub fn stop_player(&self) -> Result<(), MediaPlayerErrors> {
        self.playbin
            .set_state(State::Ready)
            .map_err(MediaPlayerErrors::Errorstopping)
            .map(|_| ())
    }

    pub fn move_forward(&self) -> Result<(), MediaPlayerErrors> {
        if !self.seek_enabled {
            return Ok(());
        }
        let position = self.get_position()?;
        self.playbin
            .seek_simple(
                SeekFlags::FLUSH | SeekFlags::KEY_UNIT,
                position + (10 * ClockTime::SECOND),
            )
            .map_err(|err| MediaPlayerErrors::ErrorSeekingForward(err))
    }

    pub fn seek_position(&self, position: ClockTime) -> Result<(), MediaPlayerErrors> {
        if !self.seek_enabled {
            return Ok(());
        }
        self.playbin
            .seek_simple(SeekFlags::FLUSH | SeekFlags::KEY_UNIT, position)
            .map_err(|err| MediaPlayerErrors::ErrorSeeking(err))
    }

    pub fn move_backward(&self) -> Result<(), MediaPlayerErrors> {
        if !self.seek_enabled {
            return Ok(());
        }
        let position = self.get_position()?;
        let new_position = if position > (10 * ClockTime::SECOND) {
            position - (10 * ClockTime::SECOND)
        } else {
            ClockTime::ZERO
        };

        self.playbin
            .seek_simple(SeekFlags::FLUSH | SeekFlags::KEY_UNIT, new_position)
            .map_err(|err| MediaPlayerErrors::ErrorSeekingBackward(err))
    }

    /// Set the volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f64) -> Result<(), MediaPlayerErrors> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.playbin.set_property("volume", clamped_volume);
        self.volume = clamped_volume;
        if !self.muted {
            self.volume_before_mute = clamped_volume;
        }
        Ok(())
    }

    /// Get the current volume (0.0 to 1.0)
    pub fn get_volume(&self) -> f64 {
        self.volume
    }

    /// Toggle mute/unmute
    pub fn toggle_mute(&mut self) -> Result<(), MediaPlayerErrors> {
        if self.muted {
            // Unmute: restore previous volume
            self.playbin.set_property("volume", self.volume_before_mute);
            self.volume = self.volume_before_mute;
            self.muted = false;
        } else {
            // Mute: save current volume and set to 0
            self.volume_before_mute = self.volume;
            self.playbin.set_property("volume", 0.0f64);
            self.volume = 0.0;
            self.muted = true;
        }
        Ok(())
    }

    /// Check if the player is muted
    pub fn is_muted(&self) -> bool {
        self.muted
    }
}

impl Drop for MediaPlayer {
    fn drop(&mut self) {
        self.playbin
            .set_state(State::Null)
            .expect("Unable to set the playbin to the `Null` state");
    }
}

pub type MediaPlayerRef = Rc<RefCell<MediaPlayer>>;

#[derive(Error, Debug)]
pub enum MediaPlayerErrors {
    #[error("Unable to seek forward with this media. Check if you're using a stream")]
    ErrorSeekingForward(glib::error::BoolError),
    #[error("Unable to seek backward with this media. Check if you're using a stream")]
    ErrorSeekingBackward(glib::error::BoolError),
    #[error(
        "Unable to seek to the specified position with this media. Check if you're using a stream"
    )]
    ErrorSeeking(glib::error::BoolError),

    #[error("Unable to get position in this media. Check if you're using a stream")]
    ErrorGettingPosition,

    #[error("Error playing media")]
    ErrorPlaying(StateChangeError),
    #[error("Error stopping media")]
    Errorstopping(StateChangeError),
    #[error("Error pausing media")]
    ErrorPausing(StateChangeError),
}

pub fn handle_message(mut media_player: RefMut<'_, MediaPlayer>, msg: &Message) {
    use MessageView;
    match msg.view() {
        MessageView::Error(err) => {
            println!(
                "Error received from element {:?}: {} ({:?})",
                err.src().map(|s| s.path_string()),
                err.error(),
                err.debug()
            );
        }
        MessageView::DurationChanged(_) => {
            // The duration has changed, mark the current one as invalid
            media_player.duration = ClockTime::NONE;
        }
        MessageView::StateChanged(state_changed) => {
            if state_changed
                .src()
                .map(|s| s == &media_player.playbin)
                .unwrap_or(false)
            {
                let new_state = state_changed.current();

                media_player.playing = new_state == State::Playing;
                if media_player.playing {
                    if media_player.duration == gstreamer::ClockTime::NONE {
                        media_player.duration = media_player.playbin.query_duration();
                    }

                    let mut seeking = query::Seeking::new(Format::Time);
                    if media_player.playbin.query(&mut seeking) {
                        let (seekable, _, _) = seeking.result();
                        media_player.seek_enabled = seekable;
                    }
                }
            }
        }
        _ => (),
    }
}
