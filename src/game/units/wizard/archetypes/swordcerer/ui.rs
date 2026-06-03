//! Swordcerer UI: cooldowns, health bar, enter-fray button, location click.

use super::components::*;
use super::constants::*;
use super::messages::*;
use super::resources::{SwordcererAssets, SwordcererPhase, SwordcererState};
use crate::config::GameConfig;
use crate::game::components::{Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::WIZARD_POSITION;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ArchetypeUI;
use crate::game::input::messages::{MouseClicked, SpacebarPressed};
use crate::game::units::components::{
    AttackTiming, Effectiveness, FacingDirection, Health, Hitbox, MovementSpeed, Team,
    WalkingAnimation,
};
use crate::game::units::systems::create_default_sprite_material;
use crate::game::units::wizard::components::{LocalWizard, Wizard};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::get_cursor_world_position;
use bevy::prelude::*;

/// Initialize or reset the swordcerer state resource and cached assets.
/// Ticks swordcerer cooldowns.
pub(super) fn tick_cooldowns(
    time: Res<Time>,
    mut commands: Commands,
    mut missile_cooldowns: Query<(Entity, &mut SwordcererMissileCooldown)>,
    mut sword_cooldowns: Query<(Entity, &mut SwordcererSwordCooldown)>,
) {
    for (entity, mut cd) in &mut missile_cooldowns {
        cd.remaining -= time.delta_secs();
        if cd.remaining <= 0.0 {
            commands
                .entity(entity)
                .remove::<SwordcererMissileCooldown>();
        }
    }
    for (entity, mut cd) in &mut sword_cooldowns {
        cd.remaining -= time.delta_secs();
        if cd.remaining <= 0.0 {
            commands.entity(entity).remove::<SwordcererSwordCooldown>();
        }
    }
}

/// Checks if the swordcerer avatar has died and triggers retreat.
pub(super) fn check_avatar_death(
    avatar_query: Query<&Health, (With<SwordcererAvatar>, Without<GuestControlledAvatar>)>,
    state: Res<SwordcererState>,
    mut retreat_messages: MessageWriter<RetreatMessage>,
) {
    if state.phase != SwordcererPhase::OnField {
        return;
    }

    // The host's OWN avatar (the guest's is `GuestControlledAvatar` and handled
    // by `check_guest_avatar_death`).
    if let Some(health) = avatar_query.iter().next()
        && health.current <= 0.0
    {
        retreat_messages.write(RetreatMessage);
    }
}

/// Spawns the swordcerer health bar UI when the avatar is on the field.
pub(super) fn spawn_health_bar(
    mut commands: Commands,
    state: Res<SwordcererState>,
    existing_bar: Query<Entity, With<SwordcererHealthBar>>,
) {
    // Only spawn when transitioning to OnField and no bar exists
    if !state.is_changed() || state.phase != SwordcererPhase::OnField || !existing_bar.is_empty() {
        return;
    }

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(80.0),
                left: Val::Percent(50.0),
                width: Val::Px(200.0),
                height: Val::Px(16.0),
                margin: UiRect::left(Val::Px(-100.0)), // center
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            BorderColor::all(Color::srgba(0.3, 0.3, 0.8, 1.0)),
            SwordcererHealthBar,
            OnGameplayScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.4, 1.0)),
                SwordcererHealthBarFill,
            ));
        });
}

/// Updates the swordcerer health bar fill. Reads the host's own `SwordcererAvatar`
/// OR (on the guest) the `GhostSwordcererAvatar` mirrored from the host snapshot,
/// so the guest sees its host-owned avatar's HP.
pub(super) fn update_health_bar(
    avatar_query: Query<
        &Health,
        (
            Or<(
                With<SwordcererAvatar>,
                With<crate::game::multiplayer::components::GhostSwordcererAvatar>,
            )>,
            Without<GuestControlledAvatar>,
        ),
    >,
    mut fill_query: Query<&mut Node, With<SwordcererHealthBarFill>>,
) {
    // `.iter().next()` (not `single()`) so a Swordcerer-vs-Swordcerer match,
    // where two avatars can match, never freezes the bar.
    if let Some(health) = avatar_query.iter().next()
        && let Ok(mut node) = fill_query.single_mut()
    {
        let pct = (health.current / health.max * 100.0).clamp(0.0, 100.0);
        node.width = Val::Percent(pct);
    }
}

/// Despawns the health bar when the avatar is no longer on the field.
pub(super) fn despawn_health_bar(
    mut commands: Commands,
    state: Res<SwordcererState>,
    bar_query: Query<Entity, With<SwordcererHealthBar>>,
) {
    if state.is_changed() && state.phase != SwordcererPhase::OnField {
        for entity in &bar_query {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Spawns the "Enter the Fray" button UI at the bottom center of the screen.
pub(crate) fn spawn_enter_fray_button(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(FRAY_BUTTON_BOTTOM),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            EnterFrayRoot,
            OnGameplayScreen,
            ArchetypeUI,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.4, 0.5, 1.0, 1.0)),
                    BackgroundColor(Color::srgba(0.15, 0.15, 0.4, 0.9)),
                    EnterFrayButton,
                ))
                .with_child((
                    Text::new(FRAY_BUTTON_TEXT),
                    TextFont::from_font_size(FRAY_BUTTON_FONT_SIZE),
                    TextColor(Color::srgba(0.8, 0.85, 1.0, 1.0)),
                    EnterFrayButtonText,
                ));
        });
}

/// Handles clicks on the "Enter the Fray" button — transitions to location choosing.
pub(super) fn handle_enter_fray_click(
    mut button_clicked: MessageReader<MouseClicked>,
    fray_button_query: Query<Entity, With<EnterFrayButton>>,
    mut state: ResMut<SwordcererState>,
    mut text_query: Query<&mut Text, With<EnterFrayButtonText>>,
) {
    for event in button_clicked.read() {
        if fray_button_query.get(event.button).is_ok() {
            toggle_enter_fray(&mut state, &mut text_query);
        }
    }
}

/// Activate-key / South-button equivalent of `handle_enter_fray_click`.
pub(super) fn handle_enter_fray_hotkey(
    mut spacebar_pressed: MessageReader<SpacebarPressed>,
    mut state: ResMut<SwordcererState>,
    mut text_query: Query<&mut Text, With<EnterFrayButtonText>>,
) {
    if spacebar_pressed.read().next().is_none() {
        return;
    }
    if !matches!(
        state.phase,
        SwordcererPhase::Idle | SwordcererPhase::ChoosingLocation
    ) {
        return;
    }
    toggle_enter_fray(&mut state, &mut text_query);
}

fn toggle_enter_fray(
    state: &mut SwordcererState,
    text_query: &mut Query<&mut Text, With<EnterFrayButtonText>>,
) {
    if state.phase == SwordcererPhase::Idle && !state.retreated {
        state.phase = SwordcererPhase::ChoosingLocation;
        if let Ok(mut text) = text_query.single_mut() {
            **text = FRAY_BUTTON_TEXT_CHOOSING.to_string();
        }
    } else if state.phase == SwordcererPhase::ChoosingLocation {
        // Activating again cancels.
        state.phase = SwordcererPhase::Idle;
        if let Ok(mut text) = text_query.single_mut() {
            **text = FRAY_BUTTON_TEXT.to_string();
        }
    }
}

/// Handles left-click / RT on the battlefield to place the avatar during ChoosingLocation phase.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_location_click(
    mut mouse_pressed: MessageReader<crate::game::input::messages::MouseLeftPressed>,
    mut state: ResMut<SwordcererState>,
    mut commands: Commands,
    // Filter to the LOCAL wizard: MP spawns two `Wizard` entities (host + guest)
    // on every peer, so a bare `With<Wizard>` single query errors and the click
    // silently no-ops.
    mut wizard_query: Query<&mut Visibility, (With<Wizard>, With<LocalWizard>)>,
    swordcerer_assets: Res<SwordcererAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
    camera_query_3d: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut text_query: Query<&mut Text, With<EnterFrayButtonText>>,
    session: Option<Res<crate::networking::session::MultiplayerSession>>,
    mut connection: Option<ResMut<crate::networking::resources::NetworkConnection>>,
) {
    use crate::networking::protocol::NetworkMessage;
    use crate::networking::resources::PeerRole;
    if state.phase != SwordcererPhase::ChoosingLocation {
        return;
    }
    if mouse_pressed.read().next().is_none() {
        return;
    }

    let Some(world_pos) = get_cursor_world_position(&camera_query_3d, &corrected_cursor) else {
        return;
    };

    let Ok(mut visibility) = wizard_query.single_mut() else {
        return;
    };

    // Play teleport sound and hide wizard
    audio::play_sfx(
        &mut commands,
        &sfx.teleport_cast,
        WIZARD_POSITION,
        &config,
        &sfx,
    );
    *visibility = Visibility::Hidden;

    // Deploy the avatar. The avatar is host-authoritative: single-player and the
    // multiplayer HOST spawn it locally (Defenders); the multiplayer GUEST is not
    // authoritative over its own avatar, so it asks the host to spawn it on the
    // guest's army (Attackers) and then streams control input.
    let is_mp_guest = session
        .as_ref()
        .map(|s| s.role == PeerRole::Guest)
        .unwrap_or(false);
    if is_mp_guest {
        if let Some(connection) = connection.as_mut() {
            connection
                .outgoing_messages
                .push(NetworkMessage::SwordcererAvatarSpawn {
                    x: world_pos.x,
                    z: world_pos.z,
                });
        }
    } else {
        spawn_swordcerer_avatar(
            &mut commands,
            &swordcerer_assets,
            &mut materials,
            world_pos,
            Team::Defenders,
        );
    }

    state.phase = SwordcererPhase::OnField;

    // Reset button text
    if let Ok(mut text) = text_query.single_mut() {
        **text = FRAY_BUTTON_TEXT.to_string();
    }
}

/// Spawns the Swordcerer battlefield avatar with the given team. Shared by the
/// local deploy (`handle_location_click`) and the host's spawn of a guest's
/// avatar (`receive_swordcerer_spawn`).
pub(super) fn spawn_swordcerer_avatar(
    commands: &mut Commands,
    swordcerer_assets: &SwordcererAssets,
    materials: &mut Assets<StandardMaterial>,
    world_pos: Vec3,
    team: Team,
) -> Entity {
    let hitbox = Hitbox::new(AVATAR_HITBOX_RADIUS, AVATAR_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let material = create_default_sprite_material(
        materials,
        swordcerer_assets.sprite_texture.clone(),
        AVATAR_SPRITE_TINT,
    );

    commands
        .spawn((
            Mesh3d(swordcerer_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(world_pos.x, spawn_y, world_pos.z),
            Velocity::default(),
            hitbox,
            Health::new(AVATAR_HEALTH),
            MovementSpeed(AVATAR_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            team,
            // 75% melee reduction — the avatar fights at melee range; without
            // this it shreds in a few hits.
            crate::game::units::components::MeleeDamageReduction {
                multiplier: AVATAR_MELEE_DAMAGE_MULTIPLIER,
            },
            (
                SwordcererAvatar,
                SwordcererFacing::default(),
                WalkingAnimation::default(),
                FacingDirection::default(),
                Billboard,
                OnGameplayScreen,
            ),
        ))
        .id()
}

/// Shows/hides the "Enter the Fray" button based on swordcerer state.
pub(super) fn update_enter_fray_visibility(
    state: Res<SwordcererState>,
    mut root_query: Query<&mut Visibility, With<EnterFrayRoot>>,
) {
    if !state.is_changed() {
        return;
    }
    let visible = matches!(
        state.phase,
        SwordcererPhase::Idle | SwordcererPhase::ChoosingLocation
    ) && !state.retreated;
    for mut vis in &mut root_query {
        *vis = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
