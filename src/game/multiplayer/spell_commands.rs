//! Guest spell result sending and host spell result processing.
//!
//! Each client runs its own casting pipeline (cast time, mana, cast bar).
//! When a spell completes, the guest sends a `SpellAction` result message
//! to the host with only the data needed to spawn the spell effect.
//!
//! For channeled spells (magic missile, disintegrate, raise the dead),
//! the guest sends `ChannelStarted`/`ChannelUpdate`/`ChannelEnded` messages.
//! The host translates these into `GuestCursorPosition` and `GuestInputState`
//! resources, which the existing channeled spell guest systems read.

use bevy::prelude::*;

use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{ObstacleChanged, ObstacleType, OBSTACLE_BUFFER};
use crate::game::units::components::*;
use crate::game::units::DamageType;
use crate::game::units::wizard::components::*;
use crate::networking::protocol::{NetworkMessage, SpellAction};
use crate::networking::resources::NetworkConnection;
use crate::networking::snapshot::SpellEffectKind;

/// Marker resource inserted on the guest to prevent spell entity spawning.
///
/// When present, spell casting logic runs normally (CastingState, mana, cast bar)
/// but skips the entity spawn step. The `handle_X_casting` wrapper detects
/// completed casts and sends result messages to the host instead.
#[derive(Resource)]
pub struct SkipSpellSpawning;

/// Resource tracking the guest's cursor world position on the host.
///
/// Used only for channeled spells (magic missile, disintegrate, raise the dead).
/// Updated from `ChannelStarted`/`ChannelUpdate` messages.
#[derive(Resource, Default)]
pub struct GuestCursorPosition {
    pub position: Option<Vec3>,
}

/// Resource translating guest channel messages into input signals.
///
/// Used only for the 3 channeled spell guest systems on the host.
/// Updated from `ChannelStarted`/`ChannelUpdate`/`ChannelEnded` messages.
#[derive(Resource, Default)]
pub struct GuestInputState {
    /// True on the frame a `ChannelStarted` message is received.
    pub just_pressed: bool,
    /// True while the guest is channeling.
    pub pressed: bool,
    /// True on the frame a `ChannelEnded` message is received.
    pub just_released: bool,
}

/// Marker component for disintegrate beams owned by the guest wizard.
///
/// Used to distinguish guest beams from host beams in cleanup systems,
/// so each wizard's beams are cleaned up independently.
#[derive(Component)]
pub struct GuestBeam;

/// Pending guest spell action to be processed by the spawning system.
///
/// Stored in a resource so the message-processing system (which handles
/// all network messages) can be separate from the spell-spawning system
/// (which needs many query parameters).
#[derive(Resource, Default)]
pub struct PendingGuestSpellActions {
    pub actions: Vec<(SpellAction, Option<Transform>)>,
}

// ── Host Systems ──────────────────────────────────────────────────────

/// Processes incoming network messages and extracts spell results.
///
/// Spell actions are stored in `PendingGuestSpellActions` for the spawning
/// system to process. Channel messages update `GuestInputState`/`GuestCursorPosition`
/// directly since they don't need entity queries.
#[allow(clippy::too_many_arguments)]
pub fn process_guest_spell_results(
    mut connection: ResMut<NetworkConnection>,
    mut cursor: ResMut<GuestCursorPosition>,
    mut guest_input: ResMut<GuestInputState>,
    mut commands: Commands,
    wizard_query: Query<(Entity, &Transform), With<GuestWizard>>,
    mut pending: ResMut<PendingGuestSpellActions>,
) {
    // Reset per-frame input flags
    guest_input.just_pressed = false;
    guest_input.just_released = false;

    let messages: Vec<NetworkMessage> = connection.incoming_messages.drain(..).collect();
    let mut unhandled = Vec::new();

    let wizard_data = wizard_query.single().ok().map(|(e, t)| (e, *t));

    for msg in messages {
        match msg {
            NetworkMessage::SpellResult(action) => match &action {
                SpellAction::ChannelStarted { spell, cursor_pos, empowerment } => {
                    cursor.position = Some(Vec3::new(cursor_pos[0], cursor_pos[1], cursor_pos[2]));
                    guest_input.just_pressed = true;
                    guest_input.pressed = true;
                    if let Some((entity, _)) = wizard_data {
                        commands.entity(entity).insert(PrimedSpell {
                            spell: *spell,
                            cast_time: 0.0,
                            empowerment: *empowerment,
                            empowerment_consumed: true,
                            mana_multiplier: 1.0,
                            range_multiplier: 1.0,
                        });
                    }
                }
                SpellAction::ChannelUpdate { cursor_pos } => {
                    cursor.position = Some(Vec3::new(cursor_pos[0], cursor_pos[1], cursor_pos[2]));
                    guest_input.pressed = true;
                }
                SpellAction::ChannelEnded => {
                    guest_input.just_released = true;
                    guest_input.pressed = false;
                }
                SpellAction::SpellCast { .. } | SpellAction::DragSpellCast { .. } => {
                    let wt = wizard_data.map(|(_, t)| t);
                    pending.actions.push((action, wt));
                }
            },
            other => unhandled.push(other),
        }
    }

    if !unhandled.is_empty() {
        connection.incoming_messages.extend(unhandled);
    }
}

/// Spawns spell entities on the host from pending guest spell actions.
///
/// Runs after `process_guest_spell_results` in the same frame.
/// Split into a separate system to keep query parameters manageable.
#[allow(clippy::too_many_arguments)]
pub fn spawn_guest_spell_entities(
    mut pending: ResMut<PendingGuestSpellActions>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut units_query: Query<(Entity, &Transform, &Team), (Without<Wizard>, Without<Corpse>)>,
    mut health_query: Query<(&mut Health, Option<&mut TemporaryHitPoints>)>,
    infantry_assets: Option<Res<crate::game::units::infantry::resources::InfantryAssets>>,
    existing_marks: Query<Entity, With<crate::game::units::wizard::spells::mark_of_death::components::ActiveMarkOfDeath>>,
    polymorph_query: Query<(Entity, &Transform, &MeshMaterial3d<StandardMaterial>), (Without<Corpse>, Without<PolymorphedModifier>, Without<Wizard>)>,
) {
    let actions: Vec<_> = pending.actions.drain(..).collect();

    for (action, wizard_transform) in actions {
        match action {
            SpellAction::SpellCast { spell, cursor_pos, empowerment } => {
                let pos = Vec3::new(cursor_pos[0], cursor_pos[1], cursor_pos[2]);
                spawn_guest_spell(
                    spell, pos, empowerment,
                    wizard_transform.as_ref(),
                    &mut commands, &mut meshes, &mut materials,
                    &mut obstacle_events,
                    &mut units_query,
                    &mut health_query,
                    infantry_assets.as_deref(),
                    &existing_marks,
                    &polymorph_query,
                );
            }
            SpellAction::DragSpellCast { spell, anchor, end_pos, empowerment } => {
                let a = Vec3::new(anchor[0], anchor[1], anchor[2]);
                let e = Vec3::new(end_pos[0], end_pos[1], end_pos[2]);
                spawn_guest_drag_spell(
                    spell, a, e, empowerment,
                    &mut commands, &mut meshes, &mut materials,
                    &mut obstacle_events,
                );
            }
            _ => {}
        }
    }
}

// ── Host Spell Spawning ──────────────────────────────────────────────

/// Spawns a fire-and-forget spell entity on the host for the guest wizard.
#[allow(clippy::too_many_arguments)]
fn spawn_guest_spell(
    spell: Spell,
    cursor_pos: Vec3,
    empowerment: f32,
    wizard_transform: Option<&Transform>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    units_query: &mut Query<(Entity, &Transform, &Team), (Without<Wizard>, Without<Corpse>)>,
    health_query: &mut Query<(&mut Health, Option<&mut TemporaryHitPoints>)>,
    infantry_assets: Option<&crate::game::units::infantry::resources::InfantryAssets>,
    existing_marks: &Query<Entity, With<crate::game::units::wizard::spells::mark_of_death::components::ActiveMarkOfDeath>>,
    polymorph_query: &Query<(Entity, &Transform, &MeshMaterial3d<StandardMaterial>), (Without<Corpse>, Without<PolymorphedModifier>, Without<Wizard>)>,
) {
    use crate::game::units::wizard::spells::*;

    match spell {
        // ── Simple entity spawners ──────────────────────────────────

        Spell::Fireball => {
            if let Some(wt) = wizard_transform {
                let origin = wt.translation + Vec3::new(0.0, fireball::constants::SPAWN_HEIGHT_OFFSET, 0.0);
                let primed = PrimedSpell {
                    spell: Spell::Fireball,
                    cast_time: 0.0,
                    empowerment,
                    empowerment_consumed: true,
                    mana_multiplier: 1.0,
                    range_multiplier: 1.0,
                };
                fireball::systems::spawn_fireball(commands, meshes, materials, origin, cursor_pos, &primed);
            }
        }

        Spell::BlackHole => {
            black_hole::systems::spawn_black_hole(commands, meshes, materials, cursor_pos, empowerment);
        }

        Spell::HealingPlume => {
            let radius = healing_plume::constants::CIRCLE_RADIUS * empowerment;
            healing_plume::systems::spawn_healing_plume_zone(
                commands, meshes, materials, cursor_pos, radius, empowerment,
            );
        }

        Spell::SpikeGrowth => {
            let radius = spike_growth::constants::CIRCLE_RADIUS * empowerment;
            spike_growth::systems::spawn_spike_growth_zone(
                commands, meshes, materials, cursor_pos, radius, empowerment, obstacle_events,
            );
        }

        Spell::LightningRod => {
            lightning_rod::systems::spawn_lightning_rod(commands, meshes, materials, cursor_pos, empowerment);
        }

        Spell::ArcaneCrystal => {
            arcane_crystal::systems::spawn_crystal(commands, meshes, materials, cursor_pos, empowerment);
        }

        Spell::FogCloud => {
            let radius = fog_cloud::constants::CIRCLE_RADIUS * empowerment;
            fog_cloud::systems::spawn_fog_cloud_zone(
                commands, meshes, materials, cursor_pos, radius, empowerment,
            );
        }

        Spell::Grease => {
            let radius = grease::constants::CIRCLE_RADIUS * empowerment;
            grease::systems::spawn_grease_zone(
                commands, meshes, materials, cursor_pos, radius, empowerment, obstacle_events,
            );
        }

        // ── Concentration spells (spawn storm entity) ───────────────

        Spell::MeteorFall => {
            let radius = meteor_fall::constants::STORM_RADIUS * empowerment;
            commands.spawn((
                meteor_fall::components::MeteorFallStorm::new(cursor_pos, radius, empowerment),
                ConcentrationSpell { spell_name: "Meteor Fall" },
                OnGameplayScreen,
            ));
        }

        Spell::Squall => {
            let radius = squall::constants::STORM_RADIUS * empowerment;
            commands.spawn((
                squall::components::SquallStorm::new(cursor_pos, radius, empowerment),
                ConcentrationSpell { spell_name: "Squall" },
                OnGameplayScreen,
            ));
        }

        Spell::PlagueWind => {
            let scale = empowerment;
            let radius = plague_wind::constants::CLOUD_RADIUS * scale;
            let damage = plague_wind::constants::DAMAGE_PER_TICK * scale;
            let direction = Vec3::new(
                crate::game::constants::ATTACKER_GRID_CENTER_ANGLE.cos(),
                0.0,
                crate::game::constants::ATTACKER_GRID_CENTER_ANGLE.sin(),
            ).normalize();

            let origin_2d = Vec2::new(cursor_pos.x, cursor_pos.z);
            let buffered = radius + OBSTACLE_BUFFER;
            obstacle_events.write(ObstacleChanged {
                bounds: bevy::math::Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
                obstacle_type: ObstacleType::Hazard(10.0),
            });

            let cloud_mesh = meshes.add(Circle::new(radius));
            let cloud_material = materials.add(StandardMaterial {
                base_color: plague_wind::constants::CLOUD_COLOR,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            });

            commands.spawn((
                Mesh3d(cloud_mesh),
                MeshMaterial3d(cloud_material),
                Transform::from_translation(Vec3::new(cursor_pos.x, 1.0, cursor_pos.z))
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                plague_wind::components::PlagueWindCloud::new(
                    cursor_pos, radius, damage,
                    plague_wind::constants::TICK_INTERVAL,
                    plague_wind::constants::CLOUD_DURATION * scale,
                    plague_wind::constants::CLOUD_SPEED,
                    direction,
                ),
                NetworkedSpellEffect { kind: SpellEffectKind::PlagueWindCloud },
                OnGameplayScreen,
            ));
        }

        // ── AoE apply spells (apply modifiers to units in radius) ───

        Spell::Entangle => {
            let radius = entangle::constants::CIRCLE_RADIUS * empowerment;
            let root_duration = entangle::constants::ROOT_DURATION * empowerment;
            for (entity, transform, _) in units_query.iter() {
                if transform.translation.distance(cursor_pos) <= radius {
                    commands.entity(entity).insert(RootedModifier::new(root_duration));
                }
            }
        }

        Spell::Sleep => {
            let radius = sleep::constants::CIRCLE_RADIUS * empowerment;
            let duration = sleep::constants::SLEEP_DURATION * empowerment;
            let bonus = sleep::constants::BONUS_DAMAGE_MULTIPLIER;
            for (entity, transform, _) in units_query.iter() {
                if transform.translation.distance(cursor_pos) <= radius {
                    commands.entity(entity).insert(SleepModifier::new(duration, bonus));
                }
            }
        }

        Spell::HypnoticPattern => {
            let radius = hypnotic_pattern::constants::CIRCLE_RADIUS * empowerment;
            let duration = hypnotic_pattern::constants::MESMERIZE_DURATION * empowerment;
            for (entity, transform, _) in units_query.iter() {
                if transform.translation.distance(cursor_pos) <= radius {
                    commands.entity(entity).insert(MesmerizedModifier::new(duration));
                }
            }
        }

        Spell::Haste => {
            let radius = haste::constants::CIRCLE_RADIUS * empowerment;
            let speed_mult = haste::constants::HASTE_MODIFIER;
            let duration = haste::constants::HASTE_DURATION * empowerment;
            for (entity, transform, team) in units_query.iter() {
                if *team == Team::Defenders && transform.translation.distance(cursor_pos) <= radius {
                    commands.entity(entity).insert(HasteModifier::new(speed_mult, duration));
                }
            }
        }

        Spell::BerserkerRage => {
            let radius = berserker_rage::constants::CIRCLE_RADIUS * empowerment;
            let damage_bonus = berserker_rage::constants::DAMAGE_BONUS;
            let vulnerability = berserker_rage::constants::DAMAGE_VULNERABILITY;
            let duration = berserker_rage::constants::BUFF_DURATION * empowerment;
            for (entity, transform, _) in units_query.iter() {
                if transform.translation.distance(cursor_pos) <= radius {
                    commands.entity(entity).insert(BerserkerRageModifier::new(damage_bonus, vulnerability, duration));
                }
            }
        }

        Spell::BattleHymn => {
            let radius = battle_hymn::constants::CIRCLE_RADIUS * empowerment;
            let damage_bonus = battle_hymn::constants::DAMAGE_BONUS;
            let attack_speed = battle_hymn::constants::ATTACK_SPEED_BONUS;
            let duration = battle_hymn::constants::BUFF_DURATION * empowerment;
            for (entity, transform, team) in units_query.iter() {
                if *team == Team::Defenders && transform.translation.distance(cursor_pos) <= radius {
                    commands.entity(entity).insert(BattleHymnModifier::new(damage_bonus, attack_speed, duration));
                }
            }
        }

        Spell::GuardianCircle => {
            let radius = guardian_circle::constants::CIRCLE_RADIUS * empowerment;
            let temp_hp = guardian_circle::constants::TEMP_HP_AMOUNT * empowerment;
            let duration = guardian_circle::constants::TEMP_HP_DURATION * empowerment;
            for (entity, transform, team) in units_query.iter() {
                if *team == Team::Defenders && transform.translation.distance(cursor_pos) <= radius {
                    commands.entity(entity).insert(TemporaryHitPoints::new(temp_hp, duration));
                }
            }
        }

        // ── Single-target spells ────────────────────────────────────

        Spell::Banishment => {
            if let Some((target_entity, _)) = units_query
                .iter()
                .filter(|(_, _, team)| **team == Team::Attackers || **team == Team::Undead)
                .filter_map(|(entity, transform, _)| {
                    let dist = transform.translation.distance(cursor_pos);
                    if dist <= banishment::constants::TARGET_SEARCH_RADIUS {
                        Some((entity, dist))
                    } else {
                        None
                    }
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            {
                let duration = banishment::constants::BANISH_DURATION * empowerment;
                commands.entity(target_entity).insert((
                    BanishedModifier::new(duration),
                    Visibility::Hidden,
                ));
            }
        }

        Spell::Polymorph => {
            if let Some((target_entity, target_material)) =
                polymorph_query
                    .iter()
                    .filter_map(|(entity, transform, material)| {
                        let dist = transform.translation.distance(cursor_pos);
                        if dist <= polymorph::constants::TARGET_SEARCH_RADIUS {
                            Some((entity, dist, material))
                        } else {
                            None
                        }
                    })
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .map(|(e, _, m)| (e, m))
            {
                let duration = polymorph::constants::POLYMORPH_DURATION * empowerment;
                let original_material = target_material.0.clone();
                let (current_hp, max_hp) = health_query
                    .get(target_entity)
                    .map(|(h, _)| (h.current, h.max))
                    .unwrap_or((1.0, 1.0));
                let sheep_material = materials.add(StandardMaterial {
                    base_color: polymorph::constants::SHEEP_COLOR,
                    ..default()
                });
                commands.entity(target_entity).insert((
                    PolymorphedModifier::new(
                        duration,
                        current_hp,
                        max_hp,
                        original_material,
                    ),
                    MeshMaterial3d(sheep_material),
                    Health::new(polymorph::constants::SHEEP_HP),
                ));
                commands.entity(target_entity).remove::<AttackTiming>();
            }
        }

        Spell::MarkOfDeath => {
            if let Some((target_entity, _)) = units_query
                .iter()
                .filter(|(_, _, team)| **team == Team::Attackers || **team == Team::Undead)
                .filter_map(|(entity, transform, _)| {
                    let dist = transform.translation.distance(cursor_pos);
                    if dist <= mark_of_death::constants::TARGET_SEARCH_RADIUS {
                        Some((entity, dist))
                    } else {
                        None
                    }
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            {
                for old_mark in existing_marks.iter() {
                    commands.entity(old_mark)
                        .remove::<MarkedForDeathModifier>()
                        .remove::<mark_of_death::components::ActiveMarkOfDeath>();
                }
                let amplification = mark_of_death::constants::DAMAGE_AMPLIFICATION * empowerment;
                let duration = mark_of_death::constants::MARK_DURATION * empowerment;
                commands.entity(target_entity).insert((
                    MarkedForDeathModifier::new(amplification, duration),
                    mark_of_death::components::ActiveMarkOfDeath,
                ));
            }
        }

        // ── Projectile/beam spells ──────────────────────────────────

        Spell::ChainLightning => {
            let target_pos_2d = Vec3::new(cursor_pos.x, 0.0, cursor_pos.z);
            if let Some((target_entity, target_pos)) = units_query
                .iter()
                .filter(|(_, transform, _)| {
                    let unit_pos_2d = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
                    target_pos_2d.distance(unit_pos_2d) <= chain_lightning::constants::TARGETING_RADIUS
                })
                .min_by(|a, b| {
                    let a_pos = Vec3::new(a.1.translation.x, 0.0, a.1.translation.z);
                    let b_pos = Vec3::new(b.1.translation.x, 0.0, b.1.translation.z);
                    target_pos_2d.distance(a_pos).partial_cmp(&target_pos_2d.distance(b_pos)).unwrap()
                })
                .map(|(e, t, _)| (e, t.translation))
            {
                let wizard_pos = wizard_transform
                    .map(|wt| wt.translation + Vec3::new(0.0, chain_lightning::constants::SPAWN_HEIGHT_OFFSET, 0.0))
                    .unwrap_or(cursor_pos);

                let initial_damage = chain_lightning::constants::INITIAL_DAMAGE * empowerment;

                if let Ok((mut health, mut temp_hp)) = health_query.get_mut(target_entity) {
                    apply_spell_damage(
                        commands, target_entity, &mut health, temp_hp.as_deref_mut(),
                        initial_damage, chain_lightning::constants::DAMAGE_TYPE,
                    );
                }

                chain_lightning::systems::spawn_arc(
                    commands, meshes, materials, wizard_pos, target_pos, 0, empowerment,
                );

                let group_entity = commands.spawn((
                    chain_lightning::components::ChainLightningGroup {
                        hit_entities: vec![target_entity],
                    },
                    OnGameplayScreen,
                )).id();

                commands.spawn((
                    chain_lightning::components::ChainLightningBolt {
                        group_entity,
                        current_damage: initial_damage * chain_lightning::constants::DAMAGE_FALLOFF,
                        damage_type: chain_lightning::constants::DAMAGE_TYPE,
                        bounces_remaining: chain_lightning::constants::MAX_BOUNCES,
                        last_hit_position: target_pos,
                        bounce_delay_timer: chain_lightning::constants::BOUNCE_DELAY * empowerment,
                        empowerment,
                        split_depth: 0,
                    },
                    OnGameplayScreen,
                ));
            }
        }

        Spell::FingerOfDeath => {
            if let Some(wt) = wizard_transform {
                let beam_origin = wt.translation + Vec3::new(0.0, finger_of_death::constants::BEAM_ORIGIN_HEIGHT_OFFSET, 0.0);
                let direction = (cursor_pos - beam_origin).normalize();
                let beam_length = (cursor_pos - beam_origin).length().min(finger_of_death::constants::BEAM_LENGTH);

                finger_of_death::systems::spawn_beam_with_marker(
                    commands, meshes, materials,
                    finger_of_death::components::FingerOfDeathBeam::new(
                        beam_origin, direction, beam_length, empowerment,
                    ),
                    true,
                );
            }
        }

        Spell::PhantasmalForce => {
            if let Some(assets) = infantry_assets {
                phantasmal_force::systems::spawn_decoys(
                    commands, cursor_pos, empowerment, assets,
                );
            }
        }

        // ── Telekinesis (guest-local, no host spawn needed) ─────────

        Spell::Telekinesis => {
            // Telekinesis picks up potion drops local to each player's cauldron.
        }

        // ── Teleport ────────────────────────────────────────────────

        Spell::Teleport => {
            // Teleport needs source + destination positions — not yet supported
            // via the SpellCast message format (would need a custom variant).
            bevy::log::info!("Guest teleport received at {:?}", cursor_pos);
        }

        // ── Channeled spells (handled via channel protocol) ─────────

        Spell::MagicMissile | Spell::Disintegrate | Spell::RaiseTheDead => {
            bevy::log::warn!("Unexpected SpellCast for channeled spell: {:?}", spell);
        }

        // ── Drag spells should not arrive as SpellCast ──────────────

        Spell::WallOfStone | Spell::WallOfFire => {
            bevy::log::warn!("Drag spell {:?} arrived as SpellCast", spell);
        }
    }
}

/// Spawns a drag-to-place spell entity on the host for the guest wizard.
#[allow(clippy::too_many_arguments)]
fn spawn_guest_drag_spell(
    spell: Spell,
    anchor: Vec3,
    end_pos: Vec3,
    empowerment: f32,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    use crate::game::units::wizard::spells::*;

    match spell {
        Spell::WallOfStone => {
            let diff = Vec3::new(end_pos.x - anchor.x, 0.0, end_pos.z - anchor.z);
            let length = diff.length();

            if length < wall_of_stone::constants::MIN_WALL_LENGTH {
                return;
            }

            let clamped_length = length.min(wall_of_stone::constants::MAX_WALL_LENGTH);
            let forward = diff.normalize();
            let right = Vec3::new(-forward.z, 0.0, forward.x);
            let center = anchor + forward * (clamped_length / 2.0);

            let scale = empowerment;
            let wall_width = wall_of_stone::constants::WALL_WIDTH * scale;
            let wall_height = wall_of_stone::constants::WALL_HEIGHT * scale;
            let wall_duration = wall_of_stone::constants::WALL_DURATION * scale;

            let wall_mesh = Cuboid::new(clamped_length, wall_of_stone::constants::WALL_HEIGHT, wall_of_stone::constants::WALL_WIDTH);
            let rotation = Quat::from_rotation_arc(Vec3::X, forward);

            commands.spawn((
                Mesh3d(meshes.add(wall_mesh)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: wall_of_stone::constants::WALL_COLOR,
                    ..default()
                })),
                Transform::from_xyz(center.x, wall_height / 2.0, center.z)
                    .with_rotation(rotation),
                wall_of_stone::components::WallOfStone {
                    center,
                    half_length: clamped_length / 2.0,
                    half_width: wall_width / 2.0,
                    forward,
                    right,
                    height: wall_height,
                    time_alive: 0.0,
                    duration: wall_duration,
                    sinking: false,
                    empowerment,
                },
                NetworkedSpellEffect { kind: SpellEffectKind::WallOfStone },
                OnGameplayScreen,
            ));

            // Notify pathfinding
            let unbuffered_min_x = center.x - forward.x * (clamped_length / 2.0) - right.x * (wall_width / 2.0);
            let unbuffered_max_x = center.x + forward.x * (clamped_length / 2.0) + right.x * (wall_width / 2.0);
            let unbuffered_min_z = center.z - forward.z * (clamped_length / 2.0) - right.z * (wall_width / 2.0);
            let unbuffered_max_z = center.z + forward.z * (clamped_length / 2.0) + right.z * (wall_width / 2.0);

            let min_x = unbuffered_min_x.min(unbuffered_max_x) - OBSTACLE_BUFFER;
            let max_x = unbuffered_min_x.max(unbuffered_max_x) + OBSTACLE_BUFFER;
            let min_z = unbuffered_min_z.min(unbuffered_max_z) - OBSTACLE_BUFFER;
            let max_z = unbuffered_min_z.max(unbuffered_max_z) + OBSTACLE_BUFFER;

            obstacle_events.write(ObstacleChanged {
                bounds: bevy::math::Rect::new(
                    min_x.min(max_x),
                    min_z.min(max_z),
                    (max_x - min_x).abs(),
                    (max_z - min_z).abs(),
                ),
                obstacle_type: ObstacleType::Blocked,
            });
        }

        Spell::WallOfFire => {
            let diff = Vec3::new(end_pos.x - anchor.x, 0.0, end_pos.z - anchor.z);
            let length = diff.length();

            if length < wall_of_fire::constants::MIN_WALL_LENGTH {
                return;
            }

            let clamped_length = length.min(wall_of_fire::constants::MAX_WALL_LENGTH);
            let forward = diff.normalize();
            let center = anchor + forward * (clamped_length / 2.0);

            let scale = empowerment;
            let half_width = wall_of_fire::constants::WALL_WIDTH * scale / 2.0;
            let damage = wall_of_fire::constants::DAMAGE_PER_TICK * scale;
            let fire_duration = wall_of_fire::constants::FIRE_DURATION * scale;

            let wall_height = 10.0;
            let wall_mesh = Cuboid::new(clamped_length, wall_height, wall_of_fire::constants::WALL_WIDTH);
            let rotation = Quat::from_rotation_arc(Vec3::X, forward);

            commands.spawn((
                Mesh3d(meshes.add(wall_mesh)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 0.5, 0.0, 0.4),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_xyz(center.x, wall_height / 2.0, center.z)
                    .with_rotation(rotation),
                wall_of_fire::components::WallOfFireEffect::new(
                    anchor,
                    end_pos,
                    half_width,
                    damage,
                    DamageType::Fire,
                    wall_of_fire::constants::TICK_INTERVAL,
                    fire_duration,
                ),
                NetworkedSpellEffect { kind: SpellEffectKind::WallOfFire },
                OnGameplayScreen,
            ));
        }

        _ => {
            bevy::log::warn!("Non-drag spell {:?} arrived as DragSpellCast", spell);
        }
    }
}
