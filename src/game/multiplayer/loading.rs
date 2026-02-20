//! Multiplayer loading systems.
//!
//! Builds and processes a multiplayer-specific spawn queue that sets up
//! dual castles, armies for both sides, and two wizards. The host spawns
//! all gameplay entities; the guest spawns only visual entities (units
//! will come from state snapshots in Milestone 4).

use bevy::prelude::*;

use crate::game::battlefield::components::{Battlefield, Castle};
use crate::game::battlefield::styles::{BATTLEFIELD_COLOR, CASTLE_COLOR};
use crate::game::components::Billboard;
use crate::game::constants::*;
use crate::game::loading::resources::LoadingProgress;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::archer::components::{ArcherMovementTimer, AttackRange};
use crate::game::units::archer::constants::{ARCHER_MAX_RANGE, ARCHER_MIN_RANGE, ARCHER_MOVEMENT_SPEED};
use crate::game::units::archer::styles::ARCHER_RADIUS;
use crate::game::units::archer::{Archer, ArcherAssets};
use crate::game::units::components::{
    AttackTiming, Effectiveness, FlockingVelocity, Health, Hitbox, MovementSpeed,
    TargetingVelocity, Team, Teleportable,
};
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::infantry::styles::UNIT_RADIUS;
use crate::game::units::infantry::Infantry;
use crate::game::units::king::components::KingSpawned;
use crate::game::units::random_position_in_cell;
use crate::game::units::wizard::components::*;
use crate::game::units::wizard::constants;
use crate::game::units::wizard::spells::magic_missile_constants;
use crate::game::units::wizard::styles::WIZARD_COLOR;
use crate::networking::protocol::NetworkMessage;
use crate::networking::resources::{NetworkConnection, PeerRole};
use crate::networking::session::MultiplayerSession;
use crate::state::AppState;

use super::components::OnMultiplayerGameScreen;

/// Multiplayer-specific spawn tasks.
pub enum MpSpawnTask {
    /// Spawn the battlefield ground plane and lighting.
    Battlefield,
    /// Spawn Castle 1 (host's castle).
    Castle1,
    /// Spawn Castle 2 (guest's castle).
    Castle2,
    /// Initialize the pathfinding grid.
    PathfindingGrid,
    /// Spawn host's wizard at Castle 1.
    HostWizard,
    /// Spawn guest's wizard at Castle 2.
    GuestWizard,
    /// Spawn a host-side defender infantry (near Castle 1).
    HostInfantry { unit_index: u32 },
    /// Spawn a host-side defender archer (near Castle 1).
    HostArcher { unit_index: u32 },
    /// Spawn a guest-side infantry (near Castle 2).
    GuestInfantry { unit_index: u32 },
    /// Spawn a guest-side archer (near Castle 2).
    GuestArcher { unit_index: u32 },
    /// Spawn the host-side King near Castle 1.
    HostKing,
    /// Spawn a host-side King's Guard unit.
    HostKingsGuard { guard_index: u32 },
    /// Spawn the guest-side King near Castle 2.
    GuestKing,
    /// Spawn a guest-side King's Guard unit.
    GuestKingsGuard { guard_index: u32 },
}

/// Resource that holds the multiplayer spawn queue.
#[derive(Resource)]
pub struct MpSpawnQueue {
    pub tasks: Vec<MpSpawnTask>,
}

impl MpSpawnQueue {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn pop_next(&mut self) -> Option<MpSpawnTask> {
        if self.tasks.is_empty() {
            None
        } else {
            Some(self.tasks.remove(0))
        }
    }

    pub fn is_complete(&self) -> bool {
        self.tasks.is_empty()
    }
}

/// Tracks whether both players have finished loading.
#[derive(Resource, Default)]
pub struct MpLoadingSync {
    pub my_loaded: bool,
    pub peer_loaded: bool,
}

/// Initializes the multiplayer loading spawn queue.
pub fn init_mp_loading(mut commands: Commands, session: Res<MultiplayerSession>) {
    commands.insert_resource(LoadingProgress::new());
    commands.insert_resource(MpLoadingSync::default());

    let mut queue = MpSpawnQueue::new();

    // Both host and guest spawn the visual world
    queue.tasks.push(MpSpawnTask::Battlefield);
    queue.tasks.push(MpSpawnTask::Castle1);
    queue.tasks.push(MpSpawnTask::Castle2);
    queue.tasks.push(MpSpawnTask::PathfindingGrid);

    // Host spawns all gameplay entities (both armies)
    if session.role == PeerRole::Host {
        // Host's King and Guard
        queue.tasks.push(MpSpawnTask::HostKing);
        for i in 0..KINGS_GUARD_COUNT {
            queue
                .tasks
                .push(MpSpawnTask::HostKingsGuard { guard_index: i });
        }

        // Guest's King and Guard
        queue.tasks.push(MpSpawnTask::GuestKing);
        for i in 0..KINGS_GUARD_COUNT {
            queue
                .tasks
                .push(MpSpawnTask::GuestKingsGuard { guard_index: i });
        }

        // Host-side army (near Castle 1)
        for i in 0..MP_INFANTRY_COUNT {
            queue.tasks.push(MpSpawnTask::HostInfantry { unit_index: i });
        }
        for i in 0..MP_ARCHER_COUNT {
            queue.tasks.push(MpSpawnTask::HostArcher { unit_index: i });
        }

        // Guest-side army (near Castle 2)
        for i in 0..MP_INFANTRY_COUNT {
            queue.tasks.push(MpSpawnTask::GuestInfantry { unit_index: i });
        }
        for i in 0..MP_ARCHER_COUNT {
            queue.tasks.push(MpSpawnTask::GuestArcher { unit_index: i });
        }
    }

    // Both spawn wizards (visual for guest, gameplay for host)
    queue.tasks.push(MpSpawnTask::HostWizard);
    queue.tasks.push(MpSpawnTask::GuestWizard);

    commands.insert_resource(queue);
}

/// Processes one multiplayer spawn task per frame.
#[allow(clippy::too_many_arguments)]
pub fn process_mp_spawn_queue(
    mut commands: Commands,
    mut loading_progress: ResMut<LoadingProgress>,
    mut spawn_queue: ResMut<MpSpawnQueue>,
    mut loading_sync: ResMut<MpLoadingSync>,
    mut connection: ResMut<NetworkConnection>,
    session: Res<MultiplayerSession>,
    infantry_assets: Res<InfantryAssets>,
    archer_assets: Res<ArcherAssets>,
    king_assets: Res<crate::game::units::king::resources::KingAssets>,
    mut king_spawned: ResMut<KingSpawned>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Process one task per frame
    if let Some(task) = spawn_queue.pop_next() {
        match task {
            MpSpawnTask::Battlefield => {
                spawn_mp_battlefield(&mut commands, &mut meshes, &mut materials);
            }
            MpSpawnTask::Castle1 => {
                spawn_castle(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    CASTLE_POSITION,
                    CASTLE_ROTATION_DEGREES,
                );
            }
            MpSpawnTask::Castle2 => {
                spawn_castle(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    CASTLE_2_POSITION,
                    CASTLE_2_ROTATION_DEGREES,
                );
            }
            MpSpawnTask::PathfindingGrid => {
                crate::game::pathfinding::systems::initialize_pathfinding(commands.reborrow());
            }
            MpSpawnTask::HostWizard => {
                spawn_mp_wizard(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    WIZARD_POSITION,
                    session.host_wizard,
                );
            }
            MpSpawnTask::GuestWizard => {
                spawn_mp_wizard(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    WIZARD_2_POSITION,
                    session.guest_wizard,
                );
            }
            MpSpawnTask::HostInfantry { unit_index } => {
                spawn_mp_infantry(
                    &mut commands,
                    &infantry_assets,
                    unit_index,
                    Team::Defenders,
                    true, // host side (Castle 1)
                );
            }
            MpSpawnTask::GuestInfantry { unit_index } => {
                spawn_mp_infantry(
                    &mut commands,
                    &infantry_assets,
                    unit_index,
                    Team::Attackers,
                    false, // guest side (Castle 2)
                );
            }
            MpSpawnTask::HostArcher { unit_index } => {
                spawn_mp_archer(
                    &mut commands,
                    &archer_assets,
                    unit_index,
                    Team::Defenders,
                    true,
                );
            }
            MpSpawnTask::GuestArcher { unit_index } => {
                spawn_mp_archer(
                    &mut commands,
                    &archer_assets,
                    unit_index,
                    Team::Attackers,
                    false,
                );
            }
            MpSpawnTask::HostKing => {
                spawn_mp_king(
                    &mut commands,
                    &king_assets,
                    &mut meshes,
                    &mut materials,
                    &mut king_spawned,
                    WIZARD_POSITION,
                    DEFENDER_GRID_CENTER_ANGLE,
                    Team::Defenders,
                );
            }
            MpSpawnTask::GuestKing => {
                let mirrored_angle = DEFENDER_GRID_CENTER_ANGLE + std::f32::consts::PI;
                spawn_mp_king(
                    &mut commands,
                    &king_assets,
                    &mut meshes,
                    &mut materials,
                    &mut king_spawned,
                    WIZARD_2_POSITION,
                    mirrored_angle,
                    Team::Attackers,
                );
            }
            MpSpawnTask::HostKingsGuard { guard_index } => {
                spawn_mp_kings_guard(
                    &mut commands,
                    &infantry_assets,
                    guard_index,
                    WIZARD_POSITION,
                    DEFENDER_GRID_CENTER_ANGLE,
                    Team::Defenders,
                );
            }
            MpSpawnTask::GuestKingsGuard { guard_index } => {
                let mirrored_angle = DEFENDER_GRID_CENTER_ANGLE + std::f32::consts::PI;
                spawn_mp_kings_guard(
                    &mut commands,
                    &infantry_assets,
                    guard_index,
                    WIZARD_2_POSITION,
                    mirrored_angle,
                    Team::Attackers,
                );
            }
        }
    }

    loading_progress.advance();

    // When queue is done and minimum frames reached, signal loaded
    if spawn_queue.is_complete() && loading_progress.is_complete() && !loading_sync.my_loaded {
        loading_sync.my_loaded = true;
        connection.outgoing_messages.push(NetworkMessage::GameLoaded);
    }

    // Check for peer's GameLoaded message
    let has_game_loaded = connection
        .incoming_messages
        .iter()
        .any(|m| matches!(m, NetworkMessage::GameLoaded));

    if has_game_loaded {
        connection
            .incoming_messages
            .retain(|m| !matches!(m, NetworkMessage::GameLoaded));
        loading_sync.peer_loaded = true;
    }

    // Both loaded — transition to gameplay
    if loading_sync.my_loaded && loading_sync.peer_loaded {
        next_state.set(AppState::MultiplayerGame);
    }
}

/// Cleans up multiplayer loading resources.
pub fn cleanup_mp_loading(mut commands: Commands) {
    commands.remove_resource::<LoadingProgress>();
    commands.remove_resource::<MpSpawnQueue>();
    commands.remove_resource::<MpLoadingSync>();
}

/// Sets up the camera for the multiplayer game based on role.
///
/// Host uses the standard camera position; guest gets a mirrored view.
pub fn setup_mp_camera(
    session: Res<MultiplayerSession>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    if let Ok(mut transform) = camera_query.single_mut() {
        if session.role == PeerRole::Guest {
            // Mirrored camera: opposite corner looking at origin
            *transform = Transform::from_xyz(1000.0, 2500.0, -2500.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
        }
        // Host keeps the default camera position from main.rs
    }
}

/// Restores the default camera when leaving multiplayer.
pub fn restore_camera(mut camera_query: Query<&mut Transform, With<Camera3d>>) {
    if let Ok(mut transform) = camera_query.single_mut() {
        *transform = Transform::from_xyz(-1000.0, 2500.0, 2500.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
    }
}

// ──────────────────────────────────────────────────────────────────
// Spawn helpers
// ──────────────────────────────────────────────────────────────────

/// Spawns the battlefield ground plane and lighting for multiplayer.
fn spawn_mp_battlefield(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Light
    commands.spawn((
        PointLight {
            intensity: 2_000_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1000.0, 0.0),
        OnMultiplayerGameScreen,
    ));

    // Ground plane
    let battlefield_mesh = Plane3d::default()
        .mesh()
        .size(BATTLEFIELD_SIZE, BATTLEFIELD_SIZE);
    commands.spawn((
        Mesh3d(meshes.add(battlefield_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: BATTLEFIELD_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Battlefield,
        OnMultiplayerGameScreen,
    ));
}

/// Spawns a castle box at the given position and rotation.
/// `position` is the top surface; the box extends down to battlefield level.
fn spawn_castle(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    rotation_degrees: f32,
) {
    let castle_box = Cuboid::new(CASTLE_WIDTH, CASTLE_HEIGHT, CASTLE_DEPTH);
    let castle_center = position - Vec3::new(0.0, CASTLE_HEIGHT / 2.0, 0.0);
    commands.spawn((
        Mesh3d(meshes.add(castle_box)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: CASTLE_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(castle_center)
            .with_rotation(Quat::from_rotation_y(rotation_degrees.to_radians())),
        Castle,
        OnMultiplayerGameScreen,
    ));
}

/// Spawns a wizard at the given position for multiplayer.
fn spawn_mp_wizard(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    wizard_type: crate::config::WizardType,
) {
    let hitbox = Hitbox::new(constants::HITBOX_RADIUS, constants::HITBOX_HEIGHT);
    let wizard_width = hitbox.sprite_width();
    let wizard_height = hitbox.sprite_height();
    let wizard_triangle = Triangle2d::new(
        Vec2::new(0.0, wizard_height / 2.0),
        Vec2::new(-wizard_width / 2.0, -wizard_height / 2.0),
        Vec2::new(wizard_width / 2.0, -wizard_height / 2.0),
    );

    let mut wizard = Wizard::new(constants::DEFAULT_SPELL_RANGE);
    if wizard_type == crate::config::WizardType::BoringOleMage {
        wizard.spell_range = constants::DEFAULT_SPELL_RANGE * 1.05;
        wizard.mana_cost_multiplier = 0.95;
        wizard.spell_power_multiplier = 1.05;
        wizard.cast_speed_multiplier = 1.05;
    }

    let mut entity_commands = commands.spawn((
        Mesh3d(meshes.add(wizard_triangle)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: WIZARD_COLOR,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(position),
        hitbox,
        Health::new(constants::HEALTH),
        MovementSpeed(0.0),
        Mana::new(constants::MANA),
        ManaRegen::new(constants::MANA_REGEN),
        CastingState::new(),
        wizard,
        magic_missile_constants::PRIMED_MAGIC_MISSILE,
        Billboard,
        OnMultiplayerGameScreen,
    ));

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
fn spawn_mp_infantry(
    commands: &mut Commands,
    infantry_assets: &InfantryAssets,
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
                calculate_defender_grid_position(*row, *col)
            } else {
                calculate_guest_defender_grid_position(*row, *col)
            };
            let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

            let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;
            let spawn_pos = Vec2::new(spawn_x, spawn_z);

            let material = if host_side {
                infantry_assets.defender_material.clone()
            } else {
                infantry_assets.attacker_material.clone()
            };

            let flow_field = if host_side {
                FlowFieldInfluence::Defender { spawn_pos }
            } else {
                FlowFieldInfluence::Attacker
            };

            commands
                .spawn((
                    Mesh3d(infantry_assets.mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    crate::game::components::Velocity::default(),
                    crate::game::components::Acceleration::new(),
                    hitbox,
                    Health::new(UNIT_HEALTH),
                    MovementSpeed(UNIT_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    team,
                    Infantry,
                ))
                .insert((
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    FlowFieldVelocity::default(),
                    flow_field,
                    Teleportable,
                    Billboard,
                    OnMultiplayerGameScreen,
                ));
            return;
        }
        units_counted += units_in_this_cell;
    }
}

/// Spawns a single archer unit for multiplayer.
fn spawn_mp_archer(
    commands: &mut Commands,
    archer_assets: &ArcherAssets,
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
                calculate_defender_grid_position(archer_row, cell_idx)
            } else {
                calculate_guest_defender_grid_position(archer_row, cell_idx)
            };
            let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

            let hitbox = Hitbox::new(ARCHER_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;

            let material = if host_side {
                archer_assets.defender_material.clone()
            } else {
                archer_assets.attacker_material.clone()
            };

            let flow_field = if host_side {
                FlowFieldInfluence::Defender {
                    spawn_pos: Vec2::new(spawn_x, spawn_z),
                }
            } else {
                FlowFieldInfluence::Attacker
            };

            commands
                .spawn((
                    Mesh3d(archer_assets.mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    crate::game::components::Velocity::default(),
                    crate::game::components::Acceleration::new(),
                    hitbox,
                    Health::new(UNIT_HEALTH),
                    MovementSpeed(ARCHER_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    team,
                    Archer,
                ))
                .insert((
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
            return;
        }
        units_counted += units_in_this_cell;
    }
}

/// Spawns a King unit at the given position origin for multiplayer.
fn spawn_mp_king(
    commands: &mut Commands,
    king_assets: &crate::game::units::king::resources::KingAssets,
    meshes: &mut ResMut<Assets<Mesh>>,
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

    let radius = DEFENDER_GRID_GROUND_RANGE + 600.0;
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

    let king_entity = commands
        .spawn((
            Mesh3d(king_assets.mesh.clone()),
            MeshMaterial3d(king_assets.material.clone()),
            Transform::from_xyz(spawn_x, spawn_y, spawn_z),
            crate::game::components::Velocity::default(),
            crate::game::components::Acceleration::new(),
            hitbox,
            Health::new(KING_HEALTH),
            MovementSpeed(KING_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            DamageMultiplier(KING_DAMAGE_PERCENTAGE),
            team,
            King,
        ))
        .insert((
            Commander {
                aura_radius: KING_AURA_RADIUS,
                team_filter,
                visual_color: KING_AURA_COLOR,
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
            Teleportable,
            crate::game::units::components::FlockingModifier::new(1.0, 0.0, 0.0),
            Billboard,
            OnMultiplayerGameScreen,
        ))
        .id();

    // Visual aura circle
    let aura_circle = Circle::new(KING_AURA_RADIUS);
    let y_offset = 5.0 - spawn_y;
    let aura_entity = commands
        .spawn((
            Mesh3d(meshes.add(aura_circle)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: KING_AURA_COLOR,
                unlit: true,
                alpha_mode: bevy::prelude::AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, y_offset, 0.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            OnMultiplayerGameScreen,
        ))
        .id();
    commands.entity(king_entity).add_child(aura_entity);

    king_spawned.0 = true;
}

/// Spawns a King's Guard unit for multiplayer at the given position origin.
fn spawn_mp_kings_guard(
    commands: &mut Commands,
    infantry_assets: &InfantryAssets,
    guard_index: u32,
    wizard_position: Vec3,
    center_angle: f32,
    team: Team,
) {
    use crate::game::units::components::KingsGuard;
    use crate::game::units::elite::{EliteDamageBonus, EliteHealthBonus, EliteSpeedBonus};

    // King's position: same calculation as spawn_mp_king
    let radius = DEFENDER_GRID_GROUND_RANGE + 600.0;
    let king_x = wizard_position.x + radius * center_angle.cos();
    let king_z = wizard_position.z + radius * center_angle.sin();

    let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let angle = guard_index as f32 * (std::f32::consts::TAU / KINGS_GUARD_COUNT as f32);
    let final_x = king_x + KINGS_GUARD_ORBIT_RADIUS * angle.cos();
    let final_z = king_z + KINGS_GUARD_ORBIT_RADIUS * angle.sin();

    commands
        .spawn((
            Mesh3d(infantry_assets.mesh.clone()),
            MeshMaterial3d(infantry_assets.kings_guard_material.clone()),
            Transform::from_xyz(final_x, spawn_y, final_z),
            hitbox,
            Health::new(UNIT_HEALTH),
            AttackTiming::new(),
            Effectiveness::new(),
            team,
            Infantry,
            KingsGuard(guard_index),
        ))
        .insert((
            Teleportable,
            Billboard,
            OnMultiplayerGameScreen,
            EliteHealthBonus(crate::game::units::elite::ELITE_HEALTH_BONUS),
            EliteDamageBonus(crate::game::units::elite::ELITE_DAMAGE_BONUS),
            EliteSpeedBonus(crate::game::units::elite::ELITE_SPEED_BONUS),
        ));
}
