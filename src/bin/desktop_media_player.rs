use {
    aug_media_player::{
        config::Args,
        media_player::{DesktopMediaPlayer, MediaPlayerControl, MediaPlayerRef, handle_message},
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

    let inner_player = DesktopMediaPlayer::build(uri);

    let bus = inner_player.get_bus();
    let video_widget = inner_player.get_gtk_widget();

    let media_player_inner = Rc::new(RefCell::new(inner_player));

    let media_player_trait: MediaPlayerRef = Rc::new(RefCell::new(Box::new(
        media_player_inner.borrow().clone(),
    )
        as Box<dyn MediaPlayerControl>));

    // After dropping the guard, the bus watch is removed automatically
    // If we avoid capturing the return value, then it will be dropped an the messages handler too
    let _bus_watch = bus
        .add_watch_local(move |_, msg| {
            handle_message(media_player_inner.borrow_mut(), msg);
            glib::ControlFlow::Continue
        })
        .expect("Failed to add bus watch");

    app.connect_activate(move |application| {
        build_ui(
            application,
            media_player_trait.clone(),
            video_widget.clone(),
        );
        media_player_trait.borrow().play().ok();
    });

    app.run_with_args::<String>(&[])
}
