use {
    aug_media_player::{
        config::Args,
        media_player::{DesktopMediaPlayer, PlaybackControl, handle_message},
        ui::build_ui,
    },
    clap::Parser,
    gtk::{Application, glib, prelude::*},
    std::{cell::RefCell, rc::Rc},
};

// I decided that I wasn't gonna use poliformism here because there's only one type of media player for desktop
// Also, clone! macro from gtk-rs doesn't work well with trait objects due to the way it handles weak references
// They are no sized, and you can say you can add a box so it is sized, but then the UI starts behaving weirdly because of the indirections
// Maybe there's a leak, maybe there isn't. But using Box isn't ideal anyway

const APP_ID: &str = "org.AugMediaPlayer";

fn main() -> glib::ExitCode {
    let args = Args::parse();
    let uri = args.formatted_uri();

    gstreamer::init().expect("Unable to initialize GStreamer");

    let app = Application::builder().application_id(APP_ID).build();

    let inner_player = DesktopMediaPlayer::build(uri);

    let bus = inner_player.get_bus();

    let media_player_ref = Rc::new(RefCell::new(inner_player));
    let media_player_ref_cloned = media_player_ref.clone();

    // After dropping the guard, the bus watch is removed automatically
    // If we avoid capturing the return value, then it will be dropped an the messages handler too
    let _bus_watch = bus
        .add_watch_local(move |_, msg| {
            handle_message(media_player_ref_cloned.borrow_mut(), msg);
            glib::ControlFlow::Continue
        })
        .expect("Failed to add bus watch");

    app.connect_activate(move |application| {
        let video_widget = media_player_ref.borrow().get_gtk_widget();

        build_ui(application, media_player_ref.clone(), video_widget.clone());
        media_player_ref.borrow().play().ok();
    });

    app.run_with_args::<String>(&[])
}
