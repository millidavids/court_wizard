//! Multiplayer host-authoritative control of the GUEST's Swordcerer avatar.
//!
//! The avatar is always a host-simulated unit. When the GUEST is the Swordcerer
//! it can't spawn the avatar itself, so:
//! - the guest asks the host to spawn it (`SwordcererAvatarSpawn`) on the guest's
//!   army (Attackers) — the host owns the entity, combat, and death;
//! - the guest streams control input each frame (`SwordcererAvatarInput`); the
//!   host applies movement + missile + sword swing;
//! - on death the host despawns the avatar and tells the guest
//!   (`SwordcererAvatarDied`) so it reappears as a caster.
//!
//! The host renders the avatar locally; the guest sees it as a ghost via the
//! `UnitFlags::SWORDCERER_AVATAR` snapshot flag.

use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::messages::RetreatMessage;
use super::resources::{SwordcererAssets, SwordcererPhase, SwordcererState};
use super::ui::spawn_swordcerer_avatar;
use crate::config::{GameConfig, WizardType};
use crate::game::components::Velocity;
use crate::game::units::components::{Corpse, Health, MovementSpeed, Team};
use crate::game::units::wizard::spells::audio::SpellSfxAssets;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::NetworkConnection;
use crate::networking::session::MultiplayerSession;

/// True when the OPPONENT is the Swordcerer — the host-side gate for driving the
/// guest's avatar.
pub(crate) fn is_remote_swordcerer(session: Option<Res<MultiplayerSession>>) -> bool {
    session.is_some_and(|s| s.remote_wizard() == WizardType::Swordcerer)
}

/// Host: spawns the guest's avatar (on the guest's Attacker army) when the guest
/// deploys. Guarded against duplicate spawns.
pub(crate) fn receive_swordcerer_spawn(
    mut commands: Commands,
    connection: Option<ResMut<NetworkConnection>>,
    swordcerer_assets: Res<SwordcererAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<&Team, With<SwordcererAvatar>>,
) {
    let Some(mut connection) = connection else {
        return;
    };
    if connection.incoming_messages.is_empty() {
        return;
    }
    // Mutable so a duplicate spawn message in the SAME batch doesn't spawn a
    // second avatar (the just-spawned entity isn't visible to `existing` yet —
    // commands are deferred).
    let mut already_deployed = existing.iter().any(|t| *t == Team::Attackers);
    let messages: Vec<NetworkMessage> = std::mem::take(&mut connection.incoming_messages);
    let mut unhandled = Vec::new();
    for msg in messages {
        match msg {
            NetworkMessage::SwordcererAvatarSpawn { x, z } => {
                if !already_deployed {
                    already_deployed = true;
                    let avatar = spawn_swordcerer_avatar(
                        &mut commands,
                        &swordcerer_assets,
                        &mut materials,
                        Vec3::new(x, 0.0, z),
                        Team::Attackers,
                    );
                    commands.entity(avatar).insert(GuestControlledAvatar);
                }
            }
            other => unhandled.push(other),
        }
    }
    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Guest: streams the local control input to the host while the avatar is on the
/// field. Movement is WASD or the gamepad left stick; fire is right-click / LT,
/// swing is left-click / RT. Fire and swing read the shared `MouseRightPressed` /
/// `MouseLeftPressed` messages (the same ones the host's avatar consumes) so
/// gamepad triggers — which the input layer already translates into those
/// messages — drive the avatar identically to the host.
#[allow(clippy::too_many_arguments)]
pub(crate) fn send_swordcerer_avatar_input(
    state: Res<SwordcererState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<crate::config::input_bindings::InputBindings>,
    active_device: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
    action_state: Res<crate::game::input::action_state::GamepadActionState>,
    aim_settings: Res<crate::game::input::gamepad::resources::GamepadAimSettings>,
    mut fire_pressed: MessageReader<crate::game::input::messages::MouseRightPressed>,
    mut swing_pressed: MessageReader<crate::game::input::messages::MouseLeftPressed>,
    connection: Option<ResMut<NetworkConnection>>,
    mut last_facing: Local<Vec2>,
    mut commands: Commands,
    ghost_avatar: Query<
        &Transform,
        With<crate::game::multiplayer::components::GhostSwordcererAvatar>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Drain the press messages every frame (even when we bail early) so a click
    // made before deploying doesn't leak into the first on-field frame.
    let fire = fire_pressed.read().next().is_some();
    let swing = swing_pressed.read().next().is_some();

    if state.phase != SwordcererPhase::OnField {
        return;
    }
    let Some(mut connection) = connection else {
        return;
    };

    let mut input = Vec2::ZERO;
    if bindings
        .swordcerer
        .move_forward
        .is_some_and(|k| keyboard.pressed(k))
    {
        input.y -= 1.0;
    }
    if bindings
        .swordcerer
        .move_backward
        .is_some_and(|k| keyboard.pressed(k))
    {
        input.y += 1.0;
    }
    if bindings
        .swordcerer
        .move_left
        .is_some_and(|k| keyboard.pressed(k))
    {
        input.x -= 1.0;
    }
    if bindings
        .swordcerer
        .move_right
        .is_some_and(|k| keyboard.pressed(k))
    {
        input.x += 1.0;
    }

    // Gamepad left stick overrides keyboard when active and deflected (mirrors
    // the host avatar's `player_movement`). `shape_stick` negates Y so stick-up
    // is -Z (forward toward the battlefield).
    if active_device.is_gamepad() {
        let shaped = crate::game::input::gamepad::systems::shape_stick(
            action_state.left_stick,
            &aim_settings,
        );
        if shaped != Vec2::ZERO {
            input = shaped;
        }
    }

    let dir = if input.length_squared() > 0.0 {
        let d = input.normalize();
        *last_facing = d;
        d
    } else {
        Vec2::ZERO
    };

    connection
        .outgoing_messages
        // The guest's camera is mirrored 180° on the XZ plane vs the host's, so the
        // guest's screen-space input maps to the OPPOSITE world direction. Negate
        // both movement axes AND facing so the host receives correct world-space
        // directions — the avatar then moves and aims toward the enemy, not the
        // guest's own castle.
        .push(NetworkMessage::SwordcererAvatarInput {
            dx: -dir.x,
            dz: -dir.y,
            fire_missile: fire,
            swing_sword: swing,
            facing_x: -last_facing.x,
            facing_z: -last_facing.y,
        });

    // Draw the sword arc LOCALLY on the guest the instant it swings. The host's
    // authoritative swing (damage) still happens, but the host's arc only reaches
    // the guest through the UNRELIABLE cast-event snapshot, so a brief swing was
    // frequently missed. This is a cosmetic ghost arc (no damage) at the guest's
    // ghost-avatar position, facing the same world direction the host will use.
    if swing
        && *last_facing != Vec2::ZERO
        && let Ok(ghost_tf) = ghost_avatar.single()
    {
        let arc_pos = Vec3::new(ghost_tf.translation.x, 2.0, ghost_tf.translation.z);
        super::combat::spawn_sword_arc(
            &mut commands,
            &mut meshes,
            &mut materials,
            arc_pos,
            -*last_facing,
            true,
        );
    }
}

/// Host: applies the guest's streamed input to its avatar (movement + missile +
/// sword swing). Replaces `player_movement` / `fire_missile` / `sword_swing` for
/// the guest-controlled avatar (those drive the host's OWN avatar).
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_guest_avatar_input(
    time: Res<Time>,
    connection: Option<ResMut<NetworkConnection>>,
    mut commands: Commands,
    mut avatar_q: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &mut SwordcererFacing,
            &MovementSpeed,
            &Team,
            Option<&SwordcererMissileCooldown>,
            Option<&SwordcererSwordCooldown>,
        ),
        With<SwordcererAvatar>,
    >,
    targets: Query<(Entity, &Transform, &Team), (Without<SwordcererAvatar>, Without<Corpse>)>,
    visual_assets: Res<SpellVisualAssets>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
    swordcerer_assets: Res<SwordcererAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // The last movement direction received from the guest. Input arrives at the
    // network rate (well below 60 Hz), so we hold the last direction and apply
    // physics EVERY frame — otherwise the avatar only accelerates on the frames a
    // packet lands and crawls compared to the host's own avatar.
    mut last_move_dir: Local<Vec2>,
) {
    let Some(mut connection) = connection else {
        return;
    };

    // Combine all input messages this frame: latest movement/facing, OR'd fires.
    let mut move_dir = Vec2::ZERO;
    let mut facing = Vec2::ZERO;
    let mut fire = false;
    let mut swing = false;
    let mut got_input = false;
    if !connection.incoming_messages.is_empty() {
        let messages: Vec<NetworkMessage> = std::mem::take(&mut connection.incoming_messages);
        let mut unhandled = Vec::new();
        for msg in messages {
            match msg {
                NetworkMessage::SwordcererAvatarInput {
                    dx,
                    dz,
                    fire_missile,
                    swing_sword,
                    facing_x,
                    facing_z,
                } => {
                    move_dir = Vec2::new(dx, dz);
                    facing = Vec2::new(facing_x, facing_z);
                    fire |= fire_missile;
                    swing |= swing_sword;
                    got_input = true;
                }
                other => unhandled.push(other),
            }
        }
        if !unhandled.is_empty() {
            connection.incoming_messages.extend(unhandled);
        }
    }
    // Remember the latest movement direction; a released key arrives as (0,0), so
    // when no packet lands this frame we keep applying the last direction (and
    // damping) every frame — matching the host avatar's per-frame physics.
    if got_input {
        *last_move_dir = move_dir;
    }

    let dt = time.delta_secs();
    for (entity, mut transform, mut velocity, mut facing_c, speed, team, missile_cd, sword_cd) in
        &mut avatar_q
    {
        // Only the guest's avatar (Attackers); the host's own avatar (Defenders)
        // is driven by the normal input systems.
        if *team != Team::Attackers {
            continue;
        }

        if facing != Vec2::ZERO {
            facing_c.0 = facing;
        }

        // Movement — shared with the host's own avatar (`player_movement`). Use the
        // last-known direction so physics runs every frame, not just on packet frames.
        super::combat::apply_avatar_physics(
            &mut transform,
            &mut velocity,
            *last_move_dir,
            speed.0,
            dt,
        );

        let avatar_pos = transform.translation;
        let dir3 = Vec3::new(facing_c.0.x, 0.0, facing_c.0.y).normalize_or_zero();

        // Fire missile — shared with the host's own avatar. Targets are derived
        // from the avatar's team (Attackers ⇒ Defenders/Undead). No mana cost:
        // the guest's mana lives on the guest.
        if fire && !missile_cd.is_some_and(|cd| cd.remaining > 0.0) {
            super::combat::spawn_avatar_missile(
                &mut commands,
                &visual_assets,
                &sfx,
                &config,
                &swordcerer_assets,
                &targets,
                entity,
                avatar_pos,
                dir3,
                *team,
            );
        }

        // Sword swing.
        if swing && !sword_cd.is_some_and(|cd| cd.remaining > 0.0) {
            let arc_pos = Vec3::new(avatar_pos.x, 2.0, avatar_pos.z);
            super::combat::spawn_sword_arc(
                &mut commands,
                &mut meshes,
                &mut materials,
                arc_pos,
                facing_c.0,
                false,
            );
            // No SwordArc cast-event emit here: the guest draws its OWN arc locally
            // and immediately (see `send_swordcerer_avatar_input`), so replicating
            // the host's arc would produce a latency-offset double slash.
            velocity.x += facing_c.0.x * SWORD_LUNGE_SPEED;
            velocity.z += facing_c.0.y * SWORD_LUNGE_SPEED;
            commands.entity(entity).insert(
                crate::game::units::components::CombatAnimation::new_attack(
                    swordcerer_assets.attacking_texture.clone(),
                    swordcerer_assets.sprite_texture.clone(),
                ),
            );
            commands.entity(entity).insert(SwordcererSwordCooldown {
                remaining: SWORD_COOLDOWN,
            });
        }
    }
}

/// Host: when the guest's avatar dies, despawn it and notify the guest so it
/// reappears as a caster.
pub(crate) fn check_guest_avatar_death(
    mut commands: Commands,
    connection: Option<ResMut<NetworkConnection>>,
    avatar_q: Query<(Entity, &Health, &Team), With<SwordcererAvatar>>,
) {
    let Some(mut connection) = connection else {
        return;
    };
    for (entity, health, team) in &avatar_q {
        if *team == Team::Attackers && health.current <= 0.0 {
            commands.entity(entity).try_despawn();
            connection
                .outgoing_messages
                .push(NetworkMessage::SwordcererAvatarDied);
        }
    }
}

/// Guest: the host reports the avatar died — trigger the local retreat (restore
/// the wizard, reset state) via the shared retreat path.
pub(crate) fn receive_swordcerer_death(
    connection: Option<ResMut<NetworkConnection>>,
    mut retreat: MessageWriter<RetreatMessage>,
) {
    let Some(mut connection) = connection else {
        return;
    };
    if connection.incoming_messages.is_empty() {
        return;
    }
    let messages: Vec<NetworkMessage> = std::mem::take(&mut connection.incoming_messages);
    let mut unhandled = Vec::new();
    for msg in messages {
        match msg {
            NetworkMessage::SwordcererAvatarDied => {
                retreat.write(RetreatMessage);
            }
            other => unhandled.push(other),
        }
    }
    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}
