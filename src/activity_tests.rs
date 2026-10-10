use super::*;

#[test]
fn hidden_activity_is_discarded_and_reopening_starts_empty() {
    let monitor = ActivityMonitor::new();
    monitor.record_input(ActivityChannel::Keyboard);
    monitor.record_sound(ActivityChannel::Keyboard, 2.0);
    monitor.set_observed(true);
    assert_eq!(
        monitor.take(ActivityChannel::Keyboard),
        ActivitySnapshot::default()
    );
    monitor.record_sound(ActivityChannel::Keyboard, 1.0);
    monitor.set_observed(false);
    monitor.record_input(ActivityChannel::Keyboard);
    monitor.set_observed(true);
    assert_eq!(
        monitor.take(ActivityChannel::Keyboard),
        ActivitySnapshot::default()
    );
}

#[test]
fn bursts_coalesce_to_one_input_flag_and_the_largest_gain() {
    let monitor = ActivityMonitor::new();
    monitor.set_observed(true);
    for gain in [0.25, 1.5, 0.75, 1.0] {
        monitor.record_input(ActivityChannel::Keyboard);
        monitor.record_sound(ActivityChannel::Keyboard, gain);
    }
    assert_eq!(
        monitor.take(ActivityChannel::Keyboard),
        ActivitySnapshot {
            input: true,
            gain: 1.5
        }
    );
    assert_eq!(
        monitor.take(ActivityChannel::Keyboard),
        ActivitySnapshot::default()
    );
}

#[test]
fn keyboard_and_mouse_activity_are_independent() {
    let monitor = ActivityMonitor::new();
    monitor.set_observed(true);
    monitor.record_input(ActivityChannel::Mouse);
    monitor.record_sound(ActivityChannel::Keyboard, 1.0);
    assert_eq!(
        monitor.take(ActivityChannel::Keyboard),
        ActivitySnapshot {
            input: false,
            gain: 1.0
        }
    );
    assert_eq!(
        monitor.take(ActivityChannel::Mouse),
        ActivitySnapshot {
            input: true,
            gain: 0.0
        }
    );
}

#[test]
fn invalid_or_silent_gain_does_not_light_the_meter() {
    let monitor = ActivityMonitor::new();
    monitor.set_observed(true);
    for gain in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
        monitor.record_sound(ActivityChannel::Keyboard, gain);
    }
    assert_eq!(
        monitor.take(ActivityChannel::Keyboard),
        ActivitySnapshot::default()
    );
    monitor.record_sound(ActivityChannel::Keyboard, 10.0);
    assert_eq!(monitor.take(ActivityChannel::Keyboard).gain, 2.0);
}

#[test]
fn concurrent_writers_preserve_pending_input_and_gain_without_a_queue() {
    let monitor = ActivityMonitor::new();
    monitor.set_observed(true);
    std::thread::scope(|scope| {
        for gain in [0.5, 1.0, 1.5, 2.0] {
            let monitor = &monitor;
            scope.spawn(move || {
                for _ in 0..5_000 {
                    monitor.record_input(ActivityChannel::Keyboard);
                    monitor.record_sound(ActivityChannel::Keyboard, gain);
                }
            });
        }
    });
    assert_eq!(
        monitor.take(ActivityChannel::Keyboard),
        ActivitySnapshot {
            input: true,
            gain: 2.0
        }
    );
    assert_eq!(
        monitor.take(ActivityChannel::Keyboard),
        ActivitySnapshot::default()
    );
}
