//! Fireball casting and projectile spawn.

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, WizardInput,
};
use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::DamageType;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    cleanup_spell_caster, handle_spell_release, try_start_cast_with_indicator,
    update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::snapshot::SpellSoundId;
use bevy::prelude::*;

/// Local wizard fireball casting — reads mouse input.
///
/// `audio_ctx` bundles `SpellSfxAssets` + `GameConfig` to keep the system
/// under Bevy's 16-parameter limit after `pending_cast_events` was added for
/// MP cast-event sync.
#[allow(clippy::too_many_arguments)]
pub fn handle_fireball_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wizard_query: Query<
        (Entity, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    target_assist: Res<TargetAssistWorldPos>,
    audio_ctx: (Res<SpellSfxAssets>, Res<GameConfig>),
    active_talents: Option<Res<ActiveTalents>>,
    local_origin: Res<LocalSpellOrigin>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    // Destructure form (matches every other casting handler) — if `audio_ctx`
    // ever gains a third field the compiler will force this binding to be
    // updated, preventing silent unwiring.
    let (sfx, game_config) = &audio_ctx;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Fireball {
        return;
    }

    let completed = fireball_casting_logic(
        &input,
        &time,
        wizard_entity,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &mut commands,
        &visual_assets,
        &mut meshes,
        sfx,
        game_config,
        &active_talents,
        local_origin.0,
        &mut pending_cast_events,
    );

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Fire,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Core fireball casting logic. Returns true if the spell completed.
#[allow(clippy::too_many_arguments)]
fn fireball_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    meshes: &mut Assets<Mesh>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    active_talents: &Option<Res<ActiveTalents>>,
    local_origin: Vec3,
    pending_cast_events: &mut crate::game::multiplayer::spell_sync::PendingCastEvents,
) -> bool {
    let mut completed = false;

    // Check for release event
    if handle_spell_release(input, commands, wizard_entity, casting_state, caster_query) {
        return false;
    }

    // Determine talent-modified cast time
    let talents = active_talents.as_deref();
    let t2 = talents.and_then(|t| t.get_selection(Spell::Fireball, 1));
    let cast_time = match t2 {
        Some(2) => 2.0, // Quick Ignition (tier 2, choice 2)
        _ => primed_spell.cast_time,
    };

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            // Update indicator position to follow cursor
            if let Some(cursor_pos) = input.cursor_pos {
                update_indicator_position(wizard_entity, cursor_pos, caster_query, indicator_query);
            }

            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(cast_time) {
                if mana.consume(constants::MANA_COST)
                    && let Some(target_pos) = input.cursor_pos
                {
                    let spawn_origin =
                        local_origin + Vec3::new(0.0, constants::SPAWN_HEIGHT_OFFSET, 0.0);
                    spawn_fireball_with_talents(
                        commands,
                        assets,
                        spawn_origin,
                        target_pos,
                        primed_spell,
                        active_talents,
                    );
                    audio::play_sfx_synced(
                        commands,
                        pending_cast_events,
                        SpellSoundId::FireballCast,
                        spawn_origin,
                        game_config,
                        sfx,
                    );
                    completed = true;
                }
                cleanup_spell_caster(commands, wizard_entity, caster_query);
                casting_state.cancel();
            }
        }
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                let indicator_pos = input.cursor_pos.unwrap_or(local_origin);
                let indicator_radius = constants::EXPLOSION_RADIUS * primed_spell.empowerment;
                try_start_cast_with_indicator(
                    commands,
                    meshes,
                    assets.fireball_indicator.clone(),
                    wizard_entity,
                    casting_state,
                    mana,
                    constants::MANA_COST,
                    indicator_pos,
                    indicator_radius,
                    caster_query,
                );
            }
        }
    }

    completed
}

/// Spawns a fireball with talent modifications applied.
fn spawn_fireball_with_talents(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    target: Vec3,
    primed_spell: &PrimedSpell,
    active_talents: &Option<Res<ActiveTalents>>,
) {
    let talents = active_talents.as_deref();
    let t1 = talents.and_then(|t| t.get_selection(Spell::Fireball, 0));
    let t2 = talents.and_then(|t| t.get_selection(Spell::Fireball, 1));
    let t3 = talents.and_then(|t| t.get_selection(Spell::Fireball, 2));

    // Tier 1 modifications
    let radius_mult = match t1 {
        Some(0) => 1.5, // Wider Blast
        Some(2) => 0.5, // Focused Blast: half radius
        _ => 1.0,
    };
    let duration_mult = match t1 {
        Some(1) => 1.8, // Lingering Flames: +80% duration
        _ => 1.0,
    };
    let damage_mult = match t1 {
        Some(2) => 2.0, // Focused Blast: double damage
        _ => 1.0,
    };

    // Tier 2 modifications
    let cluster_bomb = t2 == Some(0);
    let napalm = t2 == Some(1);

    // Tier 3 modifications
    let scorched_earth = t3 == Some(1);
    let chain_ignition = t3 == Some(2);
    let is_meteor = t3 == Some(0);
    let radius_mult = if is_meteor {
        radius_mult * 1.3
    } else {
        radius_mult
    };

    // Calculate spawn position and velocity
    let (spawn_origin, velocity) = if is_meteor {
        // Meteor: spawn high above target, drop straight down
        let meteor_origin = Vec3::new(target.x, 800.0, target.z);
        let speed = primed_spell.scale(constants::PROJECTILE_SPEED) * 1.5;
        (meteor_origin, Vec3::new(0.0, -speed, 0.0))
    } else {
        let direction = (target - origin).normalize();
        let speed = primed_spell.scale(constants::PROJECTILE_SPEED);
        (origin, direction * speed)
    };

    let explosion_duration = constants::EXPLOSION_DURATION * duration_mult;
    let damage = primed_spell.scale(constants::DAMAGE_PER_TICK) * damage_mult / duration_mult;
    let explosion_radius = primed_spell.scale(constants::EXPLOSION_RADIUS) * radius_mult;
    let collision_radius = primed_spell.scale(constants::PROJECTILE_COLLISION_RADIUS);
    let visual_radius = primed_spell.scale(constants::FIREBALL_RADIUS);

    // Build fireball with talent flags
    let mut fireball = Fireball::new(
        velocity,
        damage,
        constants::DAMAGE_TYPE,
        explosion_radius,
        collision_radius,
        primed_spell.empowerment,
    );
    fireball.cluster_bomb = cluster_bomb;
    fireball.napalm = napalm;
    fireball.scorched_earth = scorched_earth;
    fireball.chain_ignition = chain_ignition;
    fireball.explosion_duration = explosion_duration;

    let entity = spawn_fireball_visuals(
        commands,
        assets,
        spawn_origin,
        visual_radius,
        OnGameplayScreen,
    );
    commands.entity(entity).insert(fireball);
}

/// Spawns a raw fireball entity with explicit parameters.
///
/// Used by both wizard casting (via `spawn_fireball`) and crystal absorption.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_fireball_entity(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    velocity: Vec3,
    damage: f32,
    damage_type: DamageType,
    explosion_radius: f32,
    collision_radius: f32,
    empowerment: f32,
    visual_radius: f32,
) -> Entity {
    let entity = spawn_fireball_visuals(commands, assets, origin, visual_radius, OnGameplayScreen);
    commands.entity(entity).insert(Fireball::new(
        velocity,
        damage,
        damage_type,
        explosion_radius,
        collision_radius,
        empowerment,
    ));
    entity
}

/// Visual-only fireball entity: the projectile sphere mesh + material +
/// transform + the orbiting fire-glow halo sibling. No `Fireball` sim
/// component — the caller adds that and any additional markers.
///
/// `screen_marker` tags BOTH the parent and the glow sibling with the
/// same cleanup marker (SP: `OnGameplayScreen`; MP ghost:
/// `OnMultiplayerGameScreen`) so neither outlives its lifetime.
pub(crate) fn spawn_fireball_visuals<M: Component + Clone>(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    origin: Vec3,
    visual_radius: f32,
    screen_marker: M,
) -> Entity {
    let entity = commands
        .spawn((
            Mesh3d(assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.fireball_projectile.clone()),
            Transform::from_translation(origin).with_scale(Vec3::splat(visual_radius)),
            screen_marker.clone(),
        ))
        .id();
    vfx::systems::spawn_fire_glow(
        commands,
        assets,
        entity,
        origin,
        visual_radius,
        screen_marker,
    );
    entity
}
