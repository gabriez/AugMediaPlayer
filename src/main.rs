use gstreamer::{prelude::*, *};

/// Consideremos lo siguiente. Tenemos dos valores posibles para el cap src del elemento source:
/// - audio/x-raw
/// - video/x-raw
/// Para determinar cuál es cuál, usamos la función starts_with para comparar el nombre del cap con "audio/x-raw".
/// Si el nombre del cap comienza con "audio/x-raw", entonces sabemos que es un cap de audio. Si no, entonces es un cap de video u otro tipo.
/// ¿Será viable crear un enum para clasificar los tipos de caps y usarlo en lugar de strings?
/// Por ejemplo:
enum CapType {
    Audio,
    Video,
    Other,
}

impl CapType {
    fn from_str(cap_name: &str) -> CapType {
        if cap_name.starts_with("audio/x-raw") {
            CapType::Audio
        } else if cap_name.starts_with("video/x-raw") {
            CapType::Video
        } else {
            CapType::Other
        }
    }
}
/// Esto añadiría overhead? Sé que esto es un tutorial y la idea es mantenerlo simple, sin embargo me parece curioso. ¿Para qué otras cosas podría usar este enum?
/// Por ejemplo, podría usarlo para decidir a qué elemento conectar el pad dinámico. Si es audio, lo conecto al audioconvert; si es video, al videoconvert; si es otro tipo, lo ignoro.
/// El tema es que debo usar un match

fn tutorial_main() {
    // Initialize GStreamer
    init().unwrap();

    let uri = "https://gstreamer.freedesktop.org/data/media/sintel_trailer-480p.webm";

    // Create the elements
    let source = ElementFactory::make("uridecodebin")
        .name("source")
        // Set the URI to play
        .property("uri", uri)
        .build()
        .expect("Could not create source element.");
    let convert = ElementFactory::make("audioconvert")
        .name("convert")
        .build()
        .expect("Could not create audioconvert element");
    let resample = ElementFactory::make("audioresample")
        .name("resample")
        .build()
        .expect("Could not create resample element");
    let sink = ElementFactory::make("autoaudiosink")
        .name("sink")
        .build()
        .expect("Could not create sink element");

    let v_convert = ElementFactory::make("videoconvert")
        .name("v_convert")
        .build()
        .expect("Could not create videoconvert element");
    let v_sink = ElementFactory::make("autovideosink")
        .name("v_sink")
        .build()
        .expect("Could not create videosink element");

    // Create the empty pipeline
    let pipeline = Pipeline::with_name("test-pipeline");

    // Build the pipeline
    pipeline
        .add_many([&source, &convert, &resample, &sink, &v_convert, &v_sink])
        .unwrap();
    Element::link_many([&convert, &resample, &sink]).expect("Elements could not be linked.");
    Element::link_many([&v_convert, &v_sink]).expect("Video elements could not be linked");

    source.connect_pad_added(move |src, src_pad| {
        println!("Received new pad {} from {}", src_pad.name(), src.name());

        src.downcast_ref::<Bin>()
            .unwrap()
            .debug_to_dot_file_with_ts(DebugGraphDetails::all(), "pad-added");

        let new_pad_caps = src_pad
            .current_caps()
            .expect("Failed to get caps of new pad.");
        let new_pad_struct = new_pad_caps
            .structure(0)
            .expect("Failed to get first structure of caps.");
        let new_pad_type = new_pad_struct.name();

        let cap_type = CapType::from_str(new_pad_type);

        let sink_pad = match cap_type {
            CapType::Audio => {
                let sink_pad = convert
                    .static_pad("sink")
                    .expect("Failed to get static sink pad from convert");
                sink_pad
            }
            CapType::Video => {
                let sink_pad = v_convert
                    .static_pad("sink")
                    .expect("Failed to get static sink pad from convert");
                sink_pad
            }
            CapType::Other => {
                println!("It has type {new_pad_type}, which is neither audio nor video. Ignoring.");
                return;
            }
        };

        if sink_pad.is_linked() {
            println!("We are already linked. Ignoring.");
            return;
        }

        let res = src_pad.link(&sink_pad);
        if res.is_err() {
            println!("Type is {new_pad_type} but link failed.");
        } else {
            println!("Link succeeded (type {new_pad_type}).");
        }
    });

    // Start playing
    pipeline
        .set_state(State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    // Wait until State Change, error, or EOS
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed_filtered(
        ClockTime::NONE,
        &[
            MessageType::StateChanged,
            MessageType::Error,
            MessageType::Eos,
        ],
    ) {
        use MessageView;

        match msg.view() {
            MessageView::Error(err) => {
                eprintln!(
                    "Error received from element {:?} {}",
                    err.src().map(|s| s.path_string()),
                    err.error()
                );
                eprintln!("Debugging information: {:?}", err.debug());
                break;
            }
            MessageView::StateChanged(state_changed) => {
                if state_changed.src().map(|s| s == &pipeline).unwrap_or(false) {
                    println!(
                        "Pipeline state changed from {:?} to {:?}",
                        state_changed.old(),
                        state_changed.current()
                    );
                }
            }
            MessageView::Eos(..) => break,
            _ => (),
        }
    }

    pipeline
        .set_state(State::Null)
        .expect("Unable to set the pipeline to the `Null` state");
}

fn main() {
    // tutorials_common::run is only required to set up the application environment on macOS
    // (but not necessary in normal Cocoa applications where this is set up automatically)
    tutorial_main()
}
