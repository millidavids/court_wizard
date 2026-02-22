use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::super::super::components::{CastingState, GuestWizard, LocalWizard, Mana, PrimedSpell, Spell, WizardInput};
use super::components::*;
use super::constants::{
    CHANNEL_RAMP_TIME, INITIAL_CHANNEL_INTERVAL, MANA_COST_PER_CORPSE, MIN_CHANNEL_INTERVAL,
    RESURRECTION_RADIUS,
};
use crate::game::components::{Acceleration, Billboard, Velocity};
use crate::game::constants::{DEFENDER_HITBOX_HEIGHT, UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::spell_commands::{GuestCursorPosition, GuestInputState};
use crate::game::units::archer::Archer;
use crate::game::units::archer::resources::ArcherAssets;
use crate::game::units::components::{
    AttackTiming, Corpse, Effectiveness, Health, Hitbox, MovementSpeed, PermanentCorpse,
    RoughTerrain, Team, Teleportable,
};
use crate::game::units::infantry::components::Infantry;
use crate::game::units::infantry::resources::InfantryAssets;

/// Unit radius for infantry hitboxes (matches infantry/styles.rs::UNIT_RADIUS)
const UNIT_RADIUS: f32 = 8.0;

/// Local wizard Raise The Dead casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_raise_the_dead_casting(
    time: Res<Time>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut wizard_query: Query<
        (&mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    corpse_query: Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&Infantry>,
            Option<&Archer>,
        ),
        (With<Corpse>, Without<PermanentCorpse>),
    >,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
) {
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &window_query);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::RaiseTheDead { return; }

    raise_the_dead_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &corpse_query,
        &infantry_assets,
        &archer_assets,
    );
}

/// Guest wizard Raise The Dead casting — reads network signals.
#[allow(clippy::too_many_arguments)]
pub fn handle_raise_the_dead_casting_guest(
    time: Res<Time>,
    mut commands: Commands,
    mut wizard_query: Query<
        (&mut CastingState, &mut Mana, &PrimedSpell),
        With<GuestWizard>,
    >,
    corpse_query: Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&Infantry>,
            Option<&Archer>,
        ),
        (With<Corpse>, Without<PermanentCorpse>),
    >,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    guest_cursor: Res<GuestCursorPosition>,
    guest_input: Res<GuestInputState>,
) {
    let input = WizardInput {
        just_pressed: guest_input.just_pressed,
        pressed: guest_input.pressed,
        just_released: guest_input.just_released,
        cursor_pos: guest_cursor.position,
    };

    let Ok((mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut() else {
        return;
    };
    if primed_spell.spell != Spell::RaiseTheDead { return; }

    raise_the_dead_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &corpse_query,
        &infantry_assets,
        &archer_assets,
    );
}

/// Core Raise The Dead casting logic — called by both local and guest systems.
///
/// Handles the full Resting -> Casting -> Channeling state machine.
/// During channeling, resurrects corpses at increasing frequency.
#[allow(clippy::too_many_arguments)]
fn raise_the_dead_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    corpse_query: &Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&Infantry>,
            Option<&Archer>,
        ),
        (With<Corpse>, Without<PermanentCorpse>),
    >,
    infantry_assets: &Res<InfantryAssets>,
    archer_assets: &Res<ArcherAssets>,
) {
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return;
    }

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.advance_channel(time.delta_secs());

            if casting_state.should_channel(
                INITIAL_CHANNEL_INTERVAL,
                MIN_CHANNEL_INTERVAL,
                CHANNEL_RAMP_TIME,
            ) {
                if mana.consume(MANA_COST_PER_CORPSE) {
                    if let Some(cursor_pos) = input.cursor_pos {
                        resurrect_nearest_corpse(
                            commands,
                            cursor_pos,
                            corpse_query,
                            infantry_assets,
                            archer_assets,
                            primed_spell.empowerment,
                        );
                        casting_state.reset_channel_interval();
                    }
                } else {
                    casting_state.cancel();
                }
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(MANA_COST_PER_CORPSE) {
                    if let Some(cursor_pos) = input.cursor_pos {
                        resurrect_nearest_corpse(
                            commands,
                            cursor_pos,
                            corpse_query,
                            infantry_assets,
                            archer_assets,
                            primed_spell.empowerment,
                        );
                        casting_state.start_channeling();
                    }
                } else {
                    casting_state.cancel();
                }
            }
        }
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(MANA_COST_PER_CORPSE) {
                casting_state.start_cast();
            }
        }
    }
}

/// Resurrects the nearest corpse to the target position as infantry.
///
/// Searches for corpses within RESURRECTION_RADIUS and resurrects the closest one.
/// All raised undead are infantry units.
fn resurrect_nearest_corpse(
    commands: &mut Commands,
    target_pos: Vec3,
    corpse_query: &Query<
        (
            Entity,
            &Transform,
            &Team,
            Option<&Infantry>,
            Option<&Archer>,
        ),
        (With<Corpse>, Without<PermanentCorpse>),
    >,
    infantry_assets: &Res<InfantryAssets>,
    archer_assets: &Res<ArcherAssets>,
    empowerment: f32,
) {
    // Find nearest corpse within radius
    if let Some((corpse_entity, corpse_transform, _, is_infantry, is_archer)) = corpse_query
        .iter()
        .filter(|(_, transform, _, _, _)| {
            target_pos.distance(transform.translation) <= RESURRECTION_RADIUS
        })
        .min_by(|a, b| {
            let dist_a = target_pos.distance(a.1.translation);
            let dist_b = target_pos.distance(b.1.translation);
            dist_a.partial_cmp(&dist_b).unwrap()
        })
    {
        // Replace with undead material
        let undead_material = if is_infantry.is_some() {
            infantry_assets.undead_material.clone()
        } else if is_archer.is_some() {
            archer_assets.undead_material.clone()
        } else {
            // Fallback to infantry material
            infantry_assets.undead_material.clone()
        };

        // Calculate upright position: bottom edge 1 unit above battlefield
        let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
        let spawn_y = hitbox.height / 2.0 + 1.0;
        let upright_transform = Transform::from_xyz(
            corpse_transform.translation.x,
            spawn_y,
            corpse_transform.translation.z,
        );

        // Apply empowerment scaling (1.25x bonus to stats if empowered)
        let scale = empowerment;
        let health = UNIT_HEALTH * scale;
        let speed = UNIT_MOVEMENT_SPEED * 0.5 * scale; // Base is half speed, then apply scaling

        // Create effectiveness with spell bonus for damage scaling if empowered
        let mut effectiveness = Effectiveness::new();
        if empowerment > 1.0 {
            effectiveness.spell_bonus = 0.25; // +25% damage bonus
        }

        // Restore combat components but change team
        commands
            .entity(corpse_entity)
            .remove::<Corpse>()
            .remove::<RoughTerrain>()
            .insert(upright_transform) // Stand upright
            .insert(MeshMaterial3d(undead_material)) // Replace with undead material
            .insert(Team::Undead)
            .insert(Health::new(health)) // Full health restoration with empowerment scaling
            .insert(Velocity::default())
            .insert(Acceleration::new())
            .insert(MovementSpeed(speed)) // Half speed with empowerment scaling
            .insert(AttackTiming::new())
            .insert(effectiveness) // Effectiveness with empowerment bonus if applicable
            .insert(Billboard)
            .insert(hitbox) // Restore collision
            .insert(Teleportable) // Can be teleported
            .insert(RaisedUndead) // Marker for tracking
            .insert(Infantry)
            .insert(crate::game::units::components::TargetingVelocity::default())
            .insert(crate::game::units::components::FlockingVelocity::default());
    }
}

/// Gets cursor position projected onto Y=0 plane.
fn get_cursor_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: &Query<&Window, With<PrimaryWindow>>,
) -> Option<Vec3> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;
    let cursor_pos = window.cursor_position()?;

    let ray = camera
        .viewport_to_world(camera_transform, cursor_pos)
        .ok()?;
    let t = -ray.origin.y / ray.direction.y;

    if t > 0.0 {
        Some(ray.origin + ray.direction * t)
    } else {
        None
    }
}
