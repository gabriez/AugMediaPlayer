use {
    crate::media_player::MediaPlayerRef,
    gtk::{
        Application, ApplicationWindow, Button, Dialog, Label, ResponseType, Scale,
        glib::{self, clone},
        prelude::*,
    },
};

pub fn refresh_ui(window: &ApplicationWindow, media_player: &MediaPlayerRef, duration_bar: &Scale) {
    duration_bar.connect_change_value(clone!(
        #[weak]
        window,
        #[weak]
        media_player,
        #[upgrade_or]
        glib::Propagation::Stop,
        move |duration_b, _, new_pos| {
            let mut media_player_ref = media_player.borrow_mut();
            media_player_ref.set_user_is_seeking(true);
            if media_player_ref.pause_player().is_err() {
                error_dialog(&window, "There was a problem seeking the media.");
                media_player_ref.set_user_is_seeking(false);
                return glib::Propagation::Stop;
            }
            if let Some(duration) = media_player_ref.duration() {
                let seek_pos = (new_pos / 100_f64) * duration.seconds_f64();
                if let Err(err) =
                    media_player_ref.seek_position(gstreamer::ClockTime::from_seconds_f64(seek_pos))
                {
                    error_dialog(&window, &format!("{}", err));
                    media_player_ref.set_user_is_seeking(false);
                    return glib::Propagation::Stop;
                }
                duration_b.set_value(new_pos);
            }

            if media_player_ref.play_player().is_err() {
                error_dialog(&window, "There was a problem seeking the media.");
                media_player_ref.set_user_is_seeking(false);
                return glib::Propagation::Stop;
            }

            // Placeholder for future functionality
            media_player_ref.set_user_is_seeking(false);

            glib::Propagation::Stop
        }
    ));

    glib::timeout_add_seconds_local(
        1,
        clone!(
            #[weak]
            media_player,
            #[weak]
            duration_bar,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                let media_player_ref = media_player.borrow();
                if !media_player_ref.seek_enabled() {
                    duration_bar.set_value(100.0);
                    return glib::ControlFlow::Continue;
                }
                if media_player_ref.playing() && !media_player_ref.user_is_seeking() {
                    if let (Ok(position), Some(duration)) =
                        (media_player_ref.get_position(), media_player_ref.duration())
                    {
                        let bar_position =
                            (100_f64 / duration.seconds_f64()) * position.seconds_f64();
                        duration_bar.set_value(bar_position);
                    }
                }
                glib::ControlFlow::Continue
            }
        ),
    );
}

pub fn error_dialog(window: &ApplicationWindow, message: &str) {
    let dialog = Dialog::builder()
        .title("Error alert!")
        .transient_for(window)
        .modal(true)
        .build();

    let label = Label::builder()
        .label(message)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    dialog.content_area().append(&label);
    dialog.add_button("Ok", ResponseType::Ok);
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    dialog.present();
}

pub fn build_buttons(media_player: &MediaPlayerRef, window: &ApplicationWindow) -> gtk::Box {
    let start_button = Button::builder()
        .icon_name("media-playback-start")
        .margin_top(2)
        .margin_bottom(2)
        .margin_start(6)
        .margin_end(6)
        .halign(gtk::Align::Center)
        .build();

    start_button.connect_clicked(clone!(
        #[weak]
        window,
        #[weak]
        media_player,
        move |_| {
            if let Err(err) = media_player.borrow().play_player() {
                println!("Error playing player: {:?}", err);
                error_dialog(&window, &format!("{}", err));
            }
        }
    ));

    let pause_button = Button::builder()
        .icon_name("media-playback-pause")
        .margin_top(2)
        .margin_bottom(2)
        .margin_start(6)
        .margin_end(6)
        .halign(gtk::Align::Center)
        .build();

    pause_button.connect_clicked(clone!(
        #[weak]
        window,
        #[weak]
        media_player,
        move |_| {
            if let Err(err) = media_player.borrow().pause_player() {
                println!("Error pausing player: {:?}", err);
                error_dialog(&window, &format!("{}", err));
            }
        }
    ));

    let stop_button = Button::builder()
        .icon_name("media-playback-stop")
        .margin_top(2)
        .margin_bottom(2)
        .margin_start(6)
        .margin_end(6)
        .halign(gtk::Align::Center)
        .build();

    stop_button.connect_clicked(clone!(
        #[weak]
        window,
        #[weak]
        media_player,
        move |_| {
            if let Err(err) = media_player.borrow().stop_player() {
                println!("Error stopping player: {:?}", err);
                error_dialog(&window, &format!("{}", err));
            }
        }
    ));

    let forward_button = Button::builder()
        .icon_name("media-seek-forward")
        .margin_top(2)
        .margin_bottom(2)
        .margin_start(6)
        .margin_end(6)
        .halign(gtk::Align::Center)
        .build();

    forward_button.connect_clicked(clone!(
        #[weak]
        window,
        #[weak]
        media_player,
        move |_| {
            if let Err(err) = media_player.borrow().move_forward() {
                println!("Error seeking forward: {:?}", err);
                error_dialog(&window, &format!("{}", err));
            }
        }
    ));

    let backward_button = Button::builder()
        .icon_name("media-seek-backward")
        .margin_top(2)
        .margin_bottom(2)
        .margin_start(6)
        .margin_end(6)
        .halign(gtk::Align::Center)
        .build();

    backward_button.connect_clicked(clone!(
        #[weak]
        window,
        #[weak]
        media_player,
        move |_| {
            if let Err(err) = media_player.borrow().move_backward() {
                println!("Error seeking backward: {:?}", err);
                error_dialog(&window, &format!("{}", err));
            }
        }
    ));

    let button_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .halign(gtk::Align::Center)
        .spacing(6)
        .margin_top(2)
        .margin_bottom(2)
        .build();

    button_box.append(&backward_button);
    button_box.append(&pause_button);
    button_box.append(&start_button);
    button_box.append(&stop_button);
    button_box.append(&forward_button);

    button_box
}

pub fn build_ui(app: &Application, media_player: MediaPlayerRef) {
    let window: ApplicationWindow = ApplicationWindow::builder()
        .application(app)
        .title("AugMediaPlayer")
        .build();

    let button_box = build_buttons(&media_player, &window);

    let control_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let duration_bar = gtk::Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .adjustment(&gtk::Adjustment::new(0.0, 0.0, 100.0, 1.0, 5.0, 0.0))
        .halign(gtk::Align::Fill)
        .can_focus(true)
        .build();

    refresh_ui(&window, &media_player, &duration_bar);

    let video_widget = media_player.borrow().get_gtk_widget();
    video_widget.set_size_request(640, 360);
    
    control_box.append(&video_widget);
    control_box.append(&duration_bar);
    control_box.append(&button_box);

    window.set_child(Some(&control_box));

    window.present();
}
