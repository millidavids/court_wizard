use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    SystemInformationDiagnosticsPlugin,
};
use bevy::prelude::*;

use super::resources::{BenchLogTimer, DiagnosticsEnabled};

pub(super) fn toggle_diagnostics(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut enabled: ResMut<DiagnosticsEnabled>,
    mut timer: ResMut<BenchLogTimer>,
) {
    if !keyboard.just_pressed(KeyCode::F4) {
        return;
    }
    enabled.0 = !enabled.0;
    if enabled.0 {
        timer.0.reset();
        warn!(
            "BENCH START version={} target={} cores={}",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(0),
        );
    } else {
        warn!("BENCH STOP");
    }
}

pub(super) fn log_diagnostics_sampled(
    time: Res<Time<Real>>,
    mut timer: ResMut<BenchLogTimer>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let entities = diagnostics
        .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let proc_cpu = diagnostics
        .get(&SystemInformationDiagnosticsPlugin::PROCESS_CPU_USAGE)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let proc_mem = diagnostics
        .get(&SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let sys_cpu = diagnostics
        .get(&SystemInformationDiagnosticsPlugin::SYSTEM_CPU_USAGE)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let sys_mem = diagnostics
        .get(&SystemInformationDiagnosticsPlugin::SYSTEM_MEM_USAGE)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    warn!(
        "BENCH fps={:.1} frame_ms={:.2} entities={:.0} proc_cpu={:.1}% proc_mem={:.1}% sys_cpu={:.1}% sys_mem={:.1}%",
        fps, frame_ms, entities, proc_cpu, proc_mem, sys_cpu, sys_mem
    );
}

pub(super) fn diagnostics_enabled(enabled: Res<DiagnosticsEnabled>) -> bool {
    enabled.0
}
