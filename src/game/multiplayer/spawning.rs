//! Multiplayer entity spawn helpers.

use bevy::prelude::*;

use crate::game::battlefield::components::BattlefieldAssets;
use crate::game::components::Billboard;
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, WaveGroup};
use crate::game::units::archer::components::{ArcherMovementTimer, AttackRange};
use crate::game::units::archer::constants::ARCHER_RADIUS;
use crate::game::units::archer::constants::{
    ARCHER_MAX_RANGE, ARCHER_MIN_RANGE, ARCHER_MOVEMENT_SPEED,
};
use crate::game::units::archer::{Archer, ArcherAssets};
use crate::game::units::components::{
    AttackTiming, Effectiveness, FacingDirection, FlockingVelocity, Health, Hitbox, MovementSpeed,
    TargetingVelocity, Team, Teleportable, WalkingAnimation,
};
use crate::game::units::infantry::Infantry;
use crate::game::units::infantry::constants::UNIT_RADIUS;
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::king::components::KingSpawned;
use crate::game::units::random_position_in_cell;
use crate::game::units::wizard::components::*;
use crate::game::units::wizard::constants;
use crate::game::units::wizard::spells::magic_missile_constants;
use crate::networking::resources::PeerRole;

use super::components::OnMultiplayerGameScreen;

/// Returns an `AttackTiming` whose `last_attack_time` is randomised across
/// the cycle. In SP, the staging phase spaces out first contact so units
/// don't all swing on the same frame; MP has no staging, so without this
/// pre-stagger every unit spawned with `last_attack_time = None` would
/// `can_attack` on the very first frame of melee contact — letting 20+
/// units one-shot a defender in a single frame. By seeding the cycle
/// offset, first-contact damage is naturally distributed over ~2s.
fn staggered_attack_timing() -> AttackTiming {
    use rand::Rng;
    let mut rng = rand::rng();
    // `f32::EPSILON..` excludes exactly 0.0 — combined with
    // `can_attack`'s strict `attack_time > last_time`, a recorded slot of
    // 0.0 paired with the cycle's `last_time` also being 0.0 (on the very
    // first frame after game start) would silently block that unit for a
    // full cycle. Vanishingly rare with random_range, but easy to exclude.
    let offset = rng.random_range(f32::EPSILON..crate::game::constants::ATTACK_CYCLE_DURATION);
    let mut timing = AttackTiming::new();
    timing.last_attack_time = Some(offset);
    timing
}

/// Spawns a castle wall plane at the given position and rotation.
///
/// Tagged `OnGameplayScreen` to match the castle that `setup_battlefield`
/// spawns (Castle 1) — both castles share one cleanup marker so any future
/// query that looks them up by marker finds both. `origin_transform` is the
/// same per-client visual mirror passed to `setup_battlefield`.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_castle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    battlefield_assets: &BattlefieldAssets,
    position: Vec3,
    rotation_degrees: f32,
    origin_transform: Transform,
) {
    crate::game::battlefield::systems::spawn_castle_wall(
        commands,
        meshes,
        materials,
        battlefield_assets,
        position,
        rotation_degrees,
        crate::game::components::OnGameplayScreen,
        origin_transform,
    );
}

/// Spawns a wizard at the given position for multiplayer.
///
/// `is_host_wizard` indicates whether this is the host's wizard (castle 1) or
/// the guest's wizard (castle 2). Combined with `role`, this determines which
/// marker components are added:
/// - Host wizard + Host role → `LocalWizard` (host controls this wizard)
/// - Guest wizard + Guest role → `LocalWizard` (guest controls this wizard)
/// - Guest wizard + Host role → `GuestWizard` (host simulates guest's spells)
// `pub(in crate::game)` so the single-player loading queue can spawn the co-op
// guest wizard proxy beside the host (co-op host runs the SP loading path).
#[allow(clippy::too_many_arguments)]
pub(in crate::game) fn spawn_mp_wizard(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    wizard_type: crate::config::WizardType,
    role: PeerRole,
    is_host_wizard: bool,
    coop: bool,
    wizard_assets: &WizardAssets,
) {
    let hitbox = Hitbox::new(constants::HITBOX_RADIUS, constants::HITBOX_HEIGHT);

    // Create a quad mesh matching the sprite aspect ratio
    let quad_mesh = Rectangle::new(
        constants::WIZARD_SPRITE_WIDTH,
        constants::WIZARD_SPRITE_HEIGHT,
    );

    // UV transform for first frame: scale to 1/3 to show only one cell
    let grid_size = constants::WIZARD_SPRITE_GRID_SIZE as f32;
    let frame_scale = 1.0 / grid_size;
    let uv_transform = bevy::math::Affine2::from_scale(Vec2::splat(frame_scale));

    // The co-op GUEST wizard uses a distinct idle sheet so the two players are
    // visually distinguishable; everyone else (host, versus) uses the default.
    let sprite_texture = if coop && !is_host_wizard {
        wizard_assets.guest_sprite_texture.clone()
    } else {
        wizard_assets.sprite_texture.clone()
    };
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(sprite_texture),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        uv_transform,
        ..default()
    });

    // Apply archetype-identity stat bonuses via the shared helper so MP and SP
    // stay in sync (BoringOleMage, Shepherd, …). Insight/progression bonuses are
    // intentionally omitted in MP to keep matches balanced.
    let mut wizard = Wizard::new(constants::DEFAULT_SPELL_RANGE);
    crate::game::units::wizard::systems::apply_archetype_stat_bonuses(&mut wizard, wizard_type);
    // Versus-only spell-range buff (+5% reach). Applies to every archetype. Co-op
    // plays on the single-player battlefield, so it keeps SP range to match
    // single-player balance (and stay consistent with the co-op host's own wizard,
    // which is spawned via the SP `setup_wizard` with no buff). The Arcanorouter
    // recomputes its range each frame in `apply_bonuses_to_wizard_stats`, which
    // applies the same versus-only multiplier, so this spawn-time value is just its
    // first frame.
    if !coop {
        wizard.spell_range *= constants::MP_SPELL_RANGE_MULTIPLIER;
    }

    let mut entity_commands = commands.spawn((
        Mesh3d(meshes.add(quad_mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(position),
        hitbox,
        Health::new(constants::HEALTH),
        MovementSpeed(0.0),
        Mana::new(constants::MANA),
        ManaRegen::new(constants::MANA_REGEN),
        CastingState::new(),
        wizard,
        WizardAnimation::new(),
        Billboard,
        OnMultiplayerGameScreen,
    ));

    // Skip priming the default spell for archetypes that shouldn't start with
    // Magic Missile (mirrors single-player `setup_wizard`): Warglock fires guns,
    // the Randomancer only casts what its roulette rolls, and the Shepherd can't
    // cast offensive spells. Without this guard their clicks would fall through
    // to casting Magic Missile.
    if !matches!(
        wizard_type,
        crate::config::WizardType::Warglock
            | crate::config::WizardType::Randomancer
            | crate::config::WizardType::Shepherd
    ) {
        entity_commands.insert(magic_missile_constants::PRIMED_MAGIC_MISSILE);
    }

    // Wizards are spawned locally on both peers and synced via spell visuals,
    // not snapshots — keep them out of the snapshot stream (the co-op guest
    // wizard proxy carries `Team::Defenders`, which would otherwise be ghosted).
    entity_commands.insert(crate::game::multiplayer::components::NoSnapshot);

    // Add wizard role markers
    // LocalWizard: the wizard this player controls (host's own or guest's own)
    // GuestWizard: the guest's wizard as seen by the host (for spell command processing)
    if (is_host_wizard && role == PeerRole::Host) || (!is_host_wizard && role == PeerRole::Guest) {
        entity_commands.insert(LocalWizard);
    }
    if !is_host_wizard && role == PeerRole::Host {
        entity_commands.insert(GuestWizard);
    }

    if wizard_type == crate::config::WizardType::Arcanorouter {
        entity_commands.insert(
            crate::game::units::wizard::archetypes::arcanorouter::ArcanoRouterBonuses::default(),
        );
    }
}

/// Spawns a single infantry unit for multiplayer.
///
/// `host_side` = true spawns near Castle 1 using standard grid positions.
/// `host_side` = false spawns near Castle 2 using mirrored grid positions.
pub(super) fn spawn_mp_infantry(
    commands: &mut Commands,
    infantry_assets: &InfantryAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    team: Team,
    host_side: bool,
) {
    let total_units = MP_INFANTRY_COUNT;
    let cells_needed = cells_needed(total_units);
    let units_per_cell = distribute_units_to_cells(total_units);

    // Use defender grid layout (units defend near their castle)
    let mut cells = Vec::new();
    let mut cells_added = 0;
    'outer: for row in (0..DEFENDER_GRID_ROWS).rev() {
        for col in 0..DEFENDER_GRID_COLS {
            cells.push((row, col));
            cells_added += 1;
            if cells_added >= cells_needed {
                break 'outer;
            }
        }
    }

    let mut units_counted = 0;
    for (cell_idx, (row, col)) in cells.iter().enumerate() {
        let units_in_this_cell = units_per_cell[cell_idx];
        if unit_index < units_counted + units_in_this_cell {
            let (spawn_x, spawn_z) = if host_side {
                calculate_mp_defender_grid_position(*row, *col)
            } else {
                calculate_mp_guest_defender_grid_position(*row, *col)
            };
            let mut rng = rand::rng();
            let (final_x, final_z) = random_position_in_cell(&mut rng, spawn_x, spawn_z);

            let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;
            let spawn_pos = Vec2::new(spawn_x, spawn_z);

            let tint = crate::game::units::systems::sprite_tint_for_team(team);
            let anim = WalkingAnimation::default();
            let material = crate::game::units::systems::create_default_sprite_material(
                materials,
                infantry_assets.sprite_texture.clone(),
                tint,
            );

            let flow_field = if host_side {
                FlowFieldInfluence::Defender { spawn_pos }
            } else {
                FlowFieldInfluence::Attacker
            };

            let mut ec = commands.spawn((
                Mesh3d(infantry_assets.sprite_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(final_x, spawn_y, final_z),
                crate::game::components::Velocity::default(),
                crate::game::components::Acceleration::new(),
                hitbox,
                Health::new(UNIT_HEALTH),
                MovementSpeed(UNIT_MOVEMENT_SPEED),
                staggered_attack_timing(),
                Effectiveness::new(),
                team,
                Infantry,
            ));
            ec.insert((
                anim,
                FacingDirection::default(),
                TargetingVelocity::default(),
                FlockingVelocity::default(),
                FlowFieldVelocity::default(),
                flow_field,
                Teleportable,
                Billboard,
                OnMultiplayerGameScreen,
            ));
            // MP attackers are pre-activated — no staging phase. `WaveGroup(0)`
            // marks them as already-tagged so `is_staging_attacker` returns
            // false (otherwise the missing `WaveGroup` would have implicitly
            // classed them as staging, and dispeller/wave-speedup logic
            // would treat them as inactive). Predicate on `team` rather than
            // `host_side` so future spawn paths that decouple the two don't
            // silently break the staging guard.
            if team == Team::Attackers {
                ec.insert(WaveGroup(0));
            }
            return;
        }
        units_counted += units_in_this_cell;
    }
}

/// Spawns a single archer unit for multiplayer.
pub(super) fn spawn_mp_archer(
    commands: &mut Commands,
    archer_assets: &ArcherAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    team: Team,
    host_side: bool,
) {
    // Position archers one row behind infantry
    let infantry_cells = cells_needed(MP_INFANTRY_COUNT);
    let infantry_rows = infantry_cells.div_ceil(DEFENDER_GRID_COLS);
    let last_infantry_row = DEFENDER_GRID_ROWS.saturating_sub(infantry_rows);
    let archer_row = last_infantry_row.saturating_sub(1);

    let archer_cells_needed = cells_needed(MP_ARCHER_COUNT);
    let units_per_cell = distribute_units_to_cells(MP_ARCHER_COUNT);

    let mut units_counted = 0;
    for cell_idx in 0..archer_cells_needed.min(DEFENDER_GRID_COLS) {
        let units_in_this_cell = units_per_cell[cell_idx as usize];
        if unit_index < units_counted + units_in_this_cell {
            let (spawn_x, spawn_z) = if host_side {
                calculate_mp_defender_grid_position(archer_row, cell_idx)
            } else {
                calculate_mp_guest_defender_grid_position(archer_row, cell_idx)
            };
            let mut rng = rand::rng();
            let (final_x, final_z) = random_position_in_cell(&mut rng, spawn_x, spawn_z);

            let hitbox = Hitbox::new(ARCHER_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;

            let tint = crate::game::units::systems::archer_sprite_tint_for_team(team);
            let anim = WalkingAnimation::default();
            let material = crate::game::units::systems::create_default_sprite_material(
                materials,
                archer_assets.sprite_texture.clone(),
                tint,
            );

            let flow_field = if host_side {
                FlowFieldInfluence::Defender {
                    spawn_pos: Vec2::new(spawn_x, spawn_z),
                }
            } else {
                FlowFieldInfluence::Attacker
            };

            let mut ec = commands.spawn((
                Mesh3d(archer_assets.sprite_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(final_x, spawn_y, final_z),
                crate::game::components::Velocity::default(),
                crate::game::components::Acceleration::new(),
                hitbox,
                Health::new(UNIT_HEALTH),
                MovementSpeed(ARCHER_MOVEMENT_SPEED),
                staggered_attack_timing(),
                Effectiveness::new(),
                team,
                Archer,
            ));
            ec.insert((
                anim,
                FacingDirection::default(),
                AttackRange {
                    min_range: ARCHER_MIN_RANGE,
                    max_range: ARCHER_MAX_RANGE,
                },
                ArcherMovementTimer::new(),
                TargetingVelocity::default(),
                FlockingVelocity::default(),
                FlowFieldVelocity::default(),
                flow_field,
                crate::game::units::components::FlockingModifier::new(1.0, 1.0, 0.0),
                Teleportable,
                Billboard,
                OnMultiplayerGameScreen,
            ));
            if team == Team::Attackers {
                ec.insert(WaveGroup(0));
            }
            return;
        }
        units_counted += units_in_this_cell;
    }
}

/// Spawns a King unit at the given position origin for multiplayer.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_mp_king(
    commands: &mut Commands,
    king_assets: &crate::game::units::king::resources::KingAssets,
    spell_assets: &crate::game::units::wizard::spells::visual_assets::SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    king_spawned: &mut ResMut<KingSpawned>,
    wizard_position: Vec3,
    center_angle: f32,
    team: Team,
) {
    use crate::game::units::commander::{AuraDamageBuff, AuraSpeedBuff, Commander, TeamFilter};
    use crate::game::units::components::DamageMultiplier;
    use crate::game::units::king::components::King;
    use crate::game::units::king::constants::*;

    let radius = MP_DEFENDER_GRID_GROUND_RANGE + 600.0;
    let spawn_x = wizard_position.x + radius * center_angle.cos();
    let spawn_z = wizard_position.z + radius * center_angle.sin();

    let hitbox = Hitbox::new(KING_RADIUS, KING_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;
    let spawn_pos = Vec2::new(spawn_x, spawn_z);

    let team_filter = match team {
        Team::Defenders => TeamFilter::Defenders,
        Team::Attackers => TeamFilter::Attackers,
        _ => TeamFilter::Defenders,
    };

    let anim = WalkingAnimation::default();
    let king_material = crate::game::units::systems::create_default_sprite_material(
        materials,
        king_assets.sprite_texture.clone(),
        KING_SPRITE_TINT,
    );

    let king_entity = commands
        .spawn((
            Mesh3d(king_assets.sprite_mesh.clone()),
            MeshMaterial3d(king_material),
            Transform::from_xyz(spawn_x, spawn_y, spawn_z),
            crate::game::components::Velocity::default(),
            crate::game::components::Acceleration::new(),
            hitbox,
            Health::new(KING_HEALTH),
            MovementSpeed(KING_MOVEMENT_SPEED),
            staggered_attack_timing(),
            Effectiveness::new(),
            DamageMultiplier(KING_DAMAGE_PERCENTAGE),
            team,
            King,
        ))
        .insert((
            anim,
            FacingDirection::default(),
            Commander {
                aura_radius: KING_AURA_RADIUS,
                team_filter,
            },
            AuraDamageBuff(KING_AURA_DAMAGE_PERCENTAGE),
            AuraSpeedBuff(KING_AURA_SPEED_PERCENTAGE),
        ))
        .insert((
            TargetingVelocity::default(),
            FlockingVelocity::default(),
            FlowFieldVelocity::default(),
            if team == Team::Defenders {
                FlowFieldInfluence::Defender { spawn_pos }
            } else {
                FlowFieldInfluence::Attacker
            },
            // NOTE: the King is intentionally NOT `Teleportable` in multiplayer —
            // teleporting your own King out of reach was an exploit to stall the
            // match forever. (Single-player `spawn_king` keeps it teleportable.)
            crate::game::units::components::FlockingModifier::new(1.0, 0.0, 0.0),
            Billboard,
            OnMultiplayerGameScreen,
        ))
        .id();

    // MP attacker kings/guards are pre-activated — see infantry/archer
    // spawn for rationale. WaveGroup(0) prevents `is_staging_attacker`
    // from classing them as inactive due to the missing tag.
    if team == Team::Attackers {
        commands.entity(king_entity).insert(WaveGroup(0));
    }

    // Spawn the SP-style aura sphere as a child of the king. Replaces the
    // earlier flat ground-plane circle so both MP peers — and SP — show
    // the same volumetric aura halo.
    crate::game::units::king::systems::spawn_king_aura_visual(
        commands,
        king_entity,
        spell_assets,
        OnMultiplayerGameScreen,
    );

    king_spawned.0 = true;
}

/// Spawns a King's Guard unit for multiplayer at the given position origin.
pub(super) fn spawn_mp_kings_guard(
    commands: &mut Commands,
    infantry_assets: &InfantryAssets,
    materials: &mut Assets<StandardMaterial>,
    guard_index: u32,
    wizard_position: Vec3,
    center_angle: f32,
    team: Team,
) {
    use crate::game::units::components::KingsGuard;
    use crate::game::units::elite::{EliteDamageBonus, EliteHealthBonus, EliteSpeedBonus};
    use crate::game::units::infantry::constants::KINGS_GUARD_SPRITE_TINT;

    // King's position: same calculation as spawn_mp_king
    let radius = MP_DEFENDER_GRID_GROUND_RANGE + 600.0;
    let king_x = wizard_position.x + radius * center_angle.cos();
    let king_z = wizard_position.z + radius * center_angle.sin();

    let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let angle = guard_index as f32 * (std::f32::consts::TAU / KINGS_GUARD_COUNT as f32);
    let final_x = king_x + KINGS_GUARD_ORBIT_RADIUS * angle.cos();
    let final_z = king_z + KINGS_GUARD_ORBIT_RADIUS * angle.sin();

    let anim = WalkingAnimation::default();
    let guard_material = crate::game::units::systems::create_default_sprite_material(
        materials,
        infantry_assets.sprite_texture.clone(),
        KINGS_GUARD_SPRITE_TINT,
    );

    let mut ec = commands.spawn((
        Mesh3d(infantry_assets.sprite_mesh.clone()),
        MeshMaterial3d(guard_material),
        Transform::from_xyz(final_x, spawn_y, final_z),
        // `Velocity` is required so the entity passes the host's
        // `send_state_snapshots` query (`&Velocity`). Without it King's
        // Guards silently fall out of every snapshot → guest never sees
        // them and they appear invincible/missing.
        crate::game::components::Velocity::default(),
        hitbox,
        Health::new(UNIT_HEALTH),
        staggered_attack_timing(),
        Effectiveness::new(),
        team,
        Infantry,
        KingsGuard(guard_index),
    ));
    ec.insert((
        anim,
        FacingDirection::default(),
        Teleportable,
        Billboard,
        OnMultiplayerGameScreen,
        EliteHealthBonus(crate::game::units::elite::ELITE_HEALTH_BONUS),
        EliteDamageBonus(crate::game::units::elite::ELITE_DAMAGE_BONUS),
        EliteSpeedBonus(crate::game::units::elite::ELITE_SPEED_BONUS),
        crate::game::units::elite::EliteAttackSpeedBonus(
            crate::game::units::elite::ELITE_ATTACK_SPEED_BONUS,
        ),
    ));
    if team == Team::Attackers {
        ec.insert(WaveGroup(0));
    }
}
