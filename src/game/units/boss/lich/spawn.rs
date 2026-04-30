//! Lich spawn, approach, summoning.

use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::LichAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker, WaveGroup};
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::ogre::MeleeDamageReduction;
use crate::game::units::components::{
    AttackTiming, Corpse, DamageMultiplier, Effectiveness, FacingDirection, FlockingModifier,
    FlockingVelocity, Health, Hitbox, MovementSpeed, RoughTerrainModifier, TargetingVelocity, Team,
    Teleportable, WalkingAnimation,
};
use crate::game::units::infantry::components::Infantry;
use crate::game::units::infantry::constants::UNDEAD_SPRITE_TINT;
use crate::game::units::random_position_in_cell;
use crate::game::units::systems::create_default_sprite_material;
use crate::game::units::undead::resources::UndeadAssets;

/// Minimum elapsed time (seconds) after `waves_complete` becomes true before
/// the Lich is allowed to spawn. The wave-spawning system sets `waves_complete`
/// in the same frame the final wave dispatches via `Commands::spawn`, which is
/// deferred — so a query for living attackers in that frame can falsely return
/// empty before the new attackers materialize. The debounce ensures the just-
/// dispatched wave is in the world before we check for survivors.
const LICH_POST_WAVE_SPAWN_DELAY: f32 = 0.5;

/// Checks if it's time to spawn the Lich mid-game.
/// The Lich spawns as an extra wave after all normal waves have been dispatched
/// and every attacker (including staging) is dead.
#[allow(clippy::too_many_arguments)]
pub(super) fn check_lich_spawn(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    lich_assets: Res<LichAssets>,
    pending: Option<Res<LichSpawnPending>>,
    wave_state: Option<Res<crate::game::resources::WaveState>>,
    existing: Query<(), With<Lich>>,
    all_attackers: Query<&Team, Without<Corpse>>,
    mut time_since_waves_complete: Local<f32>,
) {
    let Some(_pending) = pending else { return };
    if !existing.is_empty() {
        return;
    };
    let Some(wave_state) = wave_state else { return };

    // Track how long it's been since the final wave dispatched. Reset while
    // waves are still ongoing so a fresh debounce runs on the actual final.
    if wave_state.waves_complete {
        *time_since_waves_complete += time.delta_secs();
    } else {
        *time_since_waves_complete = 0.0;
        return;
    }

    if *time_since_waves_complete < LICH_POST_WAVE_SPAWN_DELAY {
        return;
    }

    // Wait for every attacker to die (staging or activated)
    let has_living_attackers = all_attackers.iter().any(|t| *t == Team::Attackers);
    if has_living_attackers {
        return;
    }

    spawn_lich(
        &mut game_rng.0,
        &mut commands,
        &lich_assets,
        wave_state.current_wave,
    );
    commands.remove_resource::<LichSpawnPending>();
}

/// Spawns the Lich at one of the tunnel spawn points.
fn spawn_lich(
    rng: &mut impl Rng,
    commands: &mut Commands,
    lich_assets: &LichAssets,
    current_wave: u32,
) {
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(LICH_RADIUS, LICH_HITBOX_HEIGHT);
    // Sprite bottom at ground level; the float system layers hover on top.
    let spawn_y = LICH_SPRITE_HEIGHT / 2.0;

    commands
        .spawn((
            Mesh3d(lich_assets.mesh.clone()),
            MeshMaterial3d(lich_assets.floating_material.clone()),
            Transform::from_xyz(final_x, spawn_y, final_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(LICH_HEALTH),
            MovementSpeed(LICH_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Undead,
            Boss,
            Lich,
        ))
        .insert((
            LichPhase::Approaching,
            SoulPower::new(SOUL_POWER_MAX),
            LichSummonTimer::new(SUMMON_INTERVAL),
            MeleeDamageReduction {
                multiplier: LICH_MELEE_DAMAGE_REDUCTION,
            },
            StagingAttacker(CENTER_STAGING_INDEX as u8),
            WaveGroup(current_wave),
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            DamageMultiplier(LICH_DAMAGE_MULTIPLIER),
        ))
        .insert((
            FlockingVelocity::default(),
            FlockingModifier::new(0.0, 0.0, 0.0),
            RoughTerrainModifier(0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
            WalkingAnimation {
                columns: LICH_SHEET_COLUMNS,
                frame_uv: LICH_FRAME_UV,
                direction_rows: LICH_DIRECTION_ROWS,
                ..Default::default()
            },
            FacingDirection::Forward,
            LichFloatBase { base_y: spawn_y },
        ));
}

/// Detects when the normal staging system has activated the Lich
/// (removed StagingAttacker) and transitions to summoning phase.
pub(super) fn lich_approach_system(
    mut query: Query<
        (&mut LichPhase, &mut Velocity, Has<StagingAttacker>),
        (With<Lich>, Without<Corpse>),
    >,
) {
    for (mut phase, mut velocity, has_staging) in &mut query {
        if *phase != LichPhase::Approaching {
            continue;
        }

        // StagingAttacker was removed by the normal staging system —
        // that means the Lich reached the staging zone, defenders are
        // activated, and the battle timer has started.
        if !has_staging {
            *phase = LichPhase::Summoning;
            velocity.x = 0.0;
            velocity.z = 0.0;
        }
    }
}

/// Phase 1: Ticks the summon timer and starts a Raise Dead cast wind-up when
/// it expires. The actual undead spawn is deferred to `tick_lich_casting`,
/// which runs after `LICH_RAISE_DEAD_CAST_DURATION` so the casting sprite has
/// time to play.
pub(super) fn lich_summoning_system(
    time: Res<Time>,
    mut commands: Commands,
    mut lich_query: Query<
        (Entity, &mut LichSummonTimer, &LichPhase),
        (With<Lich>, Without<Corpse>, Without<LichCasting>),
    >,
) {
    for (entity, mut timer, phase) in &mut lich_query {
        if *phase != LichPhase::Summoning {
            continue;
        }

        timer.tick(time.delta_secs());
        if !timer.is_ready() {
            continue;
        }

        timer.reset(SUMMON_INTERVAL);
        commands.entity(entity).insert(LichCasting {
            remaining: LICH_RAISE_DEAD_CAST_DURATION,
            kind: LichCastKind::RaiseDead,
        });
    }
}

/// Performs the actual Raise Dead resolution: raises nearby corpses and tops
/// the wave off with freshly summoned undead around the Lich. Called from
/// `tick_lich_casting` once the cast wind-up finishes.
pub(super) fn resolve_raise_dead(
    commands: &mut Commands,
    lich_pos: Vec3,
    corpse_query: &Query<
        (Entity, &Transform),
        (
            With<Corpse>,
            Without<crate::game::units::components::PermanentCorpse>,
            Without<Lich>,
        ),
    >,
    undead_assets: &UndeadAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    let target = SUMMON_WAVE_SIZE as usize;
    let mut raised = 0usize;

    let corpses: Vec<(Entity, Vec3)> = corpse_query
        .iter()
        .map(|(e, t)| (e, t.translation))
        .take(target)
        .collect();

    for (corpse_entity, position) in corpses {
        crate::game::units::systems::resurrect_corpse_as_infantry(
            commands,
            corpse_entity,
            position,
            Team::Undead,
            SUMMONED_UNDEAD_HEALTH,
            SUMMONED_UNDEAD_SPEED,
            UNDEAD_SPRITE_TINT,
            undead_assets.sprite_texture.clone(),
            undead_assets.sprite_mesh.clone(),
            materials,
            Some(undead_assets.death_texture.clone()),
        );
        raised += 1;
    }

    let remaining = target.saturating_sub(raised);
    for i in 0..remaining {
        let angle = (i as f32 / remaining as f32) * std::f32::consts::TAU;
        let spawn_x = lich_pos.x + SUMMON_SPAWN_RADIUS * angle.cos();
        let spawn_z = lich_pos.z + SUMMON_SPAWN_RADIUS * angle.sin();
        spawn_fresh_undead(commands, undead_assets, materials, spawn_x, spawn_z);
    }
}

/// Spawns a single fresh undead infantry unit at the given position.
fn spawn_fresh_undead(
    commands: &mut Commands,
    undead_assets: &UndeadAssets,
    materials: &mut Assets<StandardMaterial>,
    x: f32,
    z: f32,
) {
    use crate::game::units::infantry::constants::UNIT_RADIUS;

    let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let material = create_default_sprite_material(
        materials,
        undead_assets.sprite_texture.clone(),
        UNDEAD_SPRITE_TINT,
    );

    commands
        .spawn((
            Mesh3d(undead_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(x, spawn_y, z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(SUMMONED_UNDEAD_HEALTH),
            MovementSpeed(SUMMONED_UNDEAD_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Undead,
            Infantry,
        ))
        .insert((
            crate::game::units::components::WalkingAnimation::default(),
            crate::game::units::components::FacingDirection::default(),
            TargetingVelocity::default(),
            FlockingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ));
}
