use {
    crate::desktop_player::DesktopMediaPlayerRef,
    common_media::media_player::{PlaybackControl, SeekControl, VolumeControl},
    gtk::{
        Application, ApplicationWindow, Button, Dialog, Label, ResponseType, Scale,
        glib::{self, clone},
        prelude::*,
    },
};

pub fn refresh_ui(
    window: &ApplicationWindow,
    media_player: &DesktopMediaPlayerRef,
    duration_bar: &Scale,
) {
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
            if media_player_ref.pause().is_err() {
                error_dialog(&window, "There was a problem seeking the media.");
                media_player_ref.set_user_is_seeking(false);
                return glib::Propagation::Stop;
            }
            if let Some(duration) = media_player_ref.duration() {
                let seek_pos = (new_pos / 100_f64) * duration;
                if let Err(err) = media_player_ref.seek_to(seek_pos) {
                    error_dialog(&window, &format!("{}", err));
                    media_player_ref.set_user_is_seeking(false);
                    return glib::Propagation::Stop;
                }
                duration_b.set_value(new_pos);
            }

            if media_player_ref.play().is_err() {
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
                if !media_player_ref.can_seek() {
                    duration_bar.set_value(100.0);
                    return glib::ControlFlow::Continue;
                }
                if media_player_ref.playing()
                    && !media_player_ref.user_is_seeking()
                    && let (Ok(position), Some(duration)) =
                        (media_player_ref.position(), media_player_ref.duration())
                {
                    let bar_position = (100_f64 / duration) * position;
                    duration_bar.set_value(bar_position);
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

pub fn build_buttons(media_player: &DesktopMediaPlayerRef, window: &ApplicationWindow) -> gtk::Box {
    let start_button = Button::builder()
        .icon_name("media-playback-pause") // It will always start and the icon should be paused
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
        move |button| {
            if media_player.borrow().playing() {
                if let Err(err) = media_player.borrow().pause() {
                    println!("Error pausing player: {:?}", err);
                    error_dialog(&window, &format!("{}", err));
                }
                button.set_icon_name("media-playback-start");
            } else {
                if let Err(err) = media_player.borrow().play() {
                    println!("Error playing player: {:?}", err);
                    error_dialog(&window, &format!("{}", err));
                }
                button.set_icon_name("media-playback-pause");
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
            if let Err(err) = media_player.borrow().stop() {
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
            if let Err(err) = media_player.borrow().seek_forward() {
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
            if let Err(err) = media_player.borrow().seek_backward() {
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
    button_box.append(&start_button);
    button_box.append(&stop_button);
    button_box.append(&forward_button);

    button_box
}

pub fn build_volume_controls(media_player: &DesktopMediaPlayerRef) -> gtk::Box {
    let volume_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .halign(gtk::Align::Fill)
        .spacing(6)
        .margin_top(2)
        .margin_bottom(2)
        .build();

    // Mute button
    let mute_button = Button::builder()
        .icon_name(if media_player.borrow().is_muted() {
            "audio-volume-muted-symbolic"
        } else {
            "audio-volume-high-symbolic"
        })
        .margin_start(6)
        .margin_end(6)
        .build();

    mute_button.connect_clicked(clone!(
        #[weak]
        media_player,
        move |button| {
            media_player.borrow_mut().toggle_mute();

            // Update button icon based on mute state
            if media_player.borrow().is_muted() {
                button.set_icon_name("audio-volume-muted-symbolic");
            } else {
                button.set_icon_name("audio-volume-high-symbolic");
            }
        }
    ));

    // Volume label
    let volume_label = Label::builder().label("Volume:").margin_start(6).build();

    // Volume slider (0-100)
    let volume_slider = Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .adjustment(&gtk::Adjustment::new(
            media_player.borrow().get_volume() * 100.0,
            0.0,
            100.0,
            1.0,
            5.0,
            0.0,
        ))
        .hexpand(true)
        .width_request(150)
        .build();

    volume_slider.connect_value_changed(clone!(
        #[weak]
        media_player,
        move |slider| {
            let volume = slider.value() / 100.0;
            media_player.borrow_mut().set_volume(volume);
        }
    ));

    volume_box.append(&volume_label);
    volume_box.append(&volume_slider);
    volume_box.append(&mute_button);

    volume_box
}

pub fn build_ui(app: &Application, media_player: DesktopMediaPlayerRef, video_widget: gtk::Widget) {
    let window: ApplicationWindow = ApplicationWindow::builder()
        .application(app)
        .title("AugMediaPlayer")
        .build();

    let button_box = build_buttons(&media_player, &window);
    let volume_box = build_volume_controls(&media_player);

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

    video_widget.set_size_request(640, 360);

    control_box.append(&video_widget);
    control_box.append(&duration_bar);
    control_box.append(&button_box);
    control_box.append(&volume_box);

    window.set_child(Some(&control_box));

    window.present();
}
