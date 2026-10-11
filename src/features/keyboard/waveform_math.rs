pub(super) const WAVEFORM_BARS: usize = 48;

pub(super) fn waveform_peaks(samples: &[f32]) -> [f32; WAVEFORM_BARS] {
    let mut peaks = [0_f32; WAVEFORM_BARS];
    if samples.is_empty() {
        return peaks;
    }
    for (index, sample) in samples.iter().enumerate() {
        if sample.is_finite() {
            let bin = index * WAVEFORM_BARS / samples.len();
            peaks[bin] = peaks[bin].max(sample.abs().min(1.));
        }
    }
    peaks
}

#[cfg(test)]
mod tests {
    use super::*;

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
