//! UI layout helpers: panels, scroll areas, escape handlers, button spawn, slider rows.

use super::button_systems::{edge_color, insert_material_background, opaque};
use bevy::ecs::relationship::Relationship;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, ShadowStyle};

use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui::RelativeCursorPosition;

use super::components::{ButtonAnimState, ButtonColors, ButtonEdge, ButtonFront, ButtonStyle};
use super::constants::{
    BUTTON_3D_OFFSET_REST, BUTTON_REST_OUTLINE, BUTTON_SHADOW_COLOR, CONTENT_BG, CONTENT_BORDER,
    DETAIL_BG, DETAIL_BORDER, DETAIL_PADDING, DOT_LEADER_COLOR, FRAME_OUTER_RING_COLOR,
    FRAME_OUTLINE_COLOR, FRAME_OUTLINE_OFFSET, FRAME_OUTLINE_WIDTH, FRAME_SHADOW_SPREAD_BASE,
    LEFT_PANEL_WIDTH, LIST_BG, LIST_BORDER, OVERLAY_BG, PANEL_BORDER_RADIUS, PANEL_SHADOW_AMBIENT,
    PANEL_SHADOW_CONTACT, PANEL_SHADOW_MEDIUM, SCROLL_BG, SCROLL_BORDER, SCROLL_SHADOW_COLOR,
    SHADOW_COLOR, SLIDER_BORDER_WIDTH, SLIDER_BUTTON_BG, SLIDER_BUTTON_BORDER_COLOR,
    SLIDER_BUTTON_FONT_SIZE, SLIDER_BUTTON_SIZE, SLIDER_GAP, SLIDER_LABEL_FONT_SIZE,
    SLIDER_TRACK_BG, SLIDER_TRACK_WIDTH, TEXT_PRIMARY, TEXT_SHADOW_COLOR,
};
use super::focus::Focusable;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::MouseButtonState;
use crate::game::input::gamepad::messages::MenuBackPressed;
use crate::state::{InGameState, MenuState, MultiplayerGameState, PauseMenuState};

/// Marker for page content panels that should receive a parchment background.
#[derive(Component)]
pub(crate) struct ParchmentPanel;

/// Marker for overlay roots that should receive a frosted glass background.
#[derive(Component)]
pub(crate) struct FrostedGlassOverlay;

/// Scales a font size down based on text width to fit within a constrained area.
///
/// Returns `base_font` when `max_width <= min_chars`, scaling linearly down to
/// `base_font * min_scale` when `max_width >= max_chars`.
pub fn apply_parchment_backgrounds(
    mut commands: Commands,
    new_panels: Query<Entity, Added<ParchmentPanel>>,
    mut materials: ResMut<Assets<ParchmentMaterial>>,
) {
    for entity in &new_panels {
        let mat = materials.add(ParchmentMaterial::new(CONTENT_BG));
        insert_material_background(&mut commands, entity, mat);
    }
}

/// Applies a frosted glass material to newly spawned overlay roots.
pub fn apply_frosted_glass_overlays(
    mut commands: Commands,
    new_overlays: Query<Entity, Added<FrostedGlassOverlay>>,
    mut materials: ResMut<Assets<FrostedGlassMaterial>>,
) {
    for entity in &new_overlays {
        let mat = materials.add(FrostedGlassMaterial::new());
        insert_material_background(&mut commands, entity, mat);
    }
}

// ---------------------------------------------------------------------------
// Page container (shared by settings, progress, instructions, and overlays)
// ---------------------------------------------------------------------------

/// Returns the shared styling bundle for inner scrollable areas, with
/// scroll-area border-radius baked into the supplied `Node`. Callers add
/// `ScrollPosition::default()` and their marker component alongside.
pub(crate) fn scroll_area_style(mut node: Node) -> (Node, BackgroundColor, BorderColor, BoxShadow) {
    node.border_radius = BorderRadius::all(Val::Px(4.0));
    (
        node,
        BackgroundColor(SCROLL_BG),
        BorderColor::all(SCROLL_BORDER),
        BoxShadow(vec![
            ShadowStyle {
                color: SCROLL_SHADOW_COLOR,
                x_offset: Val::Px(0.0),
                y_offset: Val::Px(1.0),
                spread_radius: Val::Px(0.0),
                blur_radius: Val::Px(2.0),
            },
            ShadowStyle {
                color: SHADOW_COLOR,
                x_offset: Val::Px(0.0),
                y_offset: Val::Px(3.0),
                spread_radius: Val::Px(1.0),
                blur_radius: Val::Px(6.0),
            },
        ]),
    )
}

/// Standard content node for page containers (column, centered, with scroll clipping).
pub fn default_content_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(20.0)),
        border: UiRect::all(Val::Px(1.0)),
        overflow: Overflow::clip(),
        ..default()
    }
}

/// Spawns the standard left detail panel (300px fixed, gold-bordered box).
/// Returns the **inner detail box** entity — add your content as children of this.
pub fn spawn_left_detail_panel(parent: &mut ChildSpawnerCommands) -> Entity {
    let mut detail_box_id = Entity::PLACEHOLDER;
    parent
        .spawn(Node {
            width: Val::Px(LEFT_PANEL_WIDTH),
            flex_direction: FlexDirection::Column,
            align_self: AlignSelf::Center,
            row_gap: Val::Px(16.0),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|left| {
            detail_box_id = left
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(DETAIL_PADDING)),
                        row_gap: Val::Px(12.0),
                        border: UiRect::all(Val::Px(1.0)),
                        flex_grow: 1.0,
                        border_radius: BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(DETAIL_BG),
                    BorderColor::all(DETAIL_BORDER),
                    Outline::new(
                        Val::Px(FRAME_OUTLINE_WIDTH),
                        Val::Px(1.0),
                        FRAME_OUTLINE_COLOR,
                    ),
                ))
                .id();
        });
    detail_box_id
}

/// Like `spawn_left_detail_panel`, but makes the detail box scrollable.
/// Inserts the given `marker` component and `ScrollPosition` for scroll handling.
pub fn spawn_scrollable_left_detail_panel<M: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
) -> Entity {
    let detail_box = spawn_left_detail_panel(parent);
    parent.commands().entity(detail_box).insert((
        marker,
        ScrollPosition::default(),
        crate::ui::focus::GamepadScrollTarget,
    ));
    parent
        .commands()
        .entity(detail_box)
        .entry::<Node>()
        .and_modify(|mut node| {
            node.overflow = Overflow::scroll_y();
        });
    detail_box
}

/// Spawns the standard right scrollable panel (flex-grow, dark background with scroll).
/// `marker` is attached for screen-specific queries (e.g. scroll handling).
/// Returns the scrollable content entity — add your content as children of this.
pub fn spawn_right_scroll_panel<M: Component>(
    parent: &mut ChildSpawnerCommands,
    marker: M,
    inner_direction: FlexDirection,
    inner_gap: f32,
) -> Entity {
    parent
        .spawn((
            Node {
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                min_width: Val::Px(0.0),
                // Required: flex's implicit `min-height: auto` would let this
                // item grow to its content size, defeating `scroll_y()`.
                min_height: Val::Px(0.0),
                flex_direction: inner_direction,
                row_gap: Val::Px(inner_gap),
                column_gap: Val::Px(inner_gap),
                overflow: Overflow::scroll_y(),
                // Default `Stretch` would force children to panel height,
                // again defeating scroll_y.
                align_items: AlignItems::FlexStart,
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(12.0)),
                border_radius: BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
                ..default()
            },
            BackgroundColor(LIST_BG),
            BorderColor::all(LIST_BORDER),
            ScrollPosition::default(),
            crate::ui::focus::GamepadScrollTarget,
            marker,
        ))
        .id()
}

/// Spawns a full-screen page with a semi-transparent overlay and a styled
/// content container inside it. Returns the content container entity so the
/// caller can add children.
///
/// `content_node` controls the inner container layout (flex direction, padding,
/// gaps, overflow, etc.). Use `default_content_node()` for the standard look.
/// The border color, background, border-radius, and shadow are applied
/// automatically.
///
/// When `pause_menu` is true, `GlobalZIndex(500)` is added so the page
/// renders above in-game UI.
pub fn spawn_page_container<M: Component>(
    commands: &mut Commands,
    screen_marker: M,
    pause_menu: bool,
    mut content_node: Node,
) -> Entity {
    content_node.border_radius = BorderRadius::all(Val::Px(6.0));
    let mut root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(OVERLAY_BG),
        // Make the overlay block all interactions behind it.
        Interaction::default(),
        bevy::ui::FocusPolicy::Block,
        FrostedGlassOverlay,
        screen_marker,
    ));

    // Modal overlays sit above the in-game HUD (100..). Pause goes highest
    // since it can open on top of the spell book / cauldron menus.
    root.insert(GlobalZIndex(if pause_menu { 1000 } else { 500 }));

    let root_id = root.id();

    let content_id = commands
        .spawn((
            content_node,
            BackgroundColor(CONTENT_BG),
            BorderColor::all(CONTENT_BORDER),
            // Middle ring via outline with gap
            Outline::new(
                Val::Px(FRAME_OUTLINE_WIDTH),
                Val::Px(FRAME_OUTLINE_OFFSET),
                FRAME_OUTLINE_COLOR,
            ),
            BoxShadow(vec![
                // Outermost solid ring (zero blur = solid border)
                ShadowStyle {
                    color: FRAME_OUTER_RING_COLOR,
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(0.0),
                    spread_radius: Val::Px(FRAME_SHADOW_SPREAD_BASE),
                    blur_radius: Val::Px(0.0),
                },
                // Tight contact shadow
                ShadowStyle {
                    color: PANEL_SHADOW_CONTACT,
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(1.0),
                    spread_radius: Val::Px(FRAME_SHADOW_SPREAD_BASE),
                    blur_radius: Val::Px(3.0),
                },
                // Medium depth shadow
                ShadowStyle {
                    color: PANEL_SHADOW_MEDIUM,
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(4.0),
                    spread_radius: Val::Px(FRAME_SHADOW_SPREAD_BASE + 2.0),
                    blur_radius: Val::Px(8.0),
                },
                // Wide ambient shadow
                ShadowStyle {
                    color: PANEL_SHADOW_AMBIENT,
                    x_offset: Val::Px(0.0),
                    y_offset: Val::Px(8.0),
                    spread_radius: Val::Px(FRAME_SHADOW_SPREAD_BASE + 4.0),
                    blur_radius: Val::Px(20.0),
                },
            ]),
        ))
        .id();

    commands.entity(root_id).add_child(content_id);

    content_id
}

// ---------------------------------------------------------------------------
// Shared Escape key handling
// ---------------------------------------------------------------------------

/// Generic "Back" button-click handler returning to the main-menu landing screen.
/// Parameterized by the screen's own back-button marker component `B` so each
/// screen (compendium, manual, …) reuses one implementation.
pub(crate) fn handle_main_menu_back_button<B: Component>(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    button_query: Query<&B>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            channel_change.write(ChannelChangeMessage);
            next_state.set(MenuState::Landing);
        }
    }
}

/// Generic "Back" button-click handler returning to the pause-menu main screen.
pub(crate) fn handle_pause_menu_back_button<B: Component>(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    button_query: Query<&B>,
    mut next_state: ResMut<NextState<PauseMenuState>>,
) {
    for event in button_clicked.read() {
        if button_query.get(event.button).is_ok() {
            next_state.set(PauseMenuState::Main);
        }
    }
}

/// Handles Escape key / gamepad back to return to the main menu landing screen.
pub fn escape_to_landing(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut back_msgs: MessageReader<MenuBackPressed>,
    mut next_state: ResMut<NextState<MenuState>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    let back_pressed = back_msgs.read().next().is_some();
    if keyboard.just_pressed(KeyCode::Escape) || back_pressed {
        channel_change.write(ChannelChangeMessage);
        next_state.set(MenuState::Landing);
    }
}

/// Handles Escape key / gamepad back to return to the pause menu main screen.
pub fn escape_to_pause_main(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut back_msgs: MessageReader<MenuBackPressed>,
    mut next_state: ResMut<NextState<PauseMenuState>>,
) {
    let back_pressed = back_msgs.read().next().is_some();
    if keyboard.just_pressed(KeyCode::Escape) || back_pressed {
        next_state.set(PauseMenuState::Main);
    }
}

/// Handles Escape / gamepad East or Start to return to running gameplay state.
///
/// Reads gamepad buttons directly via `just_pressed` rather than the
/// `MenuBackPressed` message — the latter persists for an extra frame,
/// which would re-fire immediately after entering a paused state and flip
/// back to Running (Start-in-Running → pause → Paused frame sees leftover
/// back-press → unpause).
pub fn escape_to_running(
    keyboard: Res<ButtonInput<KeyCode>>,
    active: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
    gamepads: Query<&Gamepad>,
    mut next_in_game_state: Option<ResMut<NextState<InGameState>>>,
    mut next_mp_state: Option<ResMut<NextState<MultiplayerGameState>>>,
) {
    let gamepad_back = active
        .gamepad_entity()
        .and_then(|e| gamepads.get(e).ok())
        .is_some_and(|g| {
            g.just_pressed(GamepadButton::East) || g.just_pressed(GamepadButton::Start)
        });
    if keyboard.just_pressed(KeyCode::Escape) || gamepad_back {
        if let Some(ref mut next_sp) = next_in_game_state {
            next_sp.set(InGameState::Running);
        }
        if let Some(ref mut next_mp) = next_mp_state {
            next_mp.set(MultiplayerGameState::Running);
        }
    }
}

/// Consumes the mouse button state on exit to prevent click bleed-through.
pub fn consume_mouse_on_exit(mut mouse_state: ResMut<MouseButtonState>) {
    mouse_state.left_consumed = true;
}

// ---------------------------------------------------------------------------
// Shared scroll handling
// ---------------------------------------------------------------------------

/// Handles mouse wheel scrolling for any scrollable container marked with `T`.
///
/// Walks up the entity hierarchy from the hovered entity to find a scrollable
/// parent, then adjusts its `ScrollPosition` based on the mouse wheel delta.
pub fn handle_scroll<T: Component>(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<bevy::picking::hover::HoverMap>,
    mut scrollable_query: Query<(&mut ScrollPosition, &ComputedNode), With<T>>,
    parent_query: Query<&ChildOf>,
) {
    use bevy::input::mouse::MouseScrollUnit;

    const LINE_HEIGHT: f32 = 10.0;
    const PIXEL_SCROLL_MULTIPLIER: f32 = 0.3;

    'event: for event in mouse_wheel_events.read() {
        let dy = match event.unit {
            MouseScrollUnit::Line => -event.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => -event.y * PIXEL_SCROLL_MULTIPLIER,
        };

        for pointer_map in hover_map.values() {
            for (hovered_entity, _) in pointer_map.iter() {
                let mut current_entity = *hovered_entity;
                loop {
                    if let Ok((mut scroll_position, computed)) =
                        scrollable_query.get_mut(current_entity)
                    {
                        let visible_size = computed.size();
                        let content_size = computed.content_size();
                        let max_scroll = (content_size.y - visible_size.y).max(0.0)
                            * computed.inverse_scale_factor();

                        scroll_position.y = (scroll_position.y + dy).clamp(0.0, max_scroll);
                        continue 'event;
                    }

                    if let Ok(parent) = parent_query.get(current_entity) {
                        current_entity = parent.get();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared button spawning
// ---------------------------------------------------------------------------

/// Generic screen cleanup system that despawns all entities with the given marker component.
///
/// Use as `cleanup_screen::<OnMyScreen>` when registering `OnExit` systems.
pub fn cleanup_screen<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).try_despawn();
    }
}

/// Spawns a 3D pushable button with edge + front layers.
///
/// The button wrapper is transparent and contains:
/// - An **edge** child (darker bg, stays in place) that peeks through at the bottom
/// - A **front** child (button face, offset upward) that slides on interaction
///
/// The front face moves up on hover and down on press, creating a physical depth illusion.
#[allow(clippy::too_many_arguments)]
pub fn spawn_button(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    action: impl Bundle,
    style: &ButtonStyle,
) {
    let depth = -BUTTON_3D_OFFSET_REST; // positive value = edge visible at bottom

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(style.width),
                height: Val::Px(style.height + depth),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Relative,
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BoxShadow(vec![ShadowStyle {
                color: BUTTON_SHADOW_COLOR,
                x_offset: Val::Px(0.0),
                y_offset: Val::Px(2.0),
                spread_radius: Val::Px(0.0),
                blur_radius: Val::Px(4.0),
            }]),
            ButtonColors {
                background: style.background,
                border: style.border,
            },
            ButtonAnimState {
                current: BUTTON_3D_OFFSET_REST,
                target: BUTTON_3D_OFFSET_REST,
            },
            Focusable,
            action,
        ))
        .with_children(|wrapper| {
            // Edge layer — same size as front but offset down and slightly narrower.
            // Outline lives here so it appears to go behind the front face.
            wrapper.spawn((
                ButtonEdge,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(edge_color(style.background)),
                Outline::new(Val::Px(1.0), Val::Px(1.0), BUTTON_REST_OUTLINE),
            ));

            // Front face — the interactive surface, offset upward.
            wrapper
                .spawn((
                    ButtonFront,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(style.height),
                        border: UiRect::all(Val::Px(style.border_width)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        overflow: Overflow::clip(),
                        position_type: PositionType::Relative,
                        top: Val::Px(BUTTON_3D_OFFSET_REST),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(opaque(style.background)),
                    BorderColor::all(style.border),
                ))
                .with_children(|front| {
                    if style.text_shadow {
                        spawn_shadowed_text(
                            front,
                            text,
                            style.font_size,
                            style.text_color,
                            Node::default(),
                        );
                    } else {
                        front.spawn((
                            Text::new(text),
                            TextFont::from_font_size(style.font_size),
                            TextColor(style.text_color),
                            TextLayout::new_with_justify(Justify::Center),
                        ));
                    }
                });
        });
}

/// Spawns text with a drop shadow inside the given parent.
/// Uses a relative wrapper with an absolute-positioned shadow behind the main text.
/// Offset scales with font size (font_size / 20).
fn spawn_shadowed_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    text_color: Color,
    node: Node,
) {
    let offset = font_size / 20.0;
    parent
        .spawn(Node {
            position_type: PositionType::Relative,
            ..node
        })
        .with_children(|wrapper| {
            wrapper.spawn((
                Text::new(text),
                TextFont::from_font_size(font_size),
                TextColor(TEXT_SHADOW_COLOR),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(offset),
                    top: Val::Px(offset),
                    width: Val::Percent(100.0),
                    ..default()
                },
            ));
            wrapper.spawn((
                Text::new(text),
                TextFont::from_font_size(font_size),
                TextColor(text_color),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

/// Spawns a standard page header row: title on the left, spacer, back button on the right.
///
/// The back button is marked `NoGamepadFocus` — on controller the B/East button
/// handles "back" universally, so we don't want the D-pad to land on it.
pub fn spawn_page_header<B: Component>(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    font_size: f32,
    title_color: Color,
    back_action: B,
    button_style: &ButtonStyle,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        })
        .with_children(|header| {
            spawn_title_with_shadow(header, title, font_size, title_color, Node::default());
            header.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            spawn_button(
                header,
                "Back",
                (back_action, super::focus::NoGamepadFocus),
                button_style,
            );
        });
}

/// Spawns a title text with a drop shadow effect.
pub fn spawn_title_with_shadow(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    font_size: f32,
    text_color: Color,
    node: Node,
) {
    spawn_shadowed_text(parent, text, font_size, text_color, node);
}

// ── UI Materials ──────────────────────────────────────────────────────────

/// Procedural parchment/stone texture material for panel backgrounds.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct ParchmentMaterial {
    #[uniform(0)]
    pub data: ParchmentData,
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub(crate) struct ParchmentData {
    pub base_color: LinearRgba,
    pub texture_strength: f32,
    pub vignette_strength: f32,
    pub noise_scale: f32,
    pub _padding: f32,
}

impl UiMaterial for ParchmentMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/parchment.wgsl".into()
    }
}

impl ParchmentMaterial {
    pub fn new(base_color: Color) -> Self {
        Self {
            data: ParchmentData {
                base_color: base_color.to_linear(),
                texture_strength: 0.45,
                vignette_strength: 0.4,
                noise_scale: 5.0,
                _padding: 0.0,
            },
        }
    }
}

/// Frosted glass overlay material for menu backgrounds.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(crate) struct FrostedGlassMaterial {
    #[uniform(0)]
    pub data: FrostedGlassData,
}

#[derive(ShaderType, Debug, Clone, Copy)]
pub(crate) struct FrostedGlassData {
    pub tint_color: LinearRgba,
    pub frost_intensity: f32,
    pub noise_scale: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl UiMaterial for FrostedGlassMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/frosted_glass.wgsl".into()
    }
}

impl FrostedGlassMaterial {
    pub fn new() -> Self {
        Self {
            data: FrostedGlassData {
                tint_color: Color::hsla(20.0, 0.04, 0.12, 0.30).to_linear(),
                frost_intensity: 1.0,
                noise_scale: 6.0,
                _padding1: 0.0,
                _padding2: 0.0,
            },
        }
    }
}

// ── Shared Slider ─────────────────────────────────────────────────────────

/// Configuration for spawning a slider row.
pub(crate) struct SliderRowConfig<
    'a,
    TText,
    TDownButton,
    TUpButton,
    TSliderTrack,
    TSliderFill,
    TSliderHandle,
> {
    pub label: &'a str,
    pub current_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub text_component: TText,
    pub down_button: TDownButton,
    pub up_button: TUpButton,
    pub slider_track: TSliderTrack,
    pub slider_fill: TSliderFill,
    pub slider_handle: TSliderHandle,
}

/// Spawns a slider row with label, decrease/increase buttons, track, fill, handle, and value text.
/// Shared by settings and roguelite modifier screens.
/// Spawns a dot-leader filler that expands to fill available horizontal space.
pub(crate) fn spawn_dot_leader(parent: &mut ChildSpawnerCommands, font_size: f32) {
    parent
        .spawn(Node {
            flex_grow: 1.0,
            flex_shrink: 1.0,
            min_width: Val::Px(0.0),
            overflow: Overflow::clip(),
            margin: UiRect::horizontal(Val::Px(6.0)),
            ..default()
        })
        .with_child((
            Text::new("......................................................................"),
            TextFont::from_font_size(font_size),
            TextColor(DOT_LEADER_COLOR),
        ));
}

pub(crate) fn spawn_slider_row<
    TText: Component,
    TDownButton: Component,
    TUpButton: Component,
    TSliderTrack: Component,
    TSliderFill: Component,
    TSliderHandle: Component,
>(
    parent: &mut ChildSpawnerCommands,
    config: SliderRowConfig<
        '_,
        TText,
        TDownButton,
        TUpButton,
        TSliderTrack,
        TSliderFill,
        TSliderHandle,
    >,
) {
    let SliderRowConfig {
        label,
        current_value,
        min_value,
        max_value,
        text_component,
        down_button,
        up_button,
        slider_track,
        slider_fill,
        slider_handle,
    } = config;

    let range = max_value - min_value;
    let normalized = if range > 0.0 {
        (current_value - min_value) / range
    } else {
        0.0
    };

    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexEnd,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(SLIDER_LABEL_FONT_SIZE),
                TextColor(TEXT_PRIMARY),
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
            ));

            spawn_dot_leader(row, SLIDER_LABEL_FONT_SIZE);

            // Controls
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: Val::Px(SLIDER_GAP),
                ..default()
            })
            .with_children(|controls| {
                // Decrease button
                controls
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(SLIDER_BUTTON_SIZE),
                            height: Val::Px(SLIDER_BUTTON_SIZE),
                            border: UiRect::all(Val::Px(SLIDER_BORDER_WIDTH)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BackgroundColor(SLIDER_BUTTON_BG),
                        ButtonColors {
                            background: SLIDER_BUTTON_BG,
                            border: SLIDER_BUTTON_BORDER_COLOR,
                        },
                        crate::ui::focus::Focusable,
                        down_button,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("-"),
                            TextFont::from_font_size(SLIDER_BUTTON_FONT_SIZE),
                            TextColor(TEXT_PRIMARY),
                        ));
                    });

                // Slider track
                controls
                    .spawn((
                        Node {
                            width: Val::Px(SLIDER_TRACK_WIDTH),
                            height: Val::Px(12.0),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            position_type: PositionType::Relative,
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BackgroundColor(SLIDER_TRACK_BG),
                        Interaction::default(),
                        RelativeCursorPosition::default(),
                        slider_track,
                    ))
                    .with_children(|track| {
                        // Slider fill
                        track.spawn((
                            Node {
                                width: Val::Percent(normalized * 100.0),
                                height: Val::Percent(100.0),
                                border_radius: BorderRadius {
                                    top_left: Val::Px(6.0),
                                    bottom_left: Val::Px(6.0),
                                    top_right: Val::Px(0.0),
                                    bottom_right: Val::Px(0.0),
                                },
                                ..default()
                            },
                            BackgroundColor(SLIDER_BUTTON_BORDER_COLOR),
                            slider_fill,
                        ));

                        // Slider handle (offset by -2px to center the 4px wide bar)
                        track.spawn((
                            Node {
                                width: Val::Px(4.0),
                                height: Val::Px(20.0),
                                position_type: PositionType::Absolute,
                                left: Val::Px(normalized * SLIDER_TRACK_WIDTH - 2.0),
                                top: Val::Px(-4.0),
                                border_radius: BorderRadius::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::WHITE),
                            BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                            Interaction::default(),
                            RelativeCursorPosition::default(),
                            slider_handle,
                        ));
                    });

                // Increase button
                controls
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(SLIDER_BUTTON_SIZE),
                            height: Val::Px(SLIDER_BUTTON_SIZE),
                            border: UiRect::all(Val::Px(SLIDER_BORDER_WIDTH)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BorderColor::all(SLIDER_BUTTON_BORDER_COLOR),
                        BackgroundColor(SLIDER_BUTTON_BG),
                        ButtonColors {
                            background: SLIDER_BUTTON_BG,
                            border: SLIDER_BUTTON_BORDER_COLOR,
                        },
                        crate::ui::focus::Focusable,
                        up_button,
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new("+"),
                            TextFont::from_font_size(SLIDER_BUTTON_FONT_SIZE),
                            TextColor(TEXT_PRIMARY),
                        ));
                    });

                // Value display
                controls.spawn((
                    Text::new(format!("{}%", (current_value * 100.0) as u32)),
                    TextFont::from_font_size(SLIDER_LABEL_FONT_SIZE),
                    TextColor(TEXT_PRIMARY),
                    Node {
                        width: Val::Px(60.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    text_component,
                ));
            });
        });
}
