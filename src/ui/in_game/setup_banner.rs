//! Multiplayer setup-stage countdown banner.
//!
//! A centered top-of-screen banner that announces the 10-second planning window
//! at the start of a multiplayer match and counts down to the battle. Spawned on
//! both peers (the guest mirrors the host's match clock via snapshots) and
//! despawned when the setup stage ends.

use bevy::prelude::*;

use super::displays::spawn_flash_banner_with_marker;
use crate::game::resources::KillStats;
use crate::game::run_conditions::MP_SETUP_DURATION;

/// Color of the setup countdown banner (calm green — this is a friendly prompt,
/// not a warning).
const SETUP_BANNER_COLOR: Color = Color::srgb(0.45, 1.0, 0.55);

/// Marker for the setup-stage countdown banner.
#[derive(Component)]
pub(super) struct SetupCountdownBanner;

/// Spawns the countdown banner on multiplayer match entry. Carries
/// `OnGameplayScreen` (via the shared helper), so it is also cleaned up on match
/// exit even if the setup stage is interrupted.
pub(super) fn spawn_setup_countdown_banner(mut commands: Commands) {
    spawn_flash_banner_with_marker(
        &mut commands,
        &format!("Set up your defenses!  {}s", MP_SETUP_DURATION as u32),
        SETUP_BANNER_COLOR,
        SetupCountdownBanner,
    );
}

/// Updates the countdown text each frame and despawns the banner once the setup
/// stage ends. Reads the (host-authoritative, guest-mirrored) match clock so the
/// countdown is identical on both peers.
pub(super) fn update_setup_countdown_banner(
    kill_stats: Res<KillStats>,
    mut commands: Commands,
    mut banner: Query<(Entity, &mut Text), With<SetupCountdownBanner>>,
) {
    let Ok((entity, mut text)) = banner.single_mut() else {
        return;
    };

    let remaining = MP_SETUP_DURATION - kill_stats.elapsed_time;
    if remaining <= 0.0 {
        commands.entity(entity).try_despawn();
        return;
    }

    let seconds = remaining.ceil() as u32;
    let new_text = format!("Set up your defenses!  {seconds}s");
    if **text != new_text {
        **text = new_text;
    }
}
