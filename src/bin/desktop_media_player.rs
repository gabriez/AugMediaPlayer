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

const APP_ID: &str = "org.AugMediaPlayer";

fn main() -> glib::ExitCode {
    let args = Args::parse();
    let uri = args.formatted_uri();

    gstreamer::init().expect("Unable to initialize GStreamer");

    let app = Application::builder().application_id(APP_ID).build();
    let media_player = Rc::new(RefCell::new(DesktopMediaPlayer::build(uri)));

    let media_player_clone = media_player.clone();

    let bus = media_player.borrow().get_bus();

    // After dropping the guard, the bus watch is removed automatically
    // If we avoid capturing the return value, then it will be dropped an the messages handler too
    let _bus_watch = bus
        .add_watch_local(move |_, msg| {
            handle_message(media_player_clone.borrow_mut(), msg);
            glib::ControlFlow::Continue
        })
        .expect("Failed to add bus watch");

    app.connect_activate(move |application| {
        build_ui(application, media_player.clone());
        media_player.borrow().play().ok();
    });

    app.run_with_args::<String>(&[])
}
