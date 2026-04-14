use bevy::prelude::*;

/// When true, the sampled diagnostics logger emits a line each time
/// `BenchLogTimer` fires. Toggled with F3.
#[derive(Resource, Default, PartialEq, Eq)]
pub(super) struct DiagnosticsEnabled(pub bool);

/// Repeating timer controlling how often a sampled diagnostics line is
/// written. Interval is tuned to keep logs readable without flooding.
#[derive(Resource)]
pub(super) struct BenchLogTimer(pub Timer);

impl Default for BenchLogTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}
