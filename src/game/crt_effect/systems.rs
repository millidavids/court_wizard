use bevy::prelude::*;

use super::components::{
    ChannelChangeTimer, CrtEffectSettings, DesaturationTimer, ScreenFlashTimer, VignettePulseTimer,
};
use super::constants::{CHANNEL_CHANGE_DURATION, DESATURATION_DURATION};
use super::messages::{
    ChannelChangeMessage, ScreenDesaturateMessage, ScreenFlashMessage, VignettePulseMessage,
};

/// Converts NDC coordinates (-1..1) to screen UV (0..1), flipping Y for screen space.
pub(super) fn ndc_to_uv(ndc: Vec3) -> Vec2 {
    Vec2::new((ndc.x + 1.0) * 0.5, 1.0 - (ndc.y + 1.0) * 0.5)
}

/// Reads `ChannelChangeMessage` and starts the channel-change animation.
pub(super) fn handle_channel_change_message(
    mut commands: Commands,
    mut messages: MessageReader<ChannelChangeMessage>,
    existing_timer: Option<Res<ChannelChangeTimer>>,
) {
    if messages.read().next().is_some() {
        // Don't restart if already animating
        if existing_timer.is_none() {
            commands.insert_resource(ChannelChangeTimer::new(CHANNEL_CHANGE_DURATION));
        }
    }
}

/// Ticks the channel-change timer, writes intensity to CrtEffectSettings,
/// and removes the timer when the animation is complete.
pub(super) fn animate_channel_change(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ChannelChangeTimer>,
    mut query: Query<&mut CrtEffectSettings>,
) {
    timer.elapsed += time.delta_secs();

    let intensity = timer.intensity();

    for mut settings in &mut query {
        settings.channel_change = intensity;
        settings.channel_change_time = timer.elapsed;
    }

    if timer.is_finished() {
        // Reset to zero before removing
        for mut settings in &mut query {
            settings.channel_change = 0.0;
            settings.channel_change_time = 0.0;
        }
        commands.remove_resource::<ChannelChangeTimer>();
    }
}

/// Reads `ScreenDesaturateMessage` and starts the desaturation animation.
pub(super) fn handle_desaturation_message(
    mut commands: Commands,
    mut messages: MessageReader<ScreenDesaturateMessage>,
    existing_timer: Option<Res<DesaturationTimer>>,
) {
    if messages.read().next().is_some() && existing_timer.is_none() {
        commands.insert_resource(DesaturationTimer::new(DESATURATION_DURATION));
    }
}

/// Handles an incoming `ScreenFlashMessage`, inserting or replacing the `ScreenFlashTimer`
/// resource so the flash animation plays from the beginning.
///
/// A new flash always takes priority over an in-progress one.
pub(super) fn handle_screen_flash_message(
    mut commands: Commands,
    mut messages: MessageReader<ScreenFlashMessage>,
    existing_timer: Option<Res<ScreenFlashTimer>>,
) {
    if let Some(msg) = messages.read().next() {
        // Replace existing flash (new one takes priority)
        if existing_timer.is_some() {
            commands.remove_resource::<ScreenFlashTimer>();
        }
        commands.insert_resource(ScreenFlashTimer::new(
            msg.color,
            msg.duration,
            msg.intensity,
        ));
    }
}

/// Ticks the screen flash timer, writes color/intensity to CrtEffectSettings.
pub(super) fn animate_screen_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<ScreenFlashTimer>,
    mut query: Query<&mut CrtEffectSettings>,
) {
    timer.elapsed += time.delta_secs();
    let intensity = timer.intensity();

    for mut settings in &mut query {
        settings.screen_flash_r = timer.color[0];
        settings.screen_flash_g = timer.color[1];
        settings.screen_flash_b = timer.color[2];
        settings.screen_flash_intensity = intensity;
    }

    if timer.is_finished() {
        for mut settings in &mut query {
            settings.screen_flash_intensity = 0.0;
        }
        commands.remove_resource::<ScreenFlashTimer>();
    }
}

/// Reads `VignettePulseMessage` and starts the vignette pulse animation.
pub(super) fn handle_vignette_pulse_message(
    mut commands: Commands,
    mut messages: MessageReader<VignettePulseMessage>,
    existing_timer: Option<Res<VignettePulseTimer>>,
) {
    if let Some(msg) = messages.read().next() {
        if existing_timer.is_some() {
            commands.remove_resource::<VignettePulseTimer>();
        }
        commands.insert_resource(VignettePulseTimer::new(msg.duration, msg.intensity));
    }
}

/// Ticks the vignette pulse timer, writes intensity to CrtEffectSettings.
pub(super) fn animate_vignette_pulse(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<VignettePulseTimer>,
    mut query: Query<&mut CrtEffectSettings>,
) {
    timer.elapsed += time.delta_secs();
    let intensity = timer.intensity();

    for mut settings in &mut query {
        settings.vignette_pulse = intensity;
    }

    if timer.is_finished() {
        for mut settings in &mut query {
            settings.vignette_pulse = 0.0;
        }
        commands.remove_resource::<VignettePulseTimer>();
    }
}

/// Ticks the desaturation timer, writes intensity to CrtEffectSettings,
/// and removes the timer when the animation is complete.
pub(super) fn animate_desaturation(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<DesaturationTimer>,
    mut query: Query<&mut CrtEffectSettings>,
) {
    timer.elapsed += time.delta_secs();

    let intensity = timer.intensity();

    for mut settings in &mut query {
        settings.desaturation = intensity;
    }

    if timer.is_finished() {
        for mut settings in &mut query {
            settings.desaturation = 0.0;
        }
        commands.remove_resource::<DesaturationTimer>();
    }
}
