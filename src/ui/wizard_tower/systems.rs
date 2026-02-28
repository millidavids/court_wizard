use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::config::ActiveSave;
use crate::config::save_data::{
    add_spell_research_progress, get_insight, get_spell_research_progress, load_unified_save,
    spend_insight,
};
#[cfg(debug_assertions)]
use crate::config::save_data::grant_insight;
use crate::game::crt_effect::ChannelChangeMessage;
use crate::game::input::messages::MouseClicked;
use crate::game::messages::SpellResearchedMessage;
use crate::game::resources::{BattleInsightData, CurrentLevel, KillStats};
use crate::game::units::DamageType;
use crate::game::units::wizard::components::Spell;
use crate::state::{AppState, MetaGameState};
use crate::ui::main_menu::settings::components::SliderAdjusted;
use crate::ui::systems::spawn_button;

use super::components::*;
use super::constants::*;

// ===========================================================================
// Shared helpers
// ===========================================================================

/// Returns a scaled font size for spell names that may overflow their container.
/// Shrinks the font when the longest word exceeds `max_chars` characters.
fn scaled_spell_name_font_size(name: &str, base_size: f32, max_chars: usize) -> f32 {
    let longest_word = name.split_whitespace().map(|w| w.len()).max().unwrap_or(0);
    if longest_word > max_chars {
        base_size * max_chars as f32 / longest_word as f32
    } else {
        base_size
    }
}

/// Element chain definition for organizing the research UI.
struct SpellChain {
    label: &'static str,
    color: Color,
    spells: &'static [Spell],
}

/// All spell chains displayed in the research UI.
const CHAINS: &[SpellChain] = &[
    SpellChain {
        label: "Fire",
        color: FIRE_COLOR,
        spells: &[
            Spell::Disintegrate,
            Spell::Fireball,
            Spell::WallOfFire,
            Spell::MeteorFall,
        ],
    },
    SpellChain {
        label: "Nature",
        color: NATURE_COLOR,
        spells: &[Spell::Grease, Spell::Entangle, Spell::SpikeGrowth],
    },
    SpellChain {
        label: "",
        color: NATURE_COLOR,
        spells: &[Spell::HealingPlume],
    },
    SpellChain {
        label: "Electric",
        color: ELECTRIC_COLOR,
        spells: &[Spell::ChainLightning, Spell::LightningRod],
    },
    SpellChain {
        label: "Necrotic",
        color: NECROTIC_COLOR,
        spells: &[Spell::FingerOfDeath, Spell::RaiseTheDead],
    },
    SpellChain {
        label: "",
        color: NECROTIC_COLOR,
        spells: &[Spell::MarkOfDeath, Spell::PlagueWind],
    },
    SpellChain {
        label: "Force",
        color: FORCE_COLOR,
        spells: &[Spell::GuardianCircle, Spell::Haste, Spell::BattleHymn],
    },
    SpellChain {
        label: "",
        color: FORCE_COLOR,
        spells: &[Spell::BerserkerRage],
    },
    SpellChain {
        label: "Earth",
        color: EARTH_COLOR,
        spells: &[Spell::WallOfStone, Spell::Teleport],
    },
];

/// Miscellaneous spells (not in chains, require N total spells researched).
const MISC_SPELLS: &[Spell] = &[
    Spell::Squall,
    Spell::FogCloud,
    Spell::Sleep,
    Spell::Banishment,
    Spell::BlackHole,
    Spell::Polymorph,
    Spell::ArcaneCrystal,
    Spell::Dispel,
];

/// Returns the number of spells the player has fully researched.
fn count_researched_spells() -> u32 {
    let save = load_unified_save();
    let unlocked: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();

    Spell::researchable()
        .iter()
        .filter(|spell| {
            let name = format!("{:?}", spell);
            unlocked.contains(&name)
        })
        .count() as u32
}

/// Returns true if a spell's prerequisite is met.
fn is_prereq_met(spell: Spell) -> bool {
    let save = load_unified_save();
    let unlocked: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();

    if let Some(prereq) = spell.prerequisite() {
        let prereq_name = format!("{:?}", prereq);
        if !unlocked.contains(&prereq_name) {
            return false;
        }
    }

    let required = spell.required_total_spells();
    if required > 0 {
        let researched = count_researched_spells();
        if researched < required {
            return false;
        }
    }

    true
}

/// Returns true if this spell is fully researched (unlocked).
fn is_spell_unlocked(spell: Spell) -> bool {
    let save = load_unified_save();
    let unlocked: Vec<String> = save
        .map(|s| s.player.unlocked_content.spells)
        .unwrap_or_default();
    let name = format!("{:?}", spell);
    unlocked.contains(&name)
}

/// Returns the color associated with a damage type for UI display.
fn element_color(damage_type: DamageType) -> Color {
    match damage_type {
        DamageType::Fire => FIRE_COLOR,
        DamageType::Nature => NATURE_COLOR,
        DamageType::Electric => ELECTRIC_COLOR,
        DamageType::Necrotic => NECROTIC_COLOR,
        DamageType::Force => FORCE_COLOR,
        DamageType::Frost => FROST_COLOR,
    }
}

// ===========================================================================
// Hub (MetaGameState::WizardTower) systems
// ===========================================================================

/// Sets up the wizard tower main hub screen.
pub(super) fn setup_wizard_tower_main(mut commands: Commands, current_level: Res<CurrentLevel>) {
    let insight_balance = get_insight();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnWizardTowerScreen,
            OnMainScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Wizard's Tower"),
                TextFont::from_font_size(TITLE_FONT_SIZE),
                TextColor(TITLE_COLOR),
            ));

            // Level display
            #[cfg(debug_assertions)]
            parent.spawn((
                Text::new(format!("Level {}", current_level.0)),
                TextFont::from_font_size(LEVEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
                LevelDisplay,
            ));
            #[cfg(not(debug_assertions))]
            parent.spawn((
                Text::new(format!("Level {}", current_level.0)),
                TextFont::from_font_size(LEVEL_FONT_SIZE),
                TextColor(TEXT_COLOR),
            ));

            // Insight balance
            parent.spawn((
                Text::new(format!("Arcane Insight: {}", insight_balance)),
                TextFont::from_font_size(INSIGHT_FONT_SIZE),
                TextColor(INSIGHT_COLOR),
                InsightDisplay,
            ));

            // Buttons
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|buttons| {
                    spawn_button(
                        buttons,
                        "Start Next Battle",
                        WizardTowerButtonAction::StartNextBattle,
                        &BUTTON_STYLE,
                    );

                    spawn_button(
                        buttons,
                        "Study Spells",
                        WizardTowerButtonAction::StudySpells,
                        &BUTTON_STYLE,
                    );

                    spawn_button(
                        buttons,
                        "Return to Menu",
                        WizardTowerButtonAction::ReturnToMenu,
                        &BUTTON_STYLE,
                    );

                    // Debug level controls
                    #[cfg(debug_assertions)]
                    {
                        buttons
                            .spawn(Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(10.0),
                                margin: UiRect::top(Val::Px(10.0)),
                                ..default()
                            })
                            .with_children(|row| {
                                spawn_button(
                                    row,
                                    "Level -1",
                                    WizardTowerButtonAction::DebugLevelDown,
                                    &DEBUG_BUTTON_STYLE,
                                );
                                spawn_button(
                                    row,
                                    "Level +1",
                                    WizardTowerButtonAction::DebugLevelUp,
                                    &DEBUG_BUTTON_STYLE,
                                );
                            });
                    }
                });
        });
}

/// Cleans up main hub screen entities.
pub(super) fn cleanup_main_screen(
    mut commands: Commands,
    query: Query<Entity, With<OnMainScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handles button actions on the hub screen.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_main_button_actions(
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&WizardTowerButtonAction>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_wt_state: ResMut<NextState<MetaGameState>>,
    mut kill_stats: ResMut<KillStats>,
    mut active_save: ResMut<ActiveSave>,
    #[cfg(debug_assertions)] mut current_level: ResMut<CurrentLevel>,
    #[cfg(debug_assertions)] mut config: ResMut<crate::config::GameConfig>,
    #[cfg(debug_assertions)] mut level_texts: Query<&mut Text, With<LevelDisplay>>,
    mut channel_change: MessageWriter<ChannelChangeMessage>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                WizardTowerButtonAction::StudySpells => {
                    next_wt_state.set(MetaGameState::Study);
                }
                WizardTowerButtonAction::StartNextBattle => {
                    channel_change.write(ChannelChangeMessage);
                    kill_stats.reset();
                    next_app_state.set(AppState::Loading);
                }
                WizardTowerButtonAction::ReturnToMenu => {
                    channel_change.write(ChannelChangeMessage);
                    kill_stats.reset();
                    active_save.0 = None;
                    next_app_state.set(AppState::MainMenu);
                }
                #[cfg(debug_assertions)]
                WizardTowerButtonAction::DebugLevelUp => {
                    current_level.0 += 1;
                    config.current_level = current_level.0;
                    for mut text in &mut level_texts {
                        text.0 = format!("Level {}", current_level.0);
                    }
                }
                #[cfg(debug_assertions)]
                WizardTowerButtonAction::DebugLevelDown => {
                    if current_level.0 > 1 {
                        current_level.0 -= 1;
                        config.current_level = current_level.0;
                        for mut text in &mut level_texts {
                            text.0 = format!("Level {}", current_level.0);
                        }
                    }
                }
            }
        }
    }
}

// ===========================================================================
// Study (MetaGameState::Study) systems
// ===========================================================================

/// Sets up the study screen with scrollable spell grid and allocation sliders.
pub(super) fn setup_study_screen(
    mut commands: Commands,
    battle_insight: Res<BattleInsightData>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(InsightAllocation::default());

    let insight_balance = get_insight();
    let affinities = &battle_insight.damage_types_used;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnWizardTowerScreen,
            OnStudyScreen,
        ))
        .with_children(|parent| {
            // Header row
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("Study Spells"),
                        TextFont::from_font_size(TITLE_FONT_SIZE),
                        TextColor(TITLE_COLOR),
                    ));

                    header.spawn((
                        Text::new(format!("Arcane Insight: {}", insight_balance)),
                        TextFont::from_font_size(INSIGHT_FONT_SIZE),
                        TextColor(INSIGHT_COLOR),
                        StudyInsightDisplay,
                    ));

                    header.spawn((
                        Text::new("Pending: 0"),
                        TextFont::from_font_size(INSIGHT_FONT_SIZE),
                        TextColor(PENDING_COLOR),
                        PendingInsightDisplay,
                    ));
                });

            // Scrollable research area
            parent
                .spawn((
                    Node {
                        width: Val::Percent(SCROLL_CONTAINER_WIDTH_PCT),
                        flex_grow: 1.0,
                        overflow: Overflow::scroll_y(),
                        border: UiRect::all(Val::Px(FRAME_BORDER_WIDTH)),
                        padding: UiRect::all(Val::Px(FRAME_PADDING)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        ..default()
                    },
                    BorderColor::all(FRAME_BORDER_COLOR),
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(FRAME_BACKGROUND),
                    ScrollPosition::default(),
                    ScrollableResearchContainer,
                ))
                .with_children(|scroll| {
                    for chain in CHAINS {
                        spawn_chain_row(scroll, chain, affinities, &asset_server);
                    }
                    spawn_misc_section(scroll, affinities, &asset_server);
                });

            // Footer buttons
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                })
                .with_children(|footer| {
                    spawn_button(
                        footer,
                        "Commit",
                        StudyButtonAction::Commit,
                        &COMMIT_BUTTON_STYLE,
                    );

                    spawn_button(footer, "Back", StudyButtonAction::Back, &BACK_BUTTON_STYLE);

                    #[cfg(debug_assertions)]
                    spawn_button(
                        footer,
                        "+10000 Insight",
                        StudyButtonAction::DebugGrantInsight,
                        &DEBUG_BUTTON_STYLE,
                    );
                });
        });
}

/// Cleans up study screen entities and resources.
pub(super) fn cleanup_study_screen(
    mut commands: Commands,
    query: Query<Entity, With<OnStudyScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<InsightAllocation>();
}

/// Handles Commit and Back button actions on the study screen.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_study_button_actions(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&StudyButtonAction>,
    mut next_wt_state: ResMut<NextState<MetaGameState>>,
    allocation: Option<Res<InsightAllocation>>,
    battle_insight: Res<BattleInsightData>,
    mut spell_researched: MessageWriter<SpellResearchedMessage>,
    screen_query: Query<Entity, With<OnStudyScreen>>,
    asset_server: Res<AssetServer>,
) {
    for event in button_clicked.read() {
        let Ok(action) = button_query.get(event.button) else {
            continue;
        };

        match action {
            StudyButtonAction::Back => {
                next_wt_state.set(MetaGameState::WizardTower);
            }
            StudyButtonAction::Commit => {
                let Some(alloc) = &allocation else {
                    continue;
                };

                let total = alloc.total_allocated();
                if total == 0 {
                    continue;
                }

                // Spend all insight at once
                if !spend_insight(total) {
                    continue;
                }

                // Apply progress per spell
                let affinities = &battle_insight.damage_types_used;
                let mut newly_unlocked = Vec::new();

                for (spell, &amount) in &alloc.allocations {
                    if amount == 0 {
                        continue;
                    }

                    let has_affinity = affinities.contains(&spell.damage_type());
                    let actual_progress = if has_affinity { amount * 2 } else { amount };

                    let unlocked = add_spell_research_progress(*spell, actual_progress);
                    if unlocked {
                        newly_unlocked.push(*spell);
                    }
                }

                // Fire popup messages for newly unlocked spells
                for spell in newly_unlocked {
                    spell_researched.write(SpellResearchedMessage { spell });
                }

                // Despawn and rebuild study screen to reflect new state
                for entity in &screen_query {
                    commands.entity(entity).despawn();
                }
                commands.remove_resource::<InsightAllocation>();

                // Re-setup will happen via OnEnter since we stay in Study state.
                // Instead, rebuild inline.
                rebuild_study_screen(&mut commands, &battle_insight, &asset_server);
            }
            #[cfg(debug_assertions)]
            StudyButtonAction::DebugGrantInsight => {
                grant_insight(10000);

                // Rebuild UI to show updated balance
                for entity in &screen_query {
                    commands.entity(entity).despawn();
                }
                commands.remove_resource::<InsightAllocation>();
                rebuild_study_screen(&mut commands, &battle_insight, &asset_server);
            }
        }
    }
}

/// Rebuilds the study screen after a commit (stays in Study state).
fn rebuild_study_screen(
    commands: &mut Commands,
    battle_insight: &BattleInsightData,
    asset_server: &AssetServer,
) {
    commands.insert_resource(InsightAllocation::default());

    let insight_balance = get_insight();
    let affinities = &battle_insight.damage_types_used;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(BACKGROUND_COLOR),
            OnWizardTowerScreen,
            OnStudyScreen,
        ))
        .with_children(|parent| {
            // Header row
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|header| {
                    header.spawn((
                        Text::new("Study Spells"),
                        TextFont::from_font_size(TITLE_FONT_SIZE),
                        TextColor(TITLE_COLOR),
                    ));

                    header.spawn((
                        Text::new(format!("Arcane Insight: {}", insight_balance)),
                        TextFont::from_font_size(INSIGHT_FONT_SIZE),
                        TextColor(INSIGHT_COLOR),
                        StudyInsightDisplay,
                    ));

                    header.spawn((
                        Text::new("Pending: 0"),
                        TextFont::from_font_size(INSIGHT_FONT_SIZE),
                        TextColor(PENDING_COLOR),
                        PendingInsightDisplay,
                    ));
                });

            // Scrollable research area
            parent
                .spawn((
                    Node {
                        width: Val::Percent(SCROLL_CONTAINER_WIDTH_PCT),
                        flex_grow: 1.0,
                        overflow: Overflow::scroll_y(),
                        border: UiRect::all(Val::Px(FRAME_BORDER_WIDTH)),
                        padding: UiRect::all(Val::Px(FRAME_PADDING)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        ..default()
                    },
                    BorderColor::all(FRAME_BORDER_COLOR),
                    BorderRadius::all(Val::Px(8.0)),
                    BackgroundColor(FRAME_BACKGROUND),
                    ScrollPosition::default(),
                    ScrollableResearchContainer,
                ))
                .with_children(|scroll| {
                    for chain in CHAINS {
                        spawn_chain_row(scroll, chain, affinities, asset_server);
                    }
                    spawn_misc_section(scroll, affinities, asset_server);
                });

            // Footer buttons
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                })
                .with_children(|footer| {
                    spawn_button(
                        footer,
                        "Commit",
                        StudyButtonAction::Commit,
                        &COMMIT_BUTTON_STYLE,
                    );

                    spawn_button(footer, "Back", StudyButtonAction::Back, &BACK_BUTTON_STYLE);

                    #[cfg(debug_assertions)]
                    spawn_button(
                        footer,
                        "+10000 Insight",
                        StudyButtonAction::DebugGrantInsight,
                        &DEBUG_BUTTON_STYLE,
                    );
                });
        });
}

// ===========================================================================
// Spell card spawning helpers
// ===========================================================================

/// Spawns a horizontal chain row: label + spell cards with arrows between them.
fn spawn_chain_row(
    parent: &mut ChildSpawnerCommands,
    chain: &SpellChain,
    affinities: &std::collections::HashSet<DamageType>,
    asset_server: &AssetServer,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            // Chain element label
            row.spawn(Node {
                width: Val::Px(70.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|label_container| {
                label_container.spawn((
                    Text::new(chain.label),
                    TextFont::from_font_size(CHAIN_LABEL_FONT_SIZE),
                    TextColor(chain.color),
                ));
            });

            for (i, spell) in chain.spells.iter().enumerate() {
                if i > 0 {
                    row.spawn((
                        Text::new(">"),
                        TextFont::from_font_size(18.0),
                        TextColor(ARROW_COLOR),
                    ));
                }

                spawn_spell_card(row, *spell, affinities, asset_server);
            }
        });
}

/// Spawns the miscellaneous spells section.
fn spawn_misc_section(
    parent: &mut ChildSpawnerCommands,
    affinities: &std::collections::HashSet<DamageType>,
    asset_server: &AssetServer,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                width: Val::Px(70.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|label_container| {
                label_container.spawn((
                    Text::new("Misc"),
                    TextFont::from_font_size(CHAIN_LABEL_FONT_SIZE),
                    TextColor(MISC_COLOR),
                ));
            });

            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(8.0),
                row_gap: Val::Px(8.0),
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                ..default()
            })
            .with_children(|grid| {
                for spell in MISC_SPELLS {
                    spawn_spell_card(grid, *spell, affinities, asset_server);
                }
            });
        });
}

/// Spawns a single spell research card.
fn spawn_spell_card(
    parent: &mut ChildSpawnerCommands,
    spell: Spell,
    affinities: &std::collections::HashSet<DamageType>,
    asset_server: &AssetServer,
) {
    let unlocked = is_spell_unlocked(spell);
    let prereq_met = is_prereq_met(spell);
    let progress = get_spell_research_progress(spell);
    let cost = spell.research_cost();
    let has_affinity = affinities.contains(&spell.damage_type());

    let (bg_color, border_color) = if unlocked {
        (CARD_BACKGROUND, CARD_COMPLETED_BORDER)
    } else if has_affinity && prereq_met {
        (CARD_BACKGROUND, CARD_AFFINITY_BORDER)
    } else if prereq_met {
        (CARD_BACKGROUND, CARD_BORDER)
    } else {
        (CARD_LOCKED_BACKGROUND, CARD_LOCKED_BORDER)
    };

    parent
        .spawn((
            Node {
                width: Val::Px(CARD_WIDTH),
                min_height: Val::Px(CARD_HEIGHT),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(CARD_BORDER_WIDTH)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor::all(border_color),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .with_children(|card| {
            let name_font_size =
                scaled_spell_name_font_size(spell.display_name(), SPELL_NAME_FONT_SIZE, 10);

            if unlocked {
                // Researched: show icon (if available) + name + element + "Researched"
                if let Some(icon_path) = spell.icon_path() {
                    card.spawn((
                        ImageNode::new(asset_server.load(icon_path)),
                        Node {
                            width: Val::Px(SPELL_ICON_SIZE),
                            height: Val::Px(SPELL_ICON_SIZE),
                            ..default()
                        },
                    ));
                }

                card.spawn((
                    Text::new(spell.display_name()),
                    TextFont::from_font_size(name_font_size),
                    TextColor(COMPLETED_COLOR),
                ));

                card.spawn((
                    Text::new(spell.damage_type().display_name()),
                    TextFont::from_font_size(SPELL_DETAIL_FONT_SIZE),
                    TextColor(element_color(spell.damage_type())),
                ));

                card.spawn((
                    Text::new("Researched"),
                    TextFont::from_font_size(SPELL_DETAIL_FONT_SIZE),
                    TextColor(COMPLETED_COLOR),
                ));

                card.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
            } else if prereq_met {
                // Available: show icon (if available) + name, element+affinity, progress bar, slider
                if let Some(icon_path) = spell.icon_path() {
                    card.spawn((
                        ImageNode::new(asset_server.load(icon_path)),
                        Node {
                            width: Val::Px(SPELL_ICON_SIZE),
                            height: Val::Px(SPELL_ICON_SIZE),
                            ..default()
                        },
                    ));
                }

                card.spawn((
                    Text::new(spell.display_name()),
                    TextFont::from_font_size(name_font_size),
                    TextColor(TEXT_COLOR),
                ));

                let element_text = if has_affinity {
                    format!("{} (2x)", spell.damage_type().display_name())
                } else {
                    spell.damage_type().display_name().to_string()
                };
                let element_text_color = if has_affinity {
                    AFFINITY_COLOR
                } else {
                    element_color(spell.damage_type())
                };

                card.spawn((
                    Text::new(element_text),
                    TextFont::from_font_size(SPELL_DETAIL_FONT_SIZE),
                    TextColor(element_text_color),
                ));

                // Progress bar (shows existing research)
                let fill_pct = if cost > 0 {
                    (progress as f32 / cost as f32 * 100.0).min(100.0)
                } else {
                    100.0
                };
                spawn_progress_bar(card, fill_pct);

                // Allocation slider
                let remaining = cost.saturating_sub(progress);
                spawn_allocation_slider(card, spell, 0, remaining);

                // Allocation text: "current+pending / total"
                card.spawn((
                    Text::new(format!("{}/{}", progress, cost)),
                    TextFont::from_font_size(SPELL_DETAIL_FONT_SIZE),
                    TextColor(TEXT_COLOR),
                    AllocationText { spell },
                ));
            } else {
                // Locked: show only flavor text (no name!)
                card.spawn((
                    Text::new(spell.locked_description()),
                    TextFont::from_font_size(SPELL_DETAIL_FONT_SIZE),
                    TextColor(LOCKED_TEXT_COLOR),
                    Node {
                        margin: UiRect::top(Val::Px(4.0)),
                        ..default()
                    },
                ));

                card.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
            }
        });
}

/// Spawns a progress bar showing existing research progress.
fn spawn_progress_bar(parent: &mut ChildSpawnerCommands, fill_pct: f32) {
    let fill_color = if fill_pct >= 100.0 {
        PROGRESS_BAR_FULL
    } else {
        PROGRESS_BAR_FILL
    };

    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(PROGRESS_BAR_HEIGHT),
                ..default()
            },
            BackgroundColor(PROGRESS_BAR_BACKGROUND),
            BorderRadius::all(Val::Px(3.0)),
        ))
        .with_children(|bg| {
            bg.spawn((
                Node {
                    width: Val::Percent(fill_pct),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(fill_color),
                BorderRadius::all(Val::Px(3.0)),
            ));
        });
}

/// Spawns an allocation slider for a spell.
fn spawn_allocation_slider(
    parent: &mut ChildSpawnerCommands,
    spell: Spell,
    current_alloc: u32,
    max_alloc: u32,
) {
    let normalized = if max_alloc > 0 {
        current_alloc as f32 / max_alloc as f32
    } else {
        0.0
    };

    parent
        .spawn((
            Node {
                width: Val::Px(SLIDER_TRACK_WIDTH),
                height: Val::Px(SLIDER_TRACK_HEIGHT),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                position_type: PositionType::Relative,
                ..default()
            },
            BorderColor::all(SLIDER_TRACK_BORDER),
            BorderRadius::all(Val::Px(5.0)),
            BackgroundColor(SLIDER_TRACK_BG),
            Interaction::default(),
            RelativeCursorPosition::default(),
            AllocationSliderTrack { spell },
        ))
        .with_children(|track| {
            // Fill
            track.spawn((
                Node {
                    width: Val::Percent(normalized * 100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BorderRadius {
                    top_left: Val::Px(5.0),
                    bottom_left: Val::Px(5.0),
                    top_right: Val::Px(0.0),
                    bottom_right: Val::Px(0.0),
                },
                BackgroundColor(SLIDER_FILL_COLOR),
                AllocationSliderFill { spell },
            ));

            // Handle
            track.spawn((
                Node {
                    width: Val::Px(SLIDER_HANDLE_WIDTH),
                    height: Val::Px(SLIDER_HANDLE_HEIGHT),
                    position_type: PositionType::Absolute,
                    left: Val::Px(normalized * SLIDER_TRACK_WIDTH - SLIDER_HANDLE_WIDTH / 2.0),
                    top: Val::Px(-(SLIDER_HANDLE_HEIGHT - SLIDER_TRACK_HEIGHT) / 2.0),
                    ..default()
                },
                BorderRadius::all(Val::Px(2.0)),
                BackgroundColor(SLIDER_HANDLE_COLOR),
                Interaction::default(),
                RelativeCursorPosition::default(),
                AllocationSliderHandle {
                    spell,
                    is_dragging: false,
                },
            ));
        });
}

// ===========================================================================
// Slider interaction systems (study screen)
// ===========================================================================

/// Handles click and drag on allocation slider tracks using cursor position.
///
/// While the mouse button is held down on a track, the slider continuously
/// tracks the cursor's x position on the track to set the allocation value.
pub(super) fn handle_allocation_slider_interaction(
    buttons: Res<ButtonInput<MouseButton>>,
    mut slider_handles: Query<(&Interaction, &mut AllocationSliderHandle)>,
    slider_tracks: Query<(
        &Interaction,
        &RelativeCursorPosition,
        &AllocationSliderTrack,
    )>,
    mut allocation: ResMut<InsightAllocation>,
    mut slider_adjusted: MessageWriter<SliderAdjusted>,
) {
    let insight_balance = get_insight();

    // On initial click, mark which handle is being dragged
    if buttons.just_pressed(MouseButton::Left) {
        for (interaction, cursor_pos, track) in &slider_tracks {
            if !matches!(*interaction, Interaction::Pressed | Interaction::Hovered) {
                continue;
            }
            if cursor_pos.normalized.is_none() {
                continue;
            }

            for (_hi, mut handle) in &mut slider_handles {
                if handle.spell == track.spell {
                    handle.is_dragging = true;
                }
            }
        }

        // Also check direct handle clicks
        for (interaction, mut handle) in &mut slider_handles {
            if *interaction == Interaction::Pressed {
                handle.is_dragging = true;
            }
        }
    }

    // Release mouse → stop all dragging
    if !buttons.pressed(MouseButton::Left) {
        for (_interaction, mut handle) in &mut slider_handles {
            handle.is_dragging = false;
        }
        return;
    }

    // While mouse is held, find which handle is dragging and read cursor
    // position from its parent track to set allocation.
    let mut dragging_spell: Option<Spell> = None;
    for (_interaction, handle) in &slider_handles {
        if handle.is_dragging {
            dragging_spell = Some(handle.spell);
            break;
        }
    }

    let Some(spell) = dragging_spell else {
        return;
    };

    // Find the track for this spell and read cursor position
    for (_interaction, cursor_pos, track) in &slider_tracks {
        if track.spell != spell {
            continue;
        }

        let Some(pos) = cursor_pos.normalized else {
            continue;
        };

        let cost = spell.research_cost();
        let progress = get_spell_research_progress(spell);
        let remaining = cost.saturating_sub(progress);

        if remaining == 0 {
            continue;
        }

        // RelativeCursorPosition.normalized: center=(0,0), left=-0.5, right=0.5
        let normalized = (pos.x + 0.5).clamp(0.0, 1.0);
        let desired = (normalized * remaining as f32).round() as u32;

        // Cap by available insight (total balance minus other spell allocations)
        let others: u32 = allocation
            .allocations
            .iter()
            .filter(|(s, _)| **s != spell)
            .map(|(_, v)| *v)
            .sum();
        let max_for_spell = insight_balance.saturating_sub(others).min(remaining);
        let clamped = desired.min(max_for_spell);

        let old = allocation.get(&spell);
        if clamped != old {
            allocation.set(spell, clamped);
            slider_adjusted.write(SliderAdjusted);
        }
    }
}

/// Updates slider fill widths and handle positions when InsightAllocation changes.
pub(super) fn update_allocation_sliders(
    allocation: Res<InsightAllocation>,
    mut slider_fills: Query<(&mut Node, &AllocationSliderFill), Without<AllocationSliderHandle>>,
    mut slider_handles: Query<(&mut Node, &AllocationSliderHandle), Without<AllocationSliderFill>>,
) {
    if !allocation.is_changed() {
        return;
    }

    for (mut node, fill) in &mut slider_fills {
        let spell = fill.spell;
        let cost = spell.research_cost();
        let progress = get_spell_research_progress(spell);
        let remaining = cost.saturating_sub(progress);
        let alloc = allocation.get(&spell);

        let normalized = if remaining > 0 {
            alloc as f32 / remaining as f32
        } else {
            0.0
        };

        node.width = Val::Percent(normalized * 100.0);
    }

    for (mut node, handle) in &mut slider_handles {
        let spell = handle.spell;
        let cost = spell.research_cost();
        let progress = get_spell_research_progress(spell);
        let remaining = cost.saturating_sub(progress);
        let alloc = allocation.get(&spell);

        let normalized = if remaining > 0 {
            alloc as f32 / remaining as f32
        } else {
            0.0
        };

        node.left = Val::Px(normalized * SLIDER_TRACK_WIDTH - SLIDER_HANDLE_WIDTH / 2.0);
    }
}

/// Updates "current+pending / total" text for each spell allocation.
pub(super) fn update_allocation_text(
    allocation: Res<InsightAllocation>,
    battle_insight: Res<BattleInsightData>,
    mut texts: Query<(&mut Text, &AllocationText)>,
) {
    if !allocation.is_changed() {
        return;
    }

    let affinities = &battle_insight.damage_types_used;

    for (mut text, alloc_text) in &mut texts {
        let spell = alloc_text.spell;
        let cost = spell.research_cost();
        let progress = get_spell_research_progress(spell);
        let alloc = allocation.get(&spell);

        let has_affinity = affinities.contains(&spell.damage_type());
        let effective = if has_affinity { alloc * 2 } else { alloc };

        if alloc > 0 {
            text.0 = format!("{}+{}/{}", progress, effective, cost);
        } else {
            text.0 = format!("{}/{}", progress, cost);
        }
    }
}

/// Updates the "Pending: X" display in the study header.
pub(super) fn update_pending_insight_display(
    allocation: Res<InsightAllocation>,
    mut texts: Query<&mut Text, With<PendingInsightDisplay>>,
) {
    if !allocation.is_changed() {
        return;
    }

    let total = allocation.total_allocated();
    for mut text in &mut texts {
        text.0 = format!("Pending: {}", total);
    }
}

// ===========================================================================
// Top-level cleanup (exiting WizardTower entirely)
// ===========================================================================

/// Cleans up all wizard tower screen entities when leaving AppState::MetaGame.
pub(super) fn cleanup_wizard_tower_screen(
    mut commands: Commands,
    query: Query<Entity, With<OnWizardTowerScreen>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<InsightAllocation>();
}
