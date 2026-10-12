#[cfg(test)]
mod tests {
    use crate::features::preset_editor::{WAVEFORM_BARS, waveform_peaks};

    #[test]
    fn silence_and_empty_audio_stay_flat() {
        assert_eq!(waveform_peaks(&[]), [0.; WAVEFORM_BARS]);
        assert_eq!(waveform_peaks(&[0.; 240]), [0.; WAVEFORM_BARS]);
    }

    #[test]
    fn retains_peaks_without_normalizing_quiet_audio() {
        let mut samples = [0.; 96];
        samples[1] = -0.2;
        samples[95] = 0.7;
        let peaks = waveform_peaks(&samples);
        assert_eq!(peaks[0], 0.2);
        assert_eq!(peaks[47], 0.7);
        assert_eq!(peaks[20], 0.);
    }

    #[test]
    fn rejects_nonfinite_samples_and_clamps_amplitude() {
        let peaks = waveform_peaks(&[f32::NAN, f32::INFINITY, -4., 0.5]);
        assert!(
            peaks
                .iter()
                .all(|peak| peak.is_finite() && (0. ..=1.).contains(peak))
        );
        assert_eq!(peaks[24], 1.);
    }
}
