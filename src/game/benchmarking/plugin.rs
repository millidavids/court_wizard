use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
};
use bevy::prelude::*;

use super::resources::{BenchLogTimer, DiagnosticsEnabled};
use super::systems::{diagnostics_enabled, log_diagnostics_sampled, toggle_diagnostics};

/// Registers Bevy's diagnostic plugins, the F3 toggle, and a sampled
/// logger that emits one `BENCH ...` line every few seconds while enabled.
///
/// Only added to the app when the `benchmarking` cargo feature is on.
pub(crate) struct BenchmarkingPlugin;

impl Plugin for BenchmarkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiagnosticsEnabled>()
            .init_resource::<BenchLogTimer>()
            .add_plugins((
                FrameTimeDiagnosticsPlugin::default(),
                EntityCountDiagnosticsPlugin::default(),
                SystemInformationDiagnosticsPlugin,
            ))
            .add_systems(
                Update,
                (
                    toggle_diagnostics,
                    log_diagnostics_sampled.run_if(diagnostics_enabled),
                ),
            );
    }
}
