use super::waveform_math::{WAVEFORM_BARS, waveform_peaks};
use gpui_kit::SharedString;
use rodio::{Decoder, Source};
use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub(crate) enum AudioDetails {
    Pending,
    Ready {
        duration: Option<Duration>,
        peaks: [f32; WAVEFORM_BARS],
        prefix: bool,
    },
    Unavailable(SharedString),
}

pub(crate) fn analyze_file(path: &Path, cancelled: &AtomicBool) -> AudioDetails {
    match read_audio_details(path, cancelled) {
        Ok(details) => details,
        Err(message) => AudioDetails::Unavailable(message.into()),
    }
}

fn read_audio_details(path: &Path, cancelled: &AtomicBool) -> Result<AudioDetails, String> {
    const MAX_BYTES: u64 = 64 * 1024 * 1024;
    const MAX_SAMPLES: usize = 1_000_000;
    let file = File::open(path).map_err(|_| "Cannot read audio file".to_string())?;
    if file
        .metadata()
        .map_err(|_| "Cannot read audio metadata".to_string())?
        .len()
        > MAX_BYTES
    {
        return Err("Waveform unavailable for files over 64 MB".into());
    }
    if cancelled.load(Ordering::Relaxed) {
        return Err("Analysis cancelled".into());
    }
    let started = Instant::now();
    let mut source = Decoder::try_from(file)
        .map_err(|_| "Audio details unavailable for this format".to_string())?;
    let sample_rate = f64::from(source.sample_rate().get());
    let channels = f64::from(source.channels().get());
    let known_duration = source.total_duration();
    let sample_limit = MAX_SAMPLES.min((sample_rate * channels * 30.) as usize);
    let mut samples = Vec::with_capacity(sample_limit.min(65_536));
    let mut complete = false;
    for index in 0..sample_limit {
        if index.is_multiple_of(4096)
            && (cancelled.load(Ordering::Relaxed) || started.elapsed() >= Duration::from_secs(2))
        {
            break;
        }
        let Some(sample) = source.next() else {
            complete = true;
            break;
        };
        samples.push(sample);
    }
    let duration = known_duration.or_else(|| {
        complete.then(|| Duration::from_secs_f64(samples.len() as f64 / (sample_rate * channels)))
    });
    Ok(AudioDetails::Ready {
        duration,
        peaks: waveform_peaks(&samples),
        prefix: !complete,
    })
}
