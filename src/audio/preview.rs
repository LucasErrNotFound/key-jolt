use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use rodio::{Decoder, DeviceSinkBuilder, Player};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewEnd {
    Stopped,
    Finished,
}

pub fn play_file(
    path: &Path,
    stop_requested: Arc<AtomicBool>,
    volume: f32,
) -> Result<PreviewEnd, String> {
    let file = File::open(path).map_err(|error| format!("Could not read audio file: {error}"))?;
    let decoder = Decoder::try_from(file)
        .map_err(|error| format!("This audio format could not be decoded: {error}"))?;
    let output = DeviceSinkBuilder::open_default_sink()
        .map_err(|error| format!("Could not open an audio output device: {error}"))?;
    let player = Player::connect_new(output.mixer());
    player.set_volume(volume.clamp(0.0, 2.0));
    player.append(decoder);

    while !player.empty() {
        if stop_requested.load(Ordering::Acquire) {
            player.stop();
            return Ok(PreviewEnd::Stopped);
        }

        thread::sleep(Duration::from_millis(20));
    }

    if stop_requested.load(Ordering::Acquire) {
        Ok(PreviewEnd::Stopped)
    } else {
        Ok(PreviewEnd::Finished)
    }
}
