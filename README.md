# Goal

Develop a native video player application written in Rust that supports basic video playback and
control.
Functional Requirements
The player must support:

1. Open a video file
   - Load a local video file (e.g. .mp4, .mkv, .webm)
   - Handle invalid or unsupported files gracefully
2. Play / Pause
   - Start and pause video playback
3. Seek controls
   - Move playback 10 seconds forward
   - Move playback 10 seconds backward
4. Volume control
   - Adjust playback volume
   - Support mute and maximum volume

## Install dependencies

First update the system dependencies:

```
sudo apt update
```

Then install the required dependencies:

```
sudo apt install libgtk-4-dev build-essential
sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
      gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
      gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
      gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev
```

gst-plugins-rs relies on cargo-c to generate shared and static C libraries. It can be installed using:

```
cargo install cargo-c
```

## Build and run

To build the project, run:

```
cargo build --release
```

To run the project, run:

```
cargo run --release -- --uri <URI> file/http
```

The commands file or http are required to specify the origin of the media to play.
