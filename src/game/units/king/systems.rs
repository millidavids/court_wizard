use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::KingAssets;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker};
use crate::game::resources::InitialDefenderCount;
use crate::game::units::commander::{
    AuraDamageBuff, AuraSpeedBuff, Commander, CommanderAuraParticle, TeamFilter,
};
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, DamageMultiplier,
    Effectiveness, EliteSpeedBonus, FacingDirection, FlockingModifier, FlockingVelocity,
    FrozenSolidModifier, HasteModifier, Health, Hitbox, KingsGuard, MovementSpeed,
    PolymorphedModifier, RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier,
    Sleepwalking, SlowMovementModifier, TargetingVelocity, Team, Teleportable, WalkingAnimation,
};
use crate::game::units::systems::create_default_sprite_material;
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::session::MultiplayerSession;

/// Spawns the King unit at the center of the defender grid.
///
/// King spawns in the center of the radial defender formation,
/// positioned between the wizard and battlefield center.
pub fn spawn_king(
    commands: &mut Commands,
    king_assets: &KingAssets,
    _meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    spell_assets: &SpellVisualAssets,
    king_spawned: &mut KingSpawned,
) {
    // King spawns at exact center of defender grid
    // Use center angle and base range (no row/col offsets)
    let angle = DEFENDER_GRID_CENTER_ANGLE;
    let radius = DEFENDER_GRID_GROUND_RANGE + 600.0;
    let spawn_x = WIZARD_POSITION.x + radius * angle.cos();
    let spawn_z = WIZARD_POSITION.z + radius * angle.sin();

    // Define King hitbox (larger than standard units)
    let hitbox = Hitbox::new(KING_RADIUS, KING_HITBOX_HEIGHT);

    // Position unit so bottom edge is 1 unit above battlefield (Y=0)
    let spawn_y = hitbox.height / 2.0 + 1.0;

    // Store spawn position for rallying when not activated
    let spawn_pos = Vec2::new(spawn_x, spawn_z);

    let anim = WalkingAnimation::default();
    let king_material = create_default_sprite_material(
        materials,
        king_assets.sprite_texture.clone(),
        KING_SPRITE_TINT,
    );

    // Spawn the King unit with Commander components
    let king_entity = commands
        .spawn((
            Mesh3d(king_assets.sprite_mesh.clone()),
            MeshMaterial3d(king_material),
            Transform::from_xyz(spawn_x, spawn_y, spawn_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(KING_HEALTH),
            MovementSpeed(KING_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            DamageMultiplier(KING_DAMAGE_PERCENTAGE),
            Team::Defenders,
            King, // Marker for game-ending logic
        ))
        .insert((
            anim,
            FacingDirection::default(),
            // Commander components
            Commander {
                aura_radius: KING_AURA_RADIUS,
                team_filter: TeamFilter::Defenders,
            },
            AuraDamageBuff(KING_AURA_DAMAGE_PERCENTAGE),
            AuraSpeedBuff(KING_AURA_SPEED_PERCENTAGE),
        ))
        .insert((
            TargetingVelocity::default(),
            FlockingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Defender { spawn_pos },
            Teleportable,
            FlockingModifier::new(1.0, 0.0, 0.0),
            Billboard,
            OnGameplayScreen,
        ))
        .id();

    // Spawn the visual aura sphere as a child of the king.
    spawn_king_aura_visual(commands, king_entity, spell_assets, OnGameplayScreen);

    // Mark that King has been spawned
    king_spawned.0 = true;
}

/// Updates King targeting velocity toward nearest enemy.
///
/// The King always moves directly toward the nearest enemy.
/// Also sets InMelee component if an enemy is within melee range.
/// King is gated by the DefendersActivated resource.
pub fn update_king_targeting(
    defenders_activated: Res<crate::game::units::infantry::components::DefendersActivated>,
    mut commands: Commands,
    mut king: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut TargetingVelocity,
            Option<&crate::game::units::components::RetaliationTarget>,
        ),
        (With<King>, Without<Corpse>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<crate::game::units::assassin::Assassin>,
            Without<StagingAttacker>,
            Without<crate::game::units::components::Flying>,
        ),
    >,
) {
    // Collect snapshot of all unit positions (excludes assassins, staging attackers, and flying units)
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    // Update King's targeting velocity
    for (entity, transform, team, mut targeting_velocity, retaliation) in &mut king {
        // Skip inactive King (wait for defenders to activate)
        if !defenders_activated.active {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
            commands
                .entity(entity)
                .remove::<crate::game::units::components::InMelee>();
            continue;
        }

        // Use shared melee targeting function
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            transform,
            *team,
            &mut targeting_velocity,
            &mut commands,
            retaliation.map(|r| r.0),
        );
    }
}

/// King-specific movement system.
///
/// Uses acceleration-based physics with maximum speed capping.
/// TargetingVelocity and FlockingVelocity are treated as acceleration forces.
/// King slows down when in melee to prevent erratic movement.
#[allow(clippy::type_complexity)]
pub fn king_movement(
    time: Res<Time>,
    mut king_units: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&crate::game::units::components::InMelee>,
            Option<&CommanderAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            Option<&SlowMovementModifier>,
            (
                Option<&CauldronSpeedModifier>,
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
            ),
            (
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&PolymorphedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                Option<&crate::game::units::components::Petrified>,
            ),
        ),
        With<King>,
    >,
) {
    // Process King unit
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        in_melee,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, sleepwalking, banished, _polymorphed, sickened, frozen, stunned, petrified),
    ) in &mut king_units
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
        ) {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        // Use shared weighted movement function
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            in_melee.is_some(),
            aura_modifier.map(|m| m.0),
            terrain_modifier.map(|m| m.0),
            slow_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
        );
    }
}

/// King cohesion force system.
///
/// Applies a dynamic cohesion force to defenders, pulling them toward the King.
/// The force strength increases when enemies are near (threatened) and decreases when safe.
/// This is King-specific behavior separate from the generic commander aura system.
///
/// Note: Damage and speed buffs are now handled by the generic commander system.
pub fn king_cohesion_force(
    king_query: Query<(&Transform, &Team), (With<King>, Without<Corpse>)>,
    mut defenders: Query<
        (&Transform, &Team, &mut FlockingVelocity),
        (Without<King>, Without<Corpse>),
    >,
    all_units: Query<(&Transform, &Team), Without<Corpse>>,
) {
    // Process each King and apply cohesion to their team's units
    for (king_transform, king_team) in &king_query {
        let king_pos = king_transform.translation;

        // Find nearest enemy to this King
        let nearest_enemy_distance = all_units
            .iter()
            .filter(|(_, team)| *team != king_team)
            .map(|(transform, _)| transform.translation.distance(king_pos))
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(f32::MAX);

        // Calculate threat level: interpolate between BASE and THREATENED
        let threat_factor = if nearest_enemy_distance > KING_AURA_RADIUS {
            0.0
        } else {
            1.0 - (nearest_enemy_distance / KING_AURA_RADIUS)
        };

        let cohesion_strength =
            KING_COHESION_BASE + (KING_COHESION_THREATENED - KING_COHESION_BASE) * threat_factor;

        // Apply cohesion force to same-team defenders within aura radius
        for (unit_transform, team, mut flocking_velocity) in &mut defenders {
            if team != king_team {
                continue;
            }

            let unit_pos = unit_transform.translation;
            let distance_to_king = unit_pos.distance(king_pos);

            // Check if unit is within aura radius
            if distance_to_king < KING_AURA_RADIUS && distance_to_king > 0.1 {
                // Calculate direction toward King
                let to_king = (king_pos - unit_pos).normalize_or_zero();

                // Add cohesion force to flocking velocity
                // Scale by distance (stronger pull when closer to edge of aura)
                let distance_factor = distance_to_king / KING_AURA_RADIUS;
                let cohesion_force = to_king * cohesion_strength * distance_factor;

                flocking_velocity.velocity += Vec3::new(cohesion_force.x, 0.0, cohesion_force.z);

                // Re-normalize to maintain consistent influence
                flocking_velocity.velocity = flocking_velocity.velocity.normalize_or_zero();
            }
        }
    }
}

/// Snaps King's Guard units to fixed positions around the King each frame.
///
/// Guards orbit the King at a fixed radius. Their positions are set directly
/// rather than using velocity/acceleration, so they stay locked to the King.
/// We also write the per-frame movement delta into `Velocity` so the shared
/// `update_walking_animation` and `update_facing_direction` systems (which
/// query `&Velocity`) match the guard entity and animate it correctly. Without
/// this they'd skip the guard and it would freeze on its idle frame, always
/// facing forward.
pub fn snap_kings_guard_to_king(
    time: Res<Time>,
    king_query: Query<(&Transform, &Team), (With<King>, Without<Corpse>)>,
    mut guards: Query<
        (&KingsGuard, &Team, &mut Transform, &mut Velocity),
        (Without<King>, Without<Corpse>),
    >,
) {
    let delta = time.delta_secs();
    let inv_delta = if delta > 1e-6 { 1.0 / delta } else { 0.0 };
    // Snap each guard to their team's King
    for (king_transform, king_team) in &king_query {
        let king_pos = king_transform.translation;

        for (guard, guard_team, mut transform, mut velocity) in &mut guards {
            if guard_team != king_team {
                continue;
            }
            let angle = guard.0 as f32 * (std::f32::consts::TAU / KINGS_GUARD_COUNT as f32);
            let new_x = king_pos.x + KINGS_GUARD_ORBIT_RADIUS * angle.cos();
            let new_z = king_pos.z + KINGS_GUARD_ORBIT_RADIUS * angle.sin();
            let dx = new_x - transform.translation.x;
            let dz = new_z - transform.translation.z;
            transform.translation.x = new_x;
            transform.translation.z = new_z;
            velocity.x = dx * inv_delta;
            velocity.z = dz * inv_delta;
        }
    }
}

/// Attaches the `SpellShield` marker component to every King that appears in
/// a multiplayer match. Runs once per king on the host (via `Added<King>`).
///
/// **Host only.** The guest must NOT independently attach SpellShield —
/// shield state is host-authoritative and propagates to the guest via the
/// `apply_state_snapshot` spell-shield transition handler. If the guest ran
/// this too, it would insert SpellShield on every ghost king the frame after
/// the ghost gets its `King` marker, racing with (and briefly overriding)
/// the snapshot's intended state.
///
/// No separate visual is spawned here — the king's aura (added by the spawn
/// path; see `spawn_king_aura_visual`) is the constant visual.
pub fn attach_king_spell_shield(
    mut commands: Commands,
    session: Option<Res<MultiplayerSession>>,
    new_kings: Query<Entity, Added<King>>,
) {
    use crate::networking::resources::PeerRole;
    // VERSUS only. The spell shield is a duel mechanic — it must NOT appear in
    // endless/roguelite, including CO-OP endless/roguelite (where the host holds
    // a co-op session but plays the single-player battlefield). Single-player has
    // no session, so it's already excluded.
    if !session.is_some_and(|s| s.role == PeerRole::Host && !s.is_coop()) {
        return;
    }

    for entity in &new_kings {
        commands.entity(entity).insert(SpellShield);
    }
}

/// Spawns the king's aura sphere (the SP-style halo) as a child of the given
/// king entity. Both single-player and multiplayer kings — host-local AND
/// ghost — call this so the aura looks identical on every peer. Generic over
/// the screen-cleanup marker (`OnGameplayScreen` for SP, `OnMultiplayerGameScreen`
/// for MP) so the right cleanup hook sweeps it.
///
/// The mesh + material are the same shared `explosion_sphere` and
/// `king_aura_sphere` handles used everywhere else; both are reference-counted
/// asset handles and nothing later mutates them per-entity, so sharing is safe.
pub(in crate::game) fn spawn_king_aura_visual<M: Component>(
    commands: &mut Commands,
    parent_entity: Entity,
    spell_assets: &SpellVisualAssets,
    screen_marker: M,
) {
    let aura_entity = commands
        .spawn((
            Mesh3d(spell_assets.explosion_sphere.clone()),
            MeshMaterial3d(spell_assets.king_aura_sphere.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(KING_AURA_RADIUS)),
            // `KingAuraVisual` marker lets `despawn_king_aura_on_death`
            // find and despawn the aura when the king dies — the king's
            // entity itself becomes a corpse rather than being despawned,
            // so child hierarchy cleanup does not fire.
            super::components::KingAuraVisual,
            screen_marker,
        ))
        .id();
    commands.entity(parent_entity).add_child(aura_entity);
}

/// Despawns the king's aura sphere when the king becomes a corpse. Without
/// this, the flat corpse sprite ends up surrounded by the still-glowing
/// aura halo — visually wrong on both SP and MP. The aura is a child of
/// the king entity, so we look it up via `Children` rather than a global
/// query (avoids tearing down the OTHER king's aura when one king dies).
pub fn despawn_king_aura_on_death(
    mut commands: Commands,
    dead_kings: Query<&Children, (With<King>, Added<Corpse>)>,
    aura_visuals: Query<(), With<super::components::KingAuraVisual>>,
) {
    for children in &dead_kings {
        for child in children.iter() {
            if aura_visuals.contains(child)
                && let Ok(mut ec) = commands.get_entity(child)
            {
                ec.try_despawn();
            }
        }
    }
}

/// Removes a King's spell shield when fewer than 10% of their own team's
/// non-King units remain alive. Iterates per-king so both kings in MP get
/// independent shield-degradation tracking. The aura visual is unrelated
/// and stays on as long as the king lives — this only touches the marker.
pub fn update_king_spell_shield(
    mut commands: Commands,
    kings: Query<(Entity, &Team), (With<King>, With<SpellShield>, Without<Corpse>)>,
    units: Query<&Team, (Without<Corpse>, Without<King>)>,
    initial_count: Option<Res<InitialDefenderCount>>,
    session: Option<Res<MultiplayerSession>>,
) {
    // Initial-count source differs by mode:
    // - SP: `InitialDefenderCount` is set by the SP loader to the actual
    //   defender army size, which scales with progression. Only the
    //   Defender team has a king in SP, so a single threshold suffices.
    // - MP: both teams start with a known, symmetric army size; we use the
    //   compile-time MP constants instead of plumbing per-team resources.
    let initial = if session.is_some() {
        use crate::game::constants::{KINGS_GUARD_COUNT, MP_ARCHER_COUNT, MP_INFANTRY_COUNT};
        (MP_INFANTRY_COUNT + MP_ARCHER_COUNT + KINGS_GUARD_COUNT) as f32
    } else if let Some(c) = initial_count {
        c.0 as f32
    } else {
        return;
    };
    if initial <= 0.0 {
        return;
    }

    for (king_entity, team) in &kings {
        let living = units.iter().filter(|t| *t == team).count() as f32;
        if living / initial <= SPELL_SHIELD_THRESHOLD {
            commands.entity(king_entity).remove::<SpellShield>();
        }
    }
}

/// Multiplayer anti-stall: force-removes the King's spell shield once the match
/// has run `MP_SPELL_SHIELD_MAX_DURATION_SECS` seconds, regardless of how many
/// units are still alive. Without this, a player could maze / play keep-away to
/// keep enough units alive that the kill-threshold drop (`update_king_spell_shield`)
/// never fires, stalling the match forever.
///
/// **Host only** (registered under `is_gameplay_running`): shield state is
/// host-authoritative and propagates to the guest via the snapshot `SPELL_SHIELD`
/// bit. Reuses the host-side `KillStats.elapsed_time` match clock (reset at match
/// start), so no extra timer is needed. Removes the shield from every still-shielded
/// king, so both MP kings lose their shields together at the timeout.
pub fn expire_king_spell_shield_on_timeout(
    mut commands: Commands,
    kill_stats: Res<crate::game::resources::KillStats>,
    kings: Query<Entity, (With<King>, With<SpellShield>, Without<Corpse>)>,
) {
    if kill_stats.elapsed_time >= MP_SPELL_SHIELD_MAX_DURATION_SECS {
        for king_entity in &kings {
            commands.entity(king_entity).remove::<SpellShield>();
        }
    }
}

/// Spawns small particles from all commanders that travel outward to the aura edge.
pub fn spawn_commander_aura_particles(
    mut commands: Commands,
    commanders: Query<(&Transform, &Commander)>,
    spell_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut timer: Local<f32>,
) {
    use rand::Rng;

    *timer += time.delta_secs();
    if *timer < 0.12 {
        return;
    }
    *timer -= 0.12;

    for (transform, commander) in &commanders {
        let pos = transform.translation;
        let radius = commander.aura_radius;

        for _ in 0..2 {
            let dir = Vec3::new(
                game_rng.0.random_range(-1.0..1.0_f32),
                game_rng.0.random_range(0.0..0.5_f32),
                game_rng.0.random_range(-1.0..1.0_f32),
            )
            .normalize_or(Vec3::Y);

            let speed = game_rng.0.random_range(80.0..150.0_f32);
            let lifetime = radius / speed;

            commands.spawn((
                CommanderAuraParticle {
                    velocity: dir * speed,
                    time_alive: 0.0,
                    lifetime,
                },
                Mesh3d(spell_assets.particle_quad.clone()),
                MeshMaterial3d(spell_assets.buff_mote.clone()),
                Transform::from_translation(pos)
                    .with_rotation(UPWARD_ROTATION)
                    .with_scale(Vec3::splat(3.0)),
                OnGameplayScreen,
            ));
        }
    }
}

/// Moves commander aura particles outward and fades them over their lifetime.
pub fn update_commander_aura_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut CommanderAuraParticle, &mut Transform)>,
) {
    let delta = time.delta_secs();

    for (entity, mut particle, mut transform) in &mut particles {
        particle.time_alive += delta;

        if particle.time_alive >= particle.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Move outward
        transform.translation += particle.velocity * delta;

        // Fade: grow slightly then shrink at the end
        let t = particle.time_alive / particle.lifetime;
        let scale = if t < 0.2 {
            3.0 * (t / 0.2)
        } else if t > 0.7 {
            3.0 * (1.0 - (t - 0.7) / 0.3)
        } else {
            3.0
        };
        transform.scale = Vec3::splat(scale);
    }
}
