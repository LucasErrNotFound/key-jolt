use super::*;

#[test]
fn meter_height_tracks_gain_and_remains_empty_without_sound_activity() {
    let now = Instant::now();
    let mut state = MeterState::default();
    assert_eq!(
        state
            .tick(ActivitySnapshot::default(), true, true, now, false)
            .segments,
        0
    );
    for (gain, segments) in [(0.5, 4), (1.0, 8), (1.5, 12), (2.0, 16)] {
        assert_eq!(
            state
                .tick(
                    ActivitySnapshot { input: true, gain },
                    true,
                    true,
                    now,
                    false
                )
                .segments,
            segments
        );
    }
}

#[test]
fn pulses_hold_then_decay_to_idle_without_further_input() {
    let now = Instant::now();
    let mut state = MeterState::default();
    state.tick(
        ActivitySnapshot {
            input: true,
            gain: 2.0,
        },
        true,
        true,
        now,
        false,
    );
    assert_eq!(
        state
            .tick(ActivitySnapshot::default(), true, true, now + HOLD, false)
            .segments,
        16
    );
    let middle = state.tick(
        ActivitySnapshot::default(),
        true,
        true,
        now + HOLD + DECAY / 2,
        false,
    );
    assert_eq!(middle.segments, 8);
    assert!((middle.dot - 0.5).abs() < f32::EPSILON);
    assert!(
        state.tick(
            ActivitySnapshot::default(),
            true,
            true,
            now + HOLD + DECAY,
            false
        ) == MeterFrame::default()
    );
}

#[test]
fn muted_or_disabled_channels_clear_both_indicators_without_replaying_old_activity() {
    let now = Instant::now();
    let mut state = MeterState::default();
    state.tick(
        ActivitySnapshot {
            input: true,
            gain: 2.0,
        },
        true,
        true,
        now,
        false,
    );
    let frame = state.tick(
        ActivitySnapshot {
            input: true,
            gain: 2.0,
        },
        false,
        false,
        now,
        false,
    );
    assert_eq!(frame.segments, 0);
    assert_eq!(frame.dot, 0.0);
    assert!(
        state.tick(ActivitySnapshot::default(), true, true, now, false) == MeterFrame::default()
    );
}

#[test]
fn unassigned_input_lights_only_the_dot() {
    let frame = MeterState::default().tick(
        ActivitySnapshot {
            input: true,
            gain: 0.0,
        },
        true,
        true,
        Instant::now(),
        false,
    );
    assert_eq!(frame.segments, 0);
    assert_eq!(frame.dot, 1.0);
}

#[test]
fn retriggering_refreshes_the_pulse_without_accumulating_gain() {
    let now = Instant::now();
    let mut state = MeterState::default();
    state.tick(
        ActivitySnapshot {
            input: true,
            gain: 2.0,
        },
        true,
        true,
        now,
        false,
    );
    let later = now + HOLD + DECAY / 2;
    assert_eq!(
        state
            .tick(
                ActivitySnapshot {
                    input: true,
                    gain: 1.0
                },
                true,
                true,
                later,
                false
            )
            .segments,
        8
    );
    assert_eq!(
        state
            .tick(ActivitySnapshot::default(), true, true, later + HOLD, false)
            .segments,
        8
    );
}

#[test]
fn reduced_motion_holds_a_static_indicator_then_clears_it() {
    let now = Instant::now();
    let mut state = MeterState::default();
    state.tick(
        ActivitySnapshot {
            input: true,
            gain: 1.0,
        },
        true,
        true,
        now,
        true,
    );
    let frame = state.tick(
        ActivitySnapshot::default(),
        true,
        true,
        now + HOLD + DECAY / 2,
        true,
    );
    assert_eq!(frame.segments, 8);
    assert_eq!(frame.dot, 1.0);
    assert!(
        state.tick(
            ActivitySnapshot::default(),
            true,
            true,
            now + HOLD + DECAY,
            true
        ) == MeterFrame::default()
    );
}

#[test]
fn reset_removes_old_activity_before_returning_to_the_home_page() {
    let now = Instant::now();
    let mut state = MeterState::default();
    state.tick(
        ActivitySnapshot {
            input: true,
            gain: 2.0,
        },
        true,
        true,
        now,
        false,
    );
    state.reset();
    assert!(
        state.tick(ActivitySnapshot::default(), true, true, now, false) == MeterFrame::default()
    );
}

#[test]
fn invalid_gain_is_ignored_and_excessive_gain_is_clamped() {
    let now = Instant::now();
    let mut state = MeterState::default();
    for gain in [f32::NAN, f32::INFINITY, -1.0] {
        assert_eq!(
            state
                .tick(
                    ActivitySnapshot { input: false, gain },
                    true,
                    true,
                    now,
                    false
                )
                .segments,
            0
        );
    }
    assert_eq!(
        state
            .tick(
                ActivitySnapshot {
                    input: false,
                    gain: 4.0
                },
                true,
                true,
                now,
                false
            )
            .segments,
        SEGMENTS
    );
}

#[test]
fn zero_volume_clears_the_meter_but_allows_input_activity_in_an_enabled_channel() {
    let now = Instant::now();
    let mut state = MeterState::default();
    state.tick(
        ActivitySnapshot {
            input: true,
            gain: 2.0,
        },
        true,
        true,
        now,
        false,
    );
    let frame = state.tick(
        ActivitySnapshot {
            input: true,
            gain: 2.0,
        },
        true,
        false,
        now,
        false,
    );
    assert_eq!(frame.segments, 0);
    assert_eq!(frame.dot, 1.0);
    let frame = state.tick(ActivitySnapshot::default(), true, true, now, false);
    assert_eq!(frame.segments, 0);
}

#[test]
fn sustained_disabled_input_does_not_queue_a_pulse_for_reenable() {
    let now = Instant::now();
    let mut state = MeterState::default();
    for offset in 0..10 {
        let frame = state.tick(
            ActivitySnapshot {
                input: true,
                gain: 2.0,
            },
            false,
            false,
            now + Duration::from_millis(offset * 33),
            false,
        );
        assert!(frame == MeterFrame::default());
    }
    assert!(
        state.tick(
            ActivitySnapshot::default(),
            true,
            true,
            now + Duration::from_secs(1),
            false
        ) == MeterFrame::default()
    );
    let frame = state.tick(
        ActivitySnapshot {
            input: true,
            gain: 1.0,
        },
        true,
        true,
        now + Duration::from_secs(1),
        false,
    );
    assert_eq!(frame.segments, 8);
    assert_eq!(frame.dot, 1.0);
}

#[test]
fn muting_one_channel_keeps_the_other_channel_reactive() {
    let now = Instant::now();
    let mut keyboard = MeterState::default();
    let mut mouse = MeterState::default();
    let input = ActivitySnapshot {
        input: true,
        gain: 1.0,
    };
    keyboard.tick(input, true, true, now, false);
    let muted = keyboard.tick(input, false, false, now, false);
    let active = mouse.tick(input, true, true, now, false);
    assert!(muted == MeterFrame::default());
    assert_eq!(active.segments, 8);
    assert_eq!(active.dot, 1.0);
}
