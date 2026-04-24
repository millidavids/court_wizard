use bevy::prelude::*;

use std::f32::consts::TAU;

use bevy::ui::UiTransform;

use crate::config::ActiveSave;
use crate::config::save_data::load_unified_save;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::game::resources::KillStats;
use crate::game::units::wizard::components::Spell;
use crate::state::AppState;
use crate::ui::components::{ButtonActive, ButtonColors, ButtonFront};
use crate::ui::systems::spawn_button;

use super::components::{ArcaneRuneText, OnWizardTowerScreen, WizardTowerButtonAction};
use super::constants::*;
use super::materials::{ArcaneRuneData, ArcaneRuneMaterial};

/// Returns all currently unlocked spells from save data.
fn get_unlocked_spells() -> Vec<Spell> {
    let save = load_unified_save();
    let unlocked_names: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();
    Spell::all()
        .iter()
        .filter(|s| unlocked_names.contains(&format!("{:?}", s)))
        .copied()
        .collect()
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Which tab is currently active in the wizard tower hub.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum WizardTowerTab {
    #[default]
    Roguelite,
    Endless,
    Study,
    Multiplayer,
}

impl WizardTowerTab {
    pub fn all() -> &'static [WizardTowerTab] {
        &[
            WizardTowerTab::Roguelite,
            WizardTowerTab::Endless,
            WizardTowerTab::Study,
            WizardTowerTab::Multiplayer,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            WizardTowerTab::Roguelite => "Roguelite",
            WizardTowerTab::Endless => "Endless",
            WizardTowerTab::Study => "Study",
            WizardTowerTab::Multiplayer => "Multiplayer",
        }
    }

    /// Whether this tab is disabled and cannot be clicked.
    pub fn is_disabled(&self) -> bool {
        matches!(self, WizardTowerTab::Multiplayer)
    }
}

/// What the right panel is currently showing.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum RightPanelView {
    /// The default view for the active tab.
    #[default]
    TabContent,
    /// Wizard type selection grid.
    WizardSelect,
}

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marker for the left panel container.
#[derive(Component)]
pub(crate) struct WizardTowerLeftPanel;

/// Marker for the right panel container.
#[derive(Component)]
pub(crate) struct WizardTowerRightPanel;

/// Identifies which tab a button corresponds to.
#[derive(Component)]
pub(super) struct WizardTowerTabButton(pub WizardTowerTab);

/// Marker on the row container holding all top-level tab buttons, so the
/// tutorial system can highlight the whole row at once.
#[derive(Component)]
pub(crate) struct WizardTowerTabRow;

/// Marker for tabs that are disabled and cannot be clicked.
#[derive(Component)]
pub(super) struct DisabledTab;

// ---------------------------------------------------------------------------
// Layout setup
// ---------------------------------------------------------------------------

/// Spawns the full wizard tower tabbed layout.
///
/// ```text
/// +---------------------------------------------------------------+
/// | [Roguelite] [Endless] [Study] [Multiplayer*]       [<- Back]  |
/// +--------------------+------------------------------------------+
/// |                    |                                            |
/// |   Left Panel       |       Right Panel                         |
/// |   (33% width)      |       (67% width)                         |
/// |                    |                                            |
/// +--------------------+------------------------------------------+
/// ```
#[allow(clippy::too_many_arguments)]
pub(super) fn setup_wizard_tower_layout(
    mut commands: Commands,
    mut config: ResMut<crate::config::GameConfig>,
    mut active_save: ResMut<crate::config::ActiveSave>,
    existing_tab: Option<Res<WizardTowerTab>>,
    existing_run: Option<Res<crate::game::game_mode::components::RogueliteRunState>>,
    asset_server: Res<AssetServer>,
    mut rune_materials: ResMut<Assets<ArcaneRuneMaterial>>,
) {
    // Preserve tab if already set (e.g. returning from a level sets the
    // appropriate tab before transitioning to MetaGame). Otherwise default.
    if existing_tab.is_none() {
        commands.insert_resource(WizardTowerTab::default());
    }
    commands.insert_resource(RightPanelView::default());

    if active_save.0.is_none() {
        crate::config::save_data::load_or_create_wizard(
            config.wizard_type,
            &mut config,
            &mut active_save,
        );
    }

    // Restore dormant roguelite run from disk if no in-memory run exists.
    // This allows resuming a run after closing the game or switching modes.
    if existing_run.is_none()
        && let Some(saved_run) = crate::config::save_data::load_current_roguelite_run(&active_save)
    {
        // Restore game mode and run state resources
        commands.insert_resource(crate::game::game_mode::components::GameMode::Roguelite);
        commands.insert_resource(crate::game::game_mode::components::RogueliteRunState {
            started_at: saved_run.started_at,
            level_stats: saved_run.level_stats,
        });
        if let Some(mods) = saved_run.modifiers {
            commands.insert_resource(mods);
        }
        let toggles =
            crate::game::game_mode::components::ActiveToggles::from_ids(&saved_run.active_toggles);
        commands.insert_resource(toggles);
        if let Some(seed) = saved_run.seed {
            commands.insert_resource(crate::game::seeded_rng::resources::GameSeed(seed));
        }
        // Ensure config reflects the run's state
        config.current_level = saved_run.current_level;
        config.wizard_type = saved_run.wizard_type;
        // Auto-select the roguelite tab
        commands.insert_resource(WizardTowerTab::Roguelite);
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnWizardTowerScreen,
        ))
        .with_children(|root| {
            let unlocked_count = get_unlocked_spells().len() as f32;
            let rune_mat = rune_materials.add(ArcaneRuneMaterial {
                data: ArcaneRuneData {
                    color: RUNE_COLOR.to_linear(),
                    time: 0.0,
                    opacity: RUNE_OPACITY,
                    unlocked_count,
                    _padding: 0.0,
                },
            });
            root.spawn((
                MaterialNode(rune_mat),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
                super::components::ArcaneRuneBackground,
            ));

            // Orbiting spell name text around the rune circles
            spawn_arcane_rune_text(root, &asset_server);

            // Header row: title left, Back button right
            let mut header_cmd = root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(MARGIN_SMALL)),
                    ..default()
                },
                ZIndex(10),
            ));
            #[cfg(debug_assertions)]
            header_cmd.insert(super::components::WizardTowerUiContent);
            header_cmd.with_children(|header| {
                crate::ui::systems::spawn_title_with_shadow(
                    header,
                    "Wizard Tower",
                    TITLE_FONT_SIZE,
                    TITLE_COLOR,
                    Node::default(),
                );
                header.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                spawn_button(
                    header,
                    "Back",
                    (
                        WizardTowerButtonAction::ReturnToMenu,
                        crate::ui::focus::NoGamepadFocus,
                    ),
                    &BACK_BUTTON_STYLE,
                );
            });

            // Content area: left panel + right column (tabs + right panel)
            let mut content_cmd = root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(90.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(COLUMN_GAP),
                ..default()
            });
            #[cfg(debug_assertions)]
            content_cmd.insert(super::components::WizardTowerUiContent);
            content_cmd.with_children(|content| {
                // Left panel — ZIndex ensures it renders above graph edges
                // that may visually leak due to UiTransform rotation bypassing clip.
                content.spawn((
                    Node {
                        width: Val::Percent(LEFT_PANEL_PERCENT),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(SECTION_PADDING)),
                        row_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(1.0)),
                        overflow: Overflow::scroll_y(),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ScrollPosition::default(),
                    Interaction::None,
                    BackgroundColor(DETAIL_BG),
                    BorderColor::all(DETAIL_BORDER),
                    BorderRadius::all(Val::Px(6.0)),
                    ZIndex(10),
                    WizardTowerLeftPanel,
                ));

                // Right column: tab bar + right panel
                content
                    .spawn(Node {
                        width: Val::Percent(RIGHT_PANEL_PERCENT),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(MARGIN_SMALL),
                        flex_grow: 1.0,
                        ..default()
                    })
                    .with_children(|right_col| {
                        // Tab bar inside the right column
                        right_col
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(4.0),
                                    border: UiRect::all(Val::Px(0.0)),
                                    ..default()
                                },
                                WizardTowerTabRow,
                            ))
                            .with_children(|tab_row| {
                                let initial_tab =
                                    existing_tab.as_deref().copied().unwrap_or_default();
                                for tab in WizardTowerTab::all() {
                                    let is_active = *tab == initial_tab;
                                    let is_disabled = tab.is_disabled();

                                    let (bg, border) = if is_active {
                                        (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
                                    } else {
                                        (INACTIVE_TAB_BG, TAB_BORDER)
                                    };

                                    let text_color = if is_disabled {
                                        DISABLED_TAB_TEXT
                                    } else {
                                        TEXT_COLOR
                                    };

                                    let mut tab_btn = tab_row.spawn((
                                        Button,
                                        Node {
                                            height: Val::Px(TAB_HEIGHT),
                                            padding: UiRect::horizontal(Val::Px(TAB_PADDING_H)),
                                            border: UiRect::all(Val::Px(1.0)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(bg),
                                        BorderColor::all(border),
                                        BorderRadius::all(Val::Px(4.0)),
                                        ButtonColors {
                                            background: bg,
                                            border,
                                        },
                                        WizardTowerTabButton(*tab),
                                        crate::ui::focus::Focusable,
                                        crate::ui::focus::TabFocusable,
                                    ));

                                    if is_disabled {
                                        tab_btn.insert(DisabledTab);
                                    }

                                    tab_btn.with_children(|btn| {
                                        btn.spawn((
                                            Text::new(tab.label()),
                                            TextFont::from_font_size(TAB_FONT_SIZE),
                                            TextColor(text_color),
                                        ));
                                    });
                                }
                            });

                        // Right panel — scrollable content below tabs
                        right_col.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                border: UiRect::all(Val::Px(1.0)),
                                overflow: Overflow::scroll_y(),
                                flex_grow: 1.0,
                                ..default()
                            },
                            ScrollPosition::default(),
                            Interaction::None,
                            BackgroundColor(SECTION_BG),
                            BorderColor::all(DETAIL_BORDER),
                            BorderRadius::all(Val::Px(6.0)),
                            WizardTowerRightPanel,
                        ));
                    });
            });
        });
}

// ---------------------------------------------------------------------------
// Tab click handling
// ---------------------------------------------------------------------------

/// Updates the active tab when a tab button is clicked. Disabled tabs are ignored.
pub(super) fn handle_tab_click(
    mut button_clicked: MessageReader<MouseClicked>,
    tab_query: Query<&WizardTowerTabButton, Without<DisabledTab>>,
    mut tab_resource: ResMut<WizardTowerTab>,
    mut right_panel_view: ResMut<RightPanelView>,
) {
    for event in button_clicked.read() {
        if let Ok(tab_btn) = tab_query.get(event.button)
            && *tab_resource != tab_btn.0
        {
            *tab_resource = tab_btn.0;
            // Reset to default tab content view when switching tabs
            *right_panel_view = RightPanelView::TabContent;
        }
    }
}

// ---------------------------------------------------------------------------
// Back button handling
// ---------------------------------------------------------------------------

/// Handles the back button to return to the main menu.
pub(super) fn handle_back_button(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&WizardTowerButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(WizardTowerButtonAction::ReturnToMenu) = button_query.get(event.button) {
            return_to_main_menu(
                &mut next_app_state,
                &mut kill_stats,
                &mut active_save,
                &mut channel_change,
            );
        }
    }
}

/// Handles Escape key / gamepad back to return to the main menu from the wizard tower.
#[allow(clippy::too_many_arguments)]
pub(super) fn escape_to_main_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut back_msgs: MessageReader<crate::game::input::gamepad::messages::MenuBackPressed>,
    tab: Option<Res<WizardTowerTab>>,
    selected_spell: Option<Res<super::components::SelectedStudySpell>>,
    selected_bonus: Option<Res<super::components::SelectedInsightBonus>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    let back_pressed = back_msgs.read().next().is_some();
    if !keyboard.just_pressed(KeyCode::Escape) && !back_pressed {
        return;
    }
    // On Study with a selection, `study_back_to_cursor` consumes the back
    // to deselect; swallow it here so we don't also exit to main menu.
    let study_consumed = tab.as_deref().is_some_and(|t| *t == WizardTowerTab::Study)
        && (selected_spell.as_deref().is_some_and(|s| s.0.is_some())
            || selected_bonus.as_deref().is_some_and(|s| s.0.is_some()));
    if study_consumed {
        return;
    }
    return_to_main_menu(
        &mut next_app_state,
        &mut kill_stats,
        &mut active_save,
        &mut channel_change,
    );
}

fn return_to_main_menu(
    next_app_state: &mut ResMut<NextState<AppState>>,
    kill_stats: &mut ResMut<KillStats>,
    active_save: &mut ResMut<ActiveSave>,
    channel_change: &mut MessageWriter<ChannelChangeMessage>,
) {
    channel_change.write(ChannelChangeMessage);
    kill_stats.reset();
    active_save.0 = None;
    next_app_state.set(AppState::MainMenu);
}

// ---------------------------------------------------------------------------
// Panel rebuild on tab change
// ---------------------------------------------------------------------------

/// When the active tab changes, despawn children of both panels and rebuild
/// with the appropriate tab's content.
#[allow(clippy::too_many_arguments)]
pub(super) fn rebuild_panels_on_tab_change(
    mut commands: Commands,
    tab: Res<WizardTowerTab>,
    right_panel_view: Res<RightPanelView>,
    left_panel: Query<Entity, With<WizardTowerLeftPanel>>,
    right_panel: Query<Entity, With<WizardTowerRightPanel>>,
    mut config: ResMut<crate::config::GameConfig>,
    roguelite_run: Option<Res<crate::game::game_mode::components::RogueliteRunState>>,
    roguelite_modifiers: Option<Res<crate::game::game_mode::components::RogueliteModifiers>>,
    pending_toggles: Option<Res<super::roguelite_tab::PendingToggles>>,
    active_toggles: Option<Res<crate::game::game_mode::components::ActiveToggles>>,
    battle_insight: Res<crate::game::resources::BattleInsightData>,
    asset_server: Res<AssetServer>,
    mut progress_materials: ResMut<Assets<super::materials::RadialProgressMaterial>>,
    mut ring_materials: ResMut<Assets<super::materials::ConcentricRingsMaterial>>,
    mut star_sky_materials: ResMut<Assets<super::materials::StarSkyMaterial>>,
) {
    // The run_if condition already gates this system to only run when
    // tab, right_panel_view, or RogueliteRunState changes/removes.
    // No additional early-return needed.

    let Ok(left_entity) = left_panel.single() else {
        return;
    };
    let Ok(right_entity) = right_panel.single() else {
        return;
    };

    // Despawn children of both panels
    commands.entity(left_entity).despawn_related::<Children>();
    commands.entity(right_entity).despawn_related::<Children>();

    // If showing wizard cards, build the card grid instead of tab content
    if *right_panel_view == RightPanelView::WizardSelect {
        commands.init_resource::<super::wizard_cards::SelectedWizard>();
        super::wizard_cards::build_wizard_card_grid(&mut commands, right_entity);
        return;
    }

    match *tab {
        WizardTowerTab::Roguelite => {
            if let Some(ref run_state) = roguelite_run {
                // Active run: show run info + continue/end buttons
                super::roguelite_tab::build_roguelite_active_run_right_panel(
                    &mut commands,
                    right_entity,
                );
                super::roguelite_tab::build_roguelite_active_run_left_panel(
                    &mut commands,
                    left_entity,
                    &config,
                    run_state,
                );
            } else {
                // No active run: init resources and show modifier UI
                super::roguelite_tab::init_roguelite_tab_resources(
                    &mut commands,
                    &mut config,
                    roguelite_modifiers.as_deref(),
                    pending_toggles.as_deref(),
                    active_toggles.as_deref(),
                );
                let seed_text = config.seed.map(|s| s.to_string()).unwrap_or_default();
                let mods = roguelite_modifiers.as_deref().cloned().unwrap_or_default();
                let pt = pending_toggles.as_deref().cloned().unwrap_or_default();
                super::roguelite_tab::build_roguelite_no_run_right_panel(
                    &mut commands,
                    right_entity,
                    &mods,
                    &pt,
                    &seed_text,
                );
                super::roguelite_tab::build_roguelite_no_run_left_panel(
                    &mut commands,
                    left_entity,
                    &config,
                    &mods,
                    &pt,
                );
            }
        }
        WizardTowerTab::Endless => {
            super::endless_tab::build_endless_right_panel(&mut commands, right_entity, &config);
            super::endless_tab::build_endless_left_panel(
                &mut commands,
                left_entity,
                config.wizard_type,
                &config,
            );
        }
        WizardTowerTab::Study => {
            super::study_tab::build_study_panels(
                &mut commands,
                right_entity,
                left_entity,
                &battle_insight,
                &asset_server,
                &mut progress_materials,
                &mut ring_materials,
                &mut star_sky_materials,
            );
        }
        WizardTowerTab::Multiplayer => {
            // Disabled tab - should not be reachable
        }
    }
}

// ---------------------------------------------------------------------------
// Tab visual state
// ---------------------------------------------------------------------------

/// Updates tab button visuals to reflect the currently active tab.
pub(super) fn update_tab_active_state(
    mut commands: Commands,
    tab: Res<WizardTowerTab>,
    tab_buttons: Query<(
        Entity,
        &WizardTowerTabButton,
        &Children,
        Has<DisabledTab>,
        Has<ButtonActive>,
    )>,
    mut tab_bg: Query<(&mut BackgroundColor, &mut BorderColor, &mut ButtonColors)>,
    mut front_bg: Query<
        (&mut BackgroundColor, &mut BorderColor),
        (With<ButtonFront>, Without<ButtonColors>),
    >,
) {
    for (entity, tab_btn, children, _is_disabled, has_active) in &tab_buttons {
        let is_active = tab_btn.0 == *tab;

        // Skip if the active/inactive state already matches
        if is_active == has_active && !tab.is_changed() {
            continue;
        }

        let (bg, border) = if is_active {
            commands.entity(entity).insert(ButtonActive);
            (ACTIVE_TAB_BG, ACTIVE_TAB_BORDER)
        } else {
            commands.entity(entity).remove::<ButtonActive>();
            (INACTIVE_TAB_BG, TAB_BORDER)
        };

        if let Ok((mut bg_color, mut border_color, mut colors)) = tab_bg.get_mut(entity) {
            *bg_color = bg.into();
            *border_color = BorderColor::all(border);
            colors.background = bg;
            colors.border = border;
        }

        // Update the 3D front face child
        for child in children.iter() {
            if let Ok((mut front_bg_color, mut front_border_color)) = front_bg.get_mut(child) {
                *front_bg_color = crate::ui::systems::opaque(bg).into();
                *front_border_color = BorderColor::all(border);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Arcane rune background
// ---------------------------------------------------------------------------

/// Spawns spell name text entities that orbit around the arcane rune circles.
/// Text entities are direct children of the root node (same parent as the MaterialNode)
/// so their coordinate system matches the shader exactly.
fn spawn_arcane_rune_text(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    let rune_font: Handle<Font> = asset_server.load("fonts/Oxford-Runes.ttf");
    let unlocked_spells = get_unlocked_spells();

    if unlocked_spells.is_empty() {
        return;
    }

    // Split unlocked spells by category into four concentric rings.
    // Radii sit between the shader's geometric rings:
    //   Shader rings: 0.44, 0.42 (outer), 0.32 (middle), 0.22 (inner), 0.06 (core)
    // Directions alternate CCW / CW / CCW / CW from outermost to innermost.
    use crate::game::units::wizard::components::SpellCategory;
    use crate::ui::constants::spell_category_color;
    let rings: &[(SpellCategory, f32, f32)] = &[
        (SpellCategory::Control, 0.47, -0.04),
        (SpellCategory::Offense, 0.37, 0.06),
        (SpellCategory::Support, 0.27, -0.08),
        (SpellCategory::Utility, 0.16, 0.10),
    ];

    for &(category, radius, speed) in rings {
        let color = spell_category_color(category);
        let spells: Vec<&Spell> = unlocked_spells
            .iter()
            .filter(|s| s.category() == category)
            .collect();
        if spells.is_empty() {
            continue;
        }
        let count = spells.len();
        for (i, spell) in spells.iter().enumerate() {
            let base_angle = (i as f32 / count as f32) * TAU;
            spawn_rune_text_node(
                parent,
                spell.display_name(),
                base_angle,
                radius,
                speed,
                color,
                &rune_font,
            );
        }
    }
}

/// Spawns a single rune text entity at a position on a circle.
fn spawn_rune_text_node(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    angle: f32,
    radius: f32,
    speed: f32,
    color: Color,
    font: &Handle<Font>,
) {
    parent.spawn((
        Text::new(label),
        TextFont {
            font: font.clone(),
            font_size: RUNE_TEXT_SIZE,
            ..default()
        },
        TextColor(color.with_alpha(RUNE_TEXT_ALPHA)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            ..default()
        },
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
        UiTransform::default(),
        ArcaneRuneText {
            angle,
            radius,
            speed,
        },
    ));
}

/// Rebuilds the orbiting rune text and updates the shader when spells are unlocked.
pub(super) fn rebuild_rune_on_spell_unlock(
    mut commands: Commands,
    mut spell_researched: MessageReader<crate::game::messages::SpellResearchedMessage>,
    asset_server: Res<AssetServer>,
    root_query: Query<Entity, With<OnWizardTowerScreen>>,
    text_query: Query<Entity, With<ArcaneRuneText>>,
    mut materials: ResMut<Assets<ArcaneRuneMaterial>>,
) {
    if spell_researched.read().next().is_none() {
        return;
    }

    let unlocked_count = get_unlocked_spells().len() as f32;
    for (_id, mat) in materials.iter_mut() {
        mat.data.unlocked_count = unlocked_count;
    }

    for entity in &text_query {
        commands.entity(entity).despawn();
    }

    let Ok(root) = root_query.single() else {
        return;
    };
    commands.entity(root).with_children(|root| {
        spawn_arcane_rune_text(root, &asset_server);
    });
}

/// Updates the time uniform on the arcane rune background material each frame.
pub(super) fn update_arcane_rune_time(
    time: Res<Time>,
    mut materials: ResMut<Assets<ArcaneRuneMaterial>>,
) {
    let elapsed = time.elapsed_secs();
    for (_id, mat) in materials.iter_mut() {
        mat.data.time = elapsed;
    }
}

/// Updates positions and rotations of orbiting spell name text around the rune circles.
/// Uses percentage-based positioning so text circles are inherently concentric with
/// the shader's geometric pattern (both reference the same parent coordinate space).
pub(super) fn update_arcane_rune_text(
    time: Res<Time>,
    bg_query: Query<&ComputedNode, With<super::components::ArcaneRuneBackground>>,
    mut text_query: Query<(&ArcaneRuneText, &mut Node, &mut UiTransform, &ComputedNode)>,
) {
    let Ok(bg_computed) = bg_query.single() else {
        return;
    };
    let size = bg_computed.size();
    if size.x < 1.0 || size.y < 1.0 {
        return;
    }
    let ratio = size.y / size.x;
    let elapsed = time.elapsed_secs();

    for (rune_text, mut node, mut ui_transform, text_computed) in &mut text_query {
        let current_angle = rune_text.angle + elapsed * rune_text.speed;
        let text_size = text_computed.size();
        let half_w_pct = text_size.x / size.x * 50.0;
        let half_h_pct = text_size.y / size.y * 50.0;
        let pct_x = 50.0 + rune_text.radius * current_angle.cos() * ratio * 100.0 - half_w_pct;
        let pct_y = 50.0 + rune_text.radius * current_angle.sin() * 100.0 - half_h_pct;
        node.left = Val::Percent(pct_x);
        node.top = Val::Percent(pct_y);
        ui_transform.rotation = Rot2::radians(current_angle + std::f32::consts::FRAC_PI_2);
    }
}

// ---------------------------------------------------------------------------
// Debug: F3 toggle UI visibility
// ---------------------------------------------------------------------------

/// Toggles visibility of the wizard tower UI panels so the background is visible.
#[cfg(debug_assertions)]
pub(super) fn toggle_debug_background(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_query: Query<&mut Visibility, With<super::components::WizardTowerUiContent>>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        for mut vis in &mut ui_query {
            *vis = match *vis {
                Visibility::Hidden => Visibility::Inherited,
                _ => Visibility::Hidden,
            };
        }
    }
}

// ---------------------------------------------------------------------------
// Cleanup
// ---------------------------------------------------------------------------

/// Removes wizard tower tab resources when exiting the meta-game.
pub(super) fn cleanup_wizard_tower_tab_resources(mut commands: Commands) {
    commands.remove_resource::<WizardTowerTab>();
    commands.remove_resource::<RightPanelView>();
}
