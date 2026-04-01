use bevy::audio::Volume;
use bevy::prelude::*;

use crate::config::GameConfig;

use super::cauldron::resources::CauldronBuffs;
use super::constants::*;
use super::plugin::GlobalAttackCycle;
use super::resources::CurrentLevel;
use super::units::boss::components::Boss;
use super::units::components::{
    Corpse, Effectiveness, Flying, HasShadow, Hitbox, InMelee, SpellDamaged, Team, UnitShadow,
};
use super::units::king::components::KingSpawned;

/// Advances the global attack cycle timer each game frame.
///
/// This timer cycles from 0.0 to cycle_duration seconds, creating a rotating
/// schedule for unit attacks that is consistent across different frame rates.
pub fn tick_attack_cycle(time: Res<Time>, mut attack_cycle: ResMut<GlobalAttackCycle>) {
    attack_cycle.tick(time.delta_secs());
}

/// Ticks the elapsed game time using real (wall-clock) time.
/// Uses `Time<Real>` so the displayed clock is not affected by the between-wave
/// staging speedup or any other virtual-time scaling.
pub fn tick_elapsed_time(
    time: Res<Time<Real>>,
    mut kill_stats: ResMut<super::resources::KillStats>,
) {
    kill_stats.elapsed_time += time.delta_secs();
}

/// Initializes the current level from saved config.
///
/// This system runs on OnEnter(AppState::InGame) to restore the player's
/// current level from their last session.
pub fn init_level_from_config(mut current_level: ResMut<CurrentLevel>, config: Res<GameConfig>) {
    current_level.0 = config.current_level;
}

/// Calculates effectiveness for all units based on melee proximity.
///
/// Effectiveness is modified by:
/// - Number of allies in melee range (positive effect: +10% per ally)
/// - Number of enemies in melee range (negative effect: -15% per enemy)
///
/// The effectiveness coefficient is applied to both movement speed and attack damage
/// in their respective systems. This encourages tactical positioning and rewards
/// units that fight together while penalizing isolated units.
pub fn calculate_effectiveness(
    mut units: Query<
        (Entity, &Transform, &Hitbox, &Team, &mut Effectiveness),
        (Without<Corpse>, Without<Boss>),
    >,
) {
    // Collect snapshot for symmetric calculations
    let unit_data: Vec<_> = units
        .iter()
        .map(|(entity, transform, hitbox, team, _)| (entity, transform.translation, *hitbox, *team))
        .collect();

    // Calculate effectiveness for each unit
    for (entity, transform, hitbox, team, mut effectiveness) in units.iter_mut() {
        let mut ally_count = 0;
        let mut enemy_count = 0;

        for (other_entity, other_pos, other_hitbox, other_team) in &unit_data {
            if *other_entity == entity {
                continue;
            }

            // Calculate XZ plane distance
            let dx = transform.translation.x - other_pos.x;
            let dz = transform.translation.z - other_pos.z;
            let distance = (dx * dx + dz * dz).sqrt();

            // Use same melee range formula as combat
            let melee_range = (hitbox.radius + other_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;

            if distance <= melee_range {
                let is_enemy = team.is_enemy(other_team);

                if is_enemy {
                    enemy_count += 1;
                } else {
                    ally_count += 1;
                }
            }
        }

        effectiveness.recalculate(ally_count, enemy_count);
    }
}

/// Stops all in-game sound effects by despawning audio entities.
/// Runs when transitioning to the score screen so spell sounds don't persist.
pub fn stop_all_sfx(
    mut commands: Commands,
    audio_query: Query<Entity, (With<AudioPlayer>, With<super::components::OnGameplayScreen>)>,
) {
    for entity in &audio_query {
        commands.entity(entity).try_despawn();
    }
}

/// Cleans up all game entities when exiting the InGame state.
pub fn cleanup_game(
    mut commands: Commands,
    query: Query<Entity, With<super::components::OnGameplayScreen>>,
) {
    // Don't reset level - it persists between sessions via config
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}

/// Cleans up game entities when replaying (transitioning from ScoreScreen).
///
/// This system runs on OnExit(InGameState::ScoreScreen) and despawns all game entities
/// in preparation for re-spawning them fresh.
pub fn cleanup_for_replay(
    mut commands: Commands,
    gameplay_entities: Query<Entity, With<super::components::OnGameplayScreen>>,
) {
    for entity in &gameplay_entities {
        commands.entity(entity).try_despawn();
    }
}

/// Resets game resources when replaying (transitioning from ScoreScreen).
///
/// This system runs on OnExit(InGameState::ScoreScreen) and resets resources like
/// the attack cycle timer and defender activation status.
pub fn reset_resources_for_replay(
    mut attack_cycle: ResMut<super::plugin::GlobalAttackCycle>,
    mut defenders_activated: ResMut<super::units::infantry::components::DefendersActivated>,
    mut king_spawned: ResMut<KingSpawned>,
    mut cauldron_buffs: ResMut<CauldronBuffs>,
    mut battle_insight: ResMut<super::resources::BattleInsightData>,
    mut stone_used: ResMut<super::cauldron::resources::PhilosophersStoneUsed>,
    mut retreat_state: ResMut<super::units::infantry::components::RetreatState>,
) {
    attack_cycle.current_time = 0.0;
    defenders_activated.active = false;
    king_spawned.0 = false;
    cauldron_buffs.reset();
    stone_used.0 = false;
    *battle_insight = Default::default();
    *retreat_state = Default::default();
    // WaveState is NOT reset here — it's freshly set by init_loading_progress
    // during each level load, so resetting here would overwrite the correct values.
}

/// Activates all defenders when any defender is close enough to an enemy.
///
/// This creates coordinated defensive behavior - the entire defensive line
/// engages together rather than individually.
pub fn activate_defenders_on_proximity(
    mut defenders_activated: ResMut<super::units::infantry::components::DefendersActivated>,
    retreat_state: Res<super::units::infantry::components::RetreatState>,
    defenders: Query<(&Transform, &Team), Without<Corpse>>,
    all_units: Query<(&Transform, &Team), Without<Corpse>>,
) {
    const ENGAGEMENT_RANGE: f32 = 800.0; // Archer max range (700) + 100

    // During retreat, defenders are force-deactivated — skip activation
    if retreat_state.is_active() {
        return;
    }

    // If already active, stay active (defenders don't deactivate once engaged)
    if defenders_activated.active {
        return;
    }

    // Check if any defender is close to any enemy
    for (defender_transform, defender_team) in &defenders {
        // Skip non-defenders
        if *defender_team != Team::Defenders {
            continue;
        }

        // Check distance to nearest enemy
        for (enemy_transform, enemy_team) in &all_units {
            // Skip same team
            if *enemy_team == *defender_team {
                continue;
            }

            let is_enemy = defender_team.is_enemy(enemy_team);

            if !is_enemy {
                continue;
            }

            let distance = defender_transform
                .translation
                .distance(enemy_transform.translation);
            if distance < ENGAGEMENT_RANGE {
                // Enemy in range - activate all defenders
                defenders_activated.active = true;
                return;
            }
        }
    }
}

/// Tracks when any wizard spell damages an attacker or undead unit.
/// Uses `Added<SpellDamaged>` to detect newly damaged entities this frame.
/// Gated by `run_if` so it stops running once flagged.
pub fn track_wizard_enemy_damage(
    query: Query<&Team, Added<SpellDamaged>>,
    mut kill_stats: ResMut<super::resources::KillStats>,
) {
    for team in &query {
        if *team == Team::Attackers || *team == Team::Undead {
            kill_stats.wizard_damaged_enemies = true;
            return;
        }
    }
}

/// Run condition: returns true when no wizard spell has damaged enemies yet this battle.
pub fn wizard_has_not_damaged_enemies(kill_stats: Res<super::resources::KillStats>) -> bool {
    !kill_stats.wizard_damaged_enemies
}

// ===== Battle Ambience =====

/// Number of units in melee at which the battle sound reaches full intensity.
const BATTLE_AMBIENCE_MAX_UNITS: f32 = 40.0;

/// Overall volume scale for battle ambience (kept subtle so it doesn't overpower spells/music).
const BATTLE_AMBIENCE_VOLUME_SCALE: f32 = 0.15;

/// Maximum distance for battle ambience attenuation (same as spell SFX).
const BATTLE_AMBIENCE_MAX_DISTANCE: f32 = 10000.0;

/// Overall volume scale for the crowd ambience loop.
const CROWD_AMBIENCE_VOLUME_SCALE: f32 = 0.12;

/// Position of the battlefield center for crowd sound attenuation (XZ from staging point).
const BATTLEFIELD_CENTER: Vec3 = Vec3::new(STAGING_POINT.0, 0.0, STAGING_POINT.1);

/// Pre-loaded battle ambience audio.
#[derive(Resource)]
pub struct BattleAmbienceAssets {
    pub battle_audio: Handle<AudioSource>,
    pub crowd_audio: Handle<AudioSource>,
}

/// Marker component for the melee battle ambience audio entity.
#[derive(Component)]
pub(crate) struct BattleAmbienceEntity;

/// Marker component for the crowd ambience audio entity.
#[derive(Component)]
pub(crate) struct CrowdAmbienceEntity;

/// Loads the battle ambience audio assets at startup.
pub fn load_battle_ambience_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(BattleAmbienceAssets {
        battle_audio: asset_server.load("audio/sound_effects/battle.ogg"),
        crowd_audio: asset_server.load("audio/sound_effects/angry_crowd.ogg"),
    });
}

/// Scales the looping battle ambience volume based on how many units are in melee combat.
/// Uses distance attenuation from the average melee position to the wizard, so battles
/// closer to the castle sound louder than distant skirmishes.
pub fn update_battle_ambience(
    mut commands: Commands,
    melee_query: Query<(&InMelee, &Transform)>,
    mut ambience_query: Query<(Entity, Option<&mut AudioSink>), With<BattleAmbienceEntity>>,
    ambience_assets: Res<BattleAmbienceAssets>,
    game_config: Res<GameConfig>,
) {
    let melee_count = melee_query.iter().count() as f32;
    let sfx_volume = game_config.effective_sfx_volume();

    if melee_count > 0.0 && sfx_volume > 0.0 {
        // Compute average position of all melee units for distance attenuation
        let avg_pos = melee_query
            .iter()
            .fold(Vec3::ZERO, |acc, (_, t)| acc + t.translation)
            / melee_count;
        let distance = avg_pos.distance(SPELL_ORIGIN);
        let linear = (1.0 - distance / BATTLE_AMBIENCE_MAX_DISTANCE).clamp(0.0, 1.0);
        let attenuation = linear * linear * linear;

        let intensity = (melee_count / BATTLE_AMBIENCE_MAX_UNITS).clamp(0.0, 1.0);
        let volume = sfx_volume * intensity * attenuation * BATTLE_AMBIENCE_VOLUME_SCALE;

        if volume <= 0.0 {
            if let Ok((entity, _)) = ambience_query.single() {
                commands.entity(entity).try_despawn();
            }
            return;
        }

        if let Ok((_entity, sink)) = ambience_query.single_mut() {
            if let Some(mut sink) = sink {
                sink.set_volume(Volume::Linear(volume));
            }
        } else {
            commands.spawn((
                AudioPlayer::new(ambience_assets.battle_audio.clone()),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
                BattleAmbienceEntity,
                super::components::OnGameplayScreen,
            ));
        }
    } else if let Ok((entity, _)) = ambience_query.single() {
        commands.entity(entity).try_despawn();
    }
}

/// Plays a muffled crowd loop throughout the battle, attenuated from the battlefield center.
/// Spawns on first run, despawns when SFX is muted.
pub fn update_crowd_ambience(
    mut commands: Commands,
    mut crowd_query: Query<(Entity, Option<&mut AudioSink>), With<CrowdAmbienceEntity>>,
    ambience_assets: Res<BattleAmbienceAssets>,
    game_config: Res<GameConfig>,
) {
    let sfx_volume = game_config.effective_sfx_volume();

    // Distance from battlefield center to wizard
    let distance = BATTLEFIELD_CENTER.distance(SPELL_ORIGIN);
    let linear = (1.0 - distance / BATTLE_AMBIENCE_MAX_DISTANCE).clamp(0.0, 1.0);
    let attenuation = linear * linear * linear;
    let volume = sfx_volume * attenuation * CROWD_AMBIENCE_VOLUME_SCALE;

    if volume > 0.0 {
        if let Ok((_entity, sink)) = crowd_query.single_mut() {
            if let Some(mut sink) = sink {
                sink.set_volume(Volume::Linear(volume));
            }
        } else {
            commands.spawn((
                AudioPlayer::new(ambience_assets.crowd_audio.clone()),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(volume)),
                CrowdAmbienceEntity,
                super::components::OnGameplayScreen,
            ));
        }
    } else if let Ok((entity, _)) = crowd_query.single() {
        commands.entity(entity).try_despawn();
    }
}

// --- Unit shadow system ---

/// Shared shadow mesh and material for all units.
#[derive(Resource)]
pub struct ShadowAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// Base shadow circle radius (half of sprite width / 2).
const SHADOW_BASE_RADIUS: f32 = super::units::constants::DEFAULT_SPRITE_WIDTH / 4.0;
/// Extra scale factor for flying unit shadows (simulates height spreading the shadow).
const FLYING_SHADOW_HEIGHT_SCALE: f32 = 1.3;
/// Y position of the shadow (just above ground to avoid z-fighting).
const SHADOW_Y: f32 = 1.5;

pub fn preload_shadow_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Circle::new(SHADOW_BASE_RADIUS));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.35),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    commands.insert_resource(ShadowAssets { mesh, material });
}

/// Spawns a static ground shadow at the given XZ position.
/// Used by terrain objects (flora, trees, bushes, boulders).
pub fn spawn_terrain_shadow(
    commands: &mut Commands,
    shadow_assets: &ShadowAssets,
    x: f32,
    z: f32,
    scale: f32,
) {
    commands.spawn((
        Mesh3d(shadow_assets.mesh.clone()),
        MeshMaterial3d(shadow_assets.material.clone()),
        Transform::from_xyz(x, SHADOW_Y, z)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(scale)),
        super::components::OnGameplayScreen,
    ));
}

/// Spawns shadow entities for any unit that doesn't have one yet.
/// Shadow scale is derived from the unit's hitbox radius so bosses (which use
/// large meshes rather than large transform scales) get proportionally larger shadows.
/// Flying units get slightly larger shadows to simulate height spreading.
pub fn spawn_unit_shadows(
    mut commands: Commands,
    shadow_assets: Res<ShadowAssets>,
    units: Query<
        (Entity, &Transform, &Hitbox, Has<Flying>),
        (With<Team>, Without<HasShadow>, Without<Corpse>),
    >,
) {
    /// Default hitbox radius for regular infantry units.
    const BASE_HITBOX_RADIUS: f32 = 8.0 * crate::game::constants::UNIT_SCALE;

    for (entity, transform, hitbox, is_flying) in &units {
        // Scale shadow proportionally to hitbox radius relative to a standard infantry unit.
        // Use the larger of transform scale or hitbox ratio to avoid double-counting
        // (e.g. brute has both large scale AND large hitbox for the same size increase).
        let hitbox_ratio = hitbox.radius / BASE_HITBOX_RADIUS;
        let scale_factor = transform.scale.x.max(transform.scale.z);
        let mut shadow_scale = hitbox_ratio.max(scale_factor);
        if is_flying {
            shadow_scale *= FLYING_SHADOW_HEIGHT_SCALE;
        }
        commands.spawn((
            Mesh3d(shadow_assets.mesh.clone()),
            MeshMaterial3d(shadow_assets.material.clone()),
            Transform::from_xyz(transform.translation.x, SHADOW_Y, transform.translation.z)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(shadow_scale)),
            super::components::OnGameplayScreen,
            UnitShadow { owner: entity },
        ));
        commands.entity(entity).insert(HasShadow);
    }
}

/// Syncs shadow positions to their owning unit's XZ, always at ground level.
/// Despawns shadows whose owner no longer exists or is a corpse.
pub fn update_unit_shadows(
    mut commands: Commands,
    mut shadows: Query<(Entity, &UnitShadow, &mut Transform)>,
    units: Query<(&Transform, Has<Corpse>), (With<Team>, Without<UnitShadow>)>,
) {
    for (shadow_entity, shadow, mut shadow_transform) in &mut shadows {
        if let Ok((unit_transform, is_corpse)) = units.get(shadow.owner) {
            if is_corpse {
                commands.entity(shadow_entity).try_despawn();
                continue;
            }
            shadow_transform.translation.x = unit_transform.translation.x;
            shadow_transform.translation.z = unit_transform.translation.z;
            shadow_transform.translation.y = SHADOW_Y;
        } else {
            commands.entity(shadow_entity).try_despawn();
        }
    }
}
