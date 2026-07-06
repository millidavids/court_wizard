//! Controller binding diagram rendered at the bottom of the Controller settings tab.
//!
//! Tracks `CurrentControllerGlyphStyle` (resolved to the connected gamepad's
//! vendor, or Xbox if none) and renders a labeled grid of bindings using the
//! existing Kenney input fonts.

use bevy::input::gamepad::GamepadButton;
use bevy::prelude::*;

use crate::config::ControllerGlyphStyle;
use crate::ui::constants::{CONTENT_BG, DETAIL_BORDER, PANEL_BORDER_RADIUS, WARNING_COLOR};
use crate::ui::focus::constants::{STICK_DIRECTION_THRESHOLD, STICK_RESET_THRESHOLD};
use crate::ui::gamepad_glyphs::{GamepadGlyphFonts, glyph_char};

use super::constants::{
    BUTTON_BORDER_WIDTH, LABEL_FONT_SIZE, MARGIN_SMALL, SECTION_FONT_SIZE, TEXT_COLOR,
};

const GLYPH_SIZE: f32 = 56.0;
/// Tight vertical spacing reused across the panel's stacked sections.
const ROW_GAP: f32 = 4.0;

struct Binding {
    buttons: &'static [GamepadButton],
    label: &'static str,
}

const LEFT_BINDINGS: &[Binding] = &[
    Binding {
        buttons: &[GamepadButton::LeftThumb],
        label: "Aim",
    },
    Binding {
        buttons: &[GamepadButton::RightTrigger2],
        label: "Cast Spell",
    },
    Binding {
        buttons: &[GamepadButton::LeftTrigger2],
        label: "Cancel Cast",
    },
    Binding {
        buttons: &[GamepadButton::RightThumb, GamepadButton::RightTrigger2],
        label: "Pick Spell",
    },
    Binding {
        buttons: &[GamepadButton::South],
        label: "Confirm",
    },
];

const RIGHT_BINDINGS: &[Binding] = &[
    Binding {
        buttons: &[GamepadButton::East],
        label: "Back / Close",
    },
    Binding {
        buttons: &[GamepadButton::West],
        label: "Spell Book",
    },
    Binding {
        buttons: &[GamepadButton::North],
        label: "Cauldron",
    },
    Binding {
        buttons: &[GamepadButton::LeftTrigger, GamepadButton::RightTrigger],
        label: "Cycle Tabs",
    },
    Binding {
        buttons: &[GamepadButton::Start],
        label: "Pause",
    },
];

/// Sized to fit two glyphs side-by-side (e.g. LB+RB) so single-glyph rows
/// align with two-glyph rows.
const GLYPH_CELL_WIDTH: f32 = GLYPH_SIZE * 2.0 + 8.0;

fn title_for(style: ControllerGlyphStyle) -> &'static str {
    match style {
        ControllerGlyphStyle::PlayStation => "PLAYSTATION",
        ControllerGlyphStyle::SteamDeck => "STEAM",
        ControllerGlyphStyle::Switch => "SWITCH",
        ControllerGlyphStyle::Auto | ControllerGlyphStyle::Xbox => "XBOX",
    }
}

/// Marker on the controller-binding diagram's focusable root. `pub(crate)`
/// only because it appears in `cycle_controller_diagram_scheme`'s query, which
/// is registered from the (cross-module) settings plugins.
#[derive(Component)]
pub(crate) struct ControllerDiagram;

/// The vendor scheme the diagram is currently displaying. Initialized to the
/// connected controller's resolved style; Left/Right cycles it locally (without
/// touching the global `CurrentControllerGlyphStyle`).
#[derive(Component)]
pub(crate) struct ControllerDiagramScheme(ControllerGlyphStyle);

/// Concrete vendor styles the diagram cycles through (`Auto` is resolved away
/// upstream, so it never appears here).
const DIAGRAM_SCHEMES: [ControllerGlyphStyle; 4] = [
    ControllerGlyphStyle::Xbox,
    ControllerGlyphStyle::PlayStation,
    ControllerGlyphStyle::SteamDeck,
    ControllerGlyphStyle::Switch,
];

pub(super) fn spawn_controller_diagram_section(
    parent: &mut ChildSpawnerCommands,
    glyph_fonts: &GamepadGlyphFonts,
    style: ControllerGlyphStyle,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(97.0),
                margin: UiRect {
                    top: Val::Px(MARGIN_SMALL),
                    bottom: Val::Px(MARGIN_SMALL),
                    ..default()
                },
                padding: UiRect::all(Val::Px(6.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(ROW_GAP),
                border: UiRect::all(Val::Px(BUTTON_BORDER_WIDTH)),
                border_radius: BorderRadius::all(Val::Px(PANEL_BORDER_RADIUS)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(CONTENT_BG),
            BorderColor::all(DETAIL_BORDER),
            // Focusable (deliberately NOT a `Button`, so A/South is a no-op) so
            // controller focus nav can reach it and scroll it into view. The
            // flat-background marker gives the standard gamepad focus tint, and
            // `ConsumeHorizontalNav` lets Left/Right cycle vendor schemes instead
            // of hopping focus sideways.
            crate::ui::focus::Focusable,
            crate::ui::focus::FocusableFlatBackground { base: CONTENT_BG },
            crate::ui::focus::ConsumeHorizontalNav,
            ControllerDiagram,
            ControllerDiagramScheme(style),
        ))
        .with_children(|panel| {
            populate_controller_diagram(panel, glyph_fonts, style);
        });
}

/// Spawns the diagram's inner content (title row + divider + binding columns +
/// footer) into an existing root. Re-run by `cycle_controller_diagram_scheme`
/// to re-render after a scheme change without respawning the focusable root.
fn populate_controller_diagram(
    panel: &mut ChildSpawnerCommands,
    glyph_fonts: &GamepadGlyphFonts,
    style: ControllerGlyphStyle,
) {
    let glyph_font = glyph_fonts.font_for(style);

    // Title row: « VENDOR » — the guillemets cue that Left/Right swap schemes.
    panel
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(MARGIN_SMALL),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new("\u{00AB}"),
                TextFont::from_font_size(SECTION_FONT_SIZE),
                TextColor(TEXT_COLOR),
            ));
            row.spawn((
                Text::new(title_for(style)),
                TextFont::from_font_size(SECTION_FONT_SIZE),
                TextColor(WARNING_COLOR),
            ));
            row.spawn((
                Text::new("\u{00BB}"),
                TextFont::from_font_size(SECTION_FONT_SIZE),
                TextColor(TEXT_COLOR),
            ));
        });

    panel.spawn((
        Node {
            width: Val::Percent(60.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(ROW_GAP)),
            ..default()
        },
        BackgroundColor(DETAIL_BORDER),
    ));

    panel
        .spawn(Node {
            min_width: Val::Px(0.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(32.0),
            margin: UiRect::vertical(Val::Px(ROW_GAP)),
            ..default()
        })
        .with_children(|cols| {
            spawn_binding_column(cols, &glyph_font, style, LEFT_BINDINGS);
            spawn_binding_column(cols, &glyph_font, style, RIGHT_BINDINGS);
        });

    panel.spawn((
        Text::new("PICK SPELL = HOLD RIGHT STICK + CAST"),
        TextFont::from_font_size(10.0),
        TextColor(DETAIL_BORDER),
        Node {
            margin: UiRect::top(Val::Px(ROW_GAP)),
            ..default()
        },
    ));
}

/// Cycles the focused controller-binding diagram through vendor schemes on a
/// Left/Right press (D-pad or left stick). Acts only while the diagram is the
/// gamepad-focused element; re-renders it in place — the root entity (and thus
/// focus + scroll position) is preserved. Deliberately does NOT mutate
/// `CurrentControllerGlyphStyle`, so this is a local reference browser and the
/// glyphs shown elsewhere (tutorials, prompts) and the saved preference are
/// unaffected. Registered per settings context (Menu / Pause / MP).
pub(crate) fn cycle_controller_diagram_scheme(
    active: Res<crate::game::input::gamepad::resources::ActiveInputDevice>,
    action_state: Res<crate::game::input::action_state::GamepadActionState>,
    focused: Res<crate::ui::focus::resources::FocusedEntity>,
    glyph_fonts: Option<Res<GamepadGlyphFonts>>,
    mut commands: Commands,
    mut diagram: Query<(Entity, &mut ControllerDiagramScheme), With<ControllerDiagram>>,
    mut stick_armed: Local<bool>,
) {
    use crate::game::input::action_state::GamepadAction;

    // Confirm a controller is active and the diagram is the focused element. If
    // not (no controller, fonts not loaded, or focus elsewhere), re-arm the stick
    // latch so the first push after (re)focusing the diagram always registers,
    // then bail. (`focus.0` navigates to the diagram vertically, so the stick is
    // near-center on arrival and re-arming doesn't cause a spurious cycle.)
    let focused_diagram = focused.0.filter(|&e| diagram.contains(e));
    let (Some(focused_entity), Some(fonts)) = (focused_diagram, glyph_fonts.as_deref()) else {
        *stick_armed = true;
        return;
    };
    if !active.is_gamepad() {
        *stick_armed = true;
        return;
    }

    // D-pad is inherently edge-triggered; the left stick fires once per push and
    // re-arms only after returning near center (so a held stick doesn't spin).
    // Thresholds are shared with focus navigation for a consistent feel.
    let lx = action_state.left_stick.x;
    let mut delta: i32 = if action_state.just_pressed(GamepadAction::AbilityRight) {
        1
    } else if action_state.just_pressed(GamepadAction::AbilityLeft) {
        -1
    } else {
        0
    };
    if lx.abs() < STICK_RESET_THRESHOLD {
        *stick_armed = true;
    } else if delta == 0 && *stick_armed && lx.abs() >= STICK_DIRECTION_THRESHOLD {
        *stick_armed = false;
        delta = if lx > 0.0 { 1 } else { -1 };
    }
    if delta == 0 {
        return;
    }

    // Borrow the scheme mutably only now that we're actually cycling.
    let Ok((entity, mut scheme)) = diagram.get_mut(focused_entity) else {
        return;
    };
    let cur = DIAGRAM_SCHEMES
        .iter()
        .position(|s| *s == scheme.0)
        .unwrap_or(0);
    let next = (cur as i32 + delta).rem_euclid(DIAGRAM_SCHEMES.len() as i32) as usize;
    let new_style = DIAGRAM_SCHEMES[next];
    scheme.0 = new_style;

    // Re-render in place: drop the old content, repopulate with the new scheme.
    // The root (with Focusable + ControllerDiagram + focus) survives.
    let fonts = fonts.clone();
    commands.entity(entity).despawn_related::<Children>();
    commands.entity(entity).with_children(|panel| {
        populate_controller_diagram(panel, &fonts, new_style);
    });
}

fn spawn_binding_column(
    parent: &mut ChildSpawnerCommands,
    glyph_font: &Handle<Font>,
    style: ControllerGlyphStyle,
    bindings: &[Binding],
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(ROW_GAP),
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|column| {
            for binding in bindings {
                spawn_binding_row(column, glyph_font, style, binding);
            }
        });
}

fn spawn_binding_row(
    parent: &mut ChildSpawnerCommands,
    glyph_font: &Handle<Font>,
    style: ControllerGlyphStyle,
    binding: &Binding,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(MARGIN_SMALL),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                width: Val::Px(GLYPH_CELL_WIDTH),
                height: Val::Px(GLYPH_SIZE + 4.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                ..default()
            })
            .with_children(|cell| {
                for &button in binding.buttons {
                    if let Some(ch) = glyph_char(button, style) {
                        cell.spawn((
                            Text::new(ch.to_string()),
                            TextFont {
                                font: glyph_font.clone(),
                                font_size: GLYPH_SIZE,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));
                    }
                }
            });

            row.spawn((
                Text::new(binding.label),
                TextFont::from_font_size(LABEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
            ));
        });
}
