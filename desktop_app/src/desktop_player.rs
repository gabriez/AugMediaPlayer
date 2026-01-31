use {
    common_media::media_player::{
        MediaPlayerControl, MediaPlayerErrors, PlaybackControl, SeekControl, VolumeControl,
    },
    gstgtk4::PaintableSink,
    gstreamer::{ClockTime, prelude::*, *},
    std::{
        cell::{RefCell, RefMut},
        rc::Rc,
    },
};

#[derive(Debug, Clone)]
pub struct DesktopMediaPlayer {
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

    /// Is user seeking the media?
    user_is_seeking: bool,
    /// Current volume (0.0 to 1.0)
    volume: f64,
    /// Is the player muted?
    muted: bool,
    /// Volume before muting (to restore when unmuting)
    volume_before_mute: f64,
}

impl DesktopMediaPlayer {
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

    pub fn get_gtk_widget(&self) -> gtk::Widget {
        let paintable = self.gtk_video.property::<gtk::gdk::Paintable>("paintable");

        gtk::Picture::for_paintable(&paintable).upcast()
    }

    /// Will return gstreamer bus used to get notifications
    pub fn get_bus(&self) -> Bus {
        // This function most of the times will return a bus
        // If by any reason the bus is None, then something is wrong with gstreamer setup
        self.playbin.bus().unwrap()
    }
}

impl Drop for DesktopMediaPlayer {
    fn drop(&mut self) {
        self.playbin
            .set_state(State::Null)
            .expect("Unable to set the playbin to the `Null` state");
    }
}

pub fn handle_message(mut media_player: RefMut<'_, DesktopMediaPlayer>, msg: &Message) {
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

impl MediaPlayerControl for DesktopMediaPlayer {
    // placeholder
}

impl PlaybackControl for DesktopMediaPlayer {
    fn pause(&self) -> Result<(), MediaPlayerErrors> {
        self.playbin
            .set_state(State::Paused)
            .map_err(|err| MediaPlayerErrors::ErrorPausing(err.to_string()))
            .map(|_| ())
    }

    fn play(&self) -> Result<(), MediaPlayerErrors> {
        self.playbin
            .set_state(State::Playing)
            .map_err(|err| MediaPlayerErrors::ErrorPlaying(err.to_string()))
            .map(|_| ())
    }

    fn stop(&self) -> Result<(), MediaPlayerErrors> {
        self.playbin
            .set_state(State::Ready)
            .map_err(|err| MediaPlayerErrors::Errorstopping(err.to_string()))
            .map(|_| ())
    }

    fn playing(&self) -> bool {
        self.playing
    }
}

impl SeekControl for DesktopMediaPlayer {
    fn user_is_seeking(&self) -> bool {
        self.user_is_seeking
    }

    fn set_user_is_seeking(&mut self, user_is_seeking: bool) {
        self.user_is_seeking = user_is_seeking;
    }

    fn duration(&self) -> Option<f64> {
        self.duration.map(|d| d.seconds_f64())
    }

    fn seek_forward(&self) -> Result<(), MediaPlayerErrors> {
        let position = self.position()?;
        self.seek_to(position + (10_f64 * ClockTime::SECOND.seconds_f64()))
    }

    fn seek_to(&self, position: f64) -> Result<(), MediaPlayerErrors> {
        if !self.seek_enabled {
            return Err(MediaPlayerErrors::ErrorSeekingUnavailable);
        }
        self.playbin
            .seek_simple(
                SeekFlags::FLUSH | SeekFlags::KEY_UNIT,
                ClockTime::from_seconds_f64(position),
            )
            .map_err(|err| MediaPlayerErrors::ErrorSeeking(err.to_string()))
    }

    fn seek_backward(&self) -> Result<(), MediaPlayerErrors> {
        let position = self.position()?;
        let new_position = if position > (10_f64 * ClockTime::SECOND.seconds_f64()) {
            position - (10_f64 * ClockTime::SECOND.seconds_f64())
        } else {
            0_f64
        };
        self.seek_to(new_position)
    }

    fn can_seek(&self) -> bool {
        self.seek_enabled
    }

    fn position(&self) -> Result<f64, MediaPlayerErrors> {
        self.playbin.query_position::<ClockTime>().map_or_else(
            || Err(MediaPlayerErrors::ErrorGettingPosition),
            |val| Ok(val.seconds_f64()),
        )
    }
}

impl VolumeControl for DesktopMediaPlayer {
    fn set_volume(&mut self, volume: f64) {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.playbin.set_property("volume", clamped_volume);
        self.volume = clamped_volume;
        if !self.muted {
            self.volume_before_mute = clamped_volume;
        }
    }

    fn get_volume(&self) -> f64 {
        self.volume
    }

    fn toggle_mute(&mut self) {
        if self.muted {
            self.playbin.set_property("volume", self.volume_before_mute);
            self.volume = self.volume_before_mute;
            self.muted = false;
        } else {
            self.volume_before_mute = self.volume;
            self.playbin.set_property("volume", 0.0f64);
            self.volume = 0.0;
            self.muted = true;
        }
    }

    fn is_muted(&self) -> bool {
        self.muted
    }
}

pub type DesktopMediaPlayerRef = Rc<RefCell<DesktopMediaPlayer>>;
