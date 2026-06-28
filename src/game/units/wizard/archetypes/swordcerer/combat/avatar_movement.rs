use super::super::components::*;
use super::super::messages::*;
use super::super::resources::{SwordcererPhase, SwordcererState};
use crate::config::GameConfig;
use crate::config::input_bindings::InputBindings;
use crate::game::components::Velocity;
use crate::game::constants::WIZARD_POSITION;
use crate::game::input::messages::BlockSpellInput;
use crate::game::units::components::MovementSpeed;
use crate::game::units::wizard::components::{LocalWizard, Wizard};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use bevy::prelude::*;

/// Initialize or reset the swordcerer state resource.
pub(crate) fn reset_swordcerer_state(mut commands: Commands) {
    commands.insert_resource(SwordcererState::default());
}

/// Blocks normal spell casting while the swordcerer is on the field.
pub(crate) fn block_spells_on_field(
    state: Res<SwordcererState>,
    mut block_spell: MessageWriter<BlockSpellInput>,
) {
    if state.phase != SwordcererPhase::Idle {
        block_spell.write(BlockSpellInput);
    }
}

/// Handles the retreat message — despawns avatar and restores wizard.
pub(crate) fn handle_retreat(
    mut messages: MessageReader<RetreatMessage>,
    mut state: ResMut<SwordcererState>,
    mut commands: Commands,
    avatar_query: Query<Entity, With<SwordcererAvatar>>,
    mut wizard_query: Query<&mut Visibility, (With<Wizard>, With<LocalWizard>)>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
) {
    for _ in messages.read() {
        if state.phase != SwordcererPhase::OnField {
            return;
        }
        // Despawn the avatar
        for entity in &avatar_query {
            commands.entity(entity).try_despawn();
        }
        // Play teleport sound and restore wizard visibility
        audio::play_sfx(
            &mut commands,
            &sfx.teleport_cast,
            WIZARD_POSITION,
            &config,
            &sfx,
        );
        if let Ok(mut visibility) = wizard_query.single_mut() {
            *visibility = Visibility::Inherited;
        }
        state.phase = SwordcererPhase::Idle;
        state.retreated = true;
    }
}

/// Handles WASD / left-stick movement for the swordcerer avatar.
/// Also updates the avatar's `SwordcererFacing` so spells aim along the
/// current movement direction (no separate aim input for this archetype).
#[allow(clippy::too_many_arguments)]
pub(crate) fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    active_device: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
    action_state: Res<crate::game::input::action_state::GamepadActionState>,
    aim_settings: Res<crate::game::input::gamepad::resources::GamepadAimSettings>,
    mut commands: Commands,
    mut avatar_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &MovementSpeed,
            Option<&mut SwordcererFacing>,
        ),
        (With<SwordcererAvatar>, Without<GuestControlledAvatar>),
    >,
    state: Res<SwordcererState>,
) {
    if state.phase != SwordcererPhase::OnField {
        return;
    }

    let Ok((avatar_entity, mut transform, mut velocity, speed, maybe_facing)) =
        avatar_query.single_mut()
    else {
        return;
    };

    let mut input = Vec2::ZERO;
    if let Some(key) = bindings.swordcerer.move_forward
        && keyboard.pressed(key)
    {
        input.y -= 1.0; // -Z is "forward" toward the battlefield
    }
    if let Some(key) = bindings.swordcerer.move_backward
        && keyboard.pressed(key)
    {
        input.y += 1.0;
    }
    if let Some(key) = bindings.swordcerer.move_left
        && keyboard.pressed(key)
    {
        input.x -= 1.0;
    }
    if let Some(key) = bindings.swordcerer.move_right
        && keyboard.pressed(key)
    {
        input.x += 1.0;
    }

    // Gamepad: left stick overrides keyboard when active and deflected.
    // `shape_stick` negates Y so stick-up is -Z (forward toward the battlefield).
    if active_device.is_gamepad() {
        let shaped = crate::game::input::gamepad::systems::shape_stick(
            action_state.left_stick,
            &aim_settings,
        );
        if shaped != Vec2::ZERO {
            input = shaped;
        }
    }

    let input_normalized = if input.length_squared() > 0.0 {
        input.normalize()
    } else {
        Vec2::ZERO
    };

    if input_normalized.length_squared() > 0.0 {
        let facing = SwordcererFacing(input_normalized);
        match maybe_facing {
            Some(mut f) => *f = facing,
            None => {
                commands.entity(avatar_entity).insert(facing);
            }
        }
    }

    apply_avatar_physics(
        &mut transform,
        &mut velocity,
        input_normalized,
        speed.0,
        time.delta_secs(),
    );
}

/// Shared avatar movement physics — acceleration, damping, max-speed clamp, and
/// battlefield-bounds clamp. Used by both the host's own avatar
/// (`player_movement`) and the guest-controlled avatar
/// (`networking::apply_guest_avatar_input`) so they move identically.
pub(crate) fn apply_avatar_physics(
    transform: &mut Transform,
    velocity: &mut Velocity,
    input_dir: Vec2,
    max_speed: f32,
    dt: f32,
) {
    use super::super::constants::*;
    velocity.x += input_dir.x * PLAYER_ACCELERATION * dt;
    velocity.z += input_dir.y * PLAYER_ACCELERATION * dt;

    let damping = PLAYER_DAMPING.powf(dt * 60.0);
    velocity.x *= damping;
    velocity.z *= damping;

    let current_speed = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
    if current_speed > max_speed {
        let scale = max_speed / current_speed;
        velocity.x *= scale;
        velocity.z *= scale;
    }

    transform.translation.x += velocity.x * dt;
    transform.translation.z += velocity.z * dt;

    let half_field = crate::game::constants::BATTLEFIELD_SIZE / 2.0;
    transform.translation.x = transform.translation.x.clamp(-half_field, half_field);
    transform.translation.z = transform.translation.z.clamp(-half_field, half_field);
}
