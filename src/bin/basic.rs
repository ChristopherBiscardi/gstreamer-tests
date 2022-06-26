use camino::Utf8Path;
use gst::{
    prelude::*, ClockTime, Element, ElementFactory,
    MessageView, Pipeline, State,
};
use gstreamer as gst;

const FORMAT: &str = "xRGB";
const WIDTH: i32 = 1280;
const HEIGHT: i32 = 720;
const FRAMERATE: i32 = 30 / 1;

fn tutorial_main() {
    // Initialize GStreamer
    gst::init().unwrap();

    let mut source_element = "filesrc";
    if cfg!(feature = "test_video") {
        source_element = "videotestsrc";
    }

    // File Source
    let source = ElementFactory::make(
        source_element,
        Some("source"),
    )
    .expect("Could not create source element.");

    if !cfg!(feature = "test_video") {
        // Make sure file exists because gstreamer won't
        let video = Utf8Path::new("test.mp4");
        let video_exists = video.exists();
        if !video_exists {
            panic!(
                "video file {} needs to exist and does not",
                "test.mp4"
            );
        };
        source.set_property("location", "test.mp4");
    }

    let capsfilter_video = ElementFactory::make(
        "capsfilter",
        Some("capsfilter"),
    )
    .expect("this is totally gonna fail");

    let caps = gst::Caps::builder("video/x-raw")
        .field("format", FORMAT)
        .field("width", WIDTH)
        .field("height", HEIGHT)
        .field("framerate", gst::Fraction::new(30, 1))
        .build();
    // let caps = gst::Caps::new_simple(
    //     "video/x-raw",
    //     &[
    //         ("format", &GRAY),
    //         ("width", &WIDTH),
    //         ("height", &HEIGHT),
    //         ("framerate", &FRAMERATE),
    //     ],
    // );
    capsfilter_video.set_property("caps", caps);
    let convert = ElementFactory::make(
        "autovideoconvert",
        Some("convert"),
    )
    .expect("Could not create covert element.");

    // Remove Silence
    let middle_sink = ElementFactory::make(
        "removesilence",
        Some("silence"),
    )
    .expect("Could not create silence element");

    middle_sink.set_property("squash", true);

    // Play Video "osxvideosink" on mac? "autovideosink" will likely choose "glimagesink"
    // which was failing for me
    let sink =
        ElementFactory::make("osxvideosink", Some("sink"))
            .expect("Could not create sink element");

    // Create the empty pipeline
    let pipeline = Pipeline::new(Some("test-pipeline"));

    // Build the pipeline
    pipeline
        .add_many(&[
            &source,
            &capsfilter_video,
            &convert,
            &sink,
        ])
        .unwrap();

    // link source later
    Element::link_many(&[
        &source,
        &capsfilter_video,
        &convert,
        &sink,
    ])
    .expect("this to work");

    // source.connect_pad_added(move |src, src_pad| {
    //     println!("Received new pad {} from {}", src_pad.name(), src.name());

    //     let sink_pad = convert
    //         .static_pad("sink")
    //         .expect("Failed to get static sink pad from convert");
    //     if sink_pad.is_linked() {
    //         println!("We are already linked. Ignoring.");
    //         return;
    //     }

    //     let new_pad_caps = src_pad
    //         .current_caps()
    //         .expect("Failed to get caps of new pad.");
    //     let new_pad_struct = new_pad_caps
    //         .structure(0)
    //         .expect("Failed to get first structure of caps.");
    //     let new_pad_type = new_pad_struct.name();

    //     let is_audio = new_pad_type.starts_with("audio/x-raw");
    //     if !is_audio {
    //         println!(
    //             "It has type {} which is not raw audio. Ignoring.",
    //             new_pad_type
    //         );
    //         return;
    //     }

    //     let res = src_pad.link(&sink_pad);
    //     if res.is_err() {
    //         println!("Type is {} but link failed.", new_pad_type);
    //     } else {
    //         println!("Link succeeded (type {}).", new_pad_type);
    //     }
    // });

    // Start playing
    pipeline.set_state(State::Playing).expect(
        "Unable to set the pipeline to the `Playing` state",
    );

    // Wait until error or EOS
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(ClockTime::NONE) {
        match msg.view() {
            MessageView::Error(err) => {
                eprintln!(
                    "Error received from element {:?}: {}",
                    err.src().map(|s| s.path_string()),
                    err.error()
                );
                eprintln!(
                    "Debugging information: {:?}",
                    err.debug()
                );
                break;
            }
            MessageView::Eos(..) => break,
            _ => (),
        }
    }

    pipeline.set_state(State::Null).expect(
        "Unable to set the pipeline to the `Null` state",
    );
}

fn main() {
    // tutorials_common::run is only required to set up the application environment on macOS
    // (but not necessary in normal Cocoa applications where this is set up automatically)
    tutorials_common::run(tutorial_main);
}
