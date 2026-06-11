//! Multiplayer score screen UI: spawn layout, stat columns, update, and teardown.

use bevy::prelude::*;

use crate::game::resources::GameOutcome;
use crate::ui::components::ButtonStyle;
use crate::ui::constants::{BUTTON_BG, BUTTON_BORDER, TEXT_PRIMARY};
use crate::ui::systems::spawn_button;

use super::super::components::{
    MpRematchState, MpScoreButtonAction, MpStatValueText, OnMpScoreScreen, OnMultiplayerGameScreen,
    RematchStatusText,
};

// ── Score Screen Constants ────────────────────────────────────────────

pub(super) const SCORE_BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);
pub(super) const SCORE_TITLE_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
pub(super) const SCORE_TEXT_COLOR: Color = Color::srgb(0.85, 0.85, 0.85);

const SCORE_BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 250.0,
    height: 65.0,
    border_width: 3.0,
    font_size: 20.0,
    background: BUTTON_BG,
    border: BUTTON_BORDER,
    text_color: TEXT_PRIMARY,
    text_shadow: true,
};

/// The four scoreboard rows, as `(label, your-side marker, enemy-side marker)`.
const STAT_ROWS: [(&str, MpStatValueText, MpStatValueText); 4] = [
    (
        "Kills",
        MpStatValueText::YourKills,
        MpStatValueText::EnemyKills,
    ),
    (
        "Deaths",
        MpStatValueText::YourDeaths,
        MpStatValueText::EnemyDeaths,
    ),
    (
        "Damage",
        MpStatValueText::YourDamage,
        MpStatValueText::EnemyDamage,
    ),
    (
        "Healed",
        MpStatValueText::YourHealing,
        MpStatValueText::EnemyHealing,
    ),
];

/// Formats a single `MatchStats` field for display. Returns "0" when stats
/// aren't available yet (e.g. the host's enemy column before the guest's
/// `WizardStatsReport` arrives). Damage/healing are shown as whole numbers.
pub(super) fn stat_value_text(
    stats: Option<&super::super::score_stats::MatchStats>,
    which: MpStatValueText,
) -> String {
    let Some(s) = stats else {
        return "0".to_string();
    };
    match which {
        MpStatValueText::YourKills => s.your_kills.to_string(),
        MpStatValueText::YourDeaths => s.your_deaths.to_string(),
        MpStatValueText::YourDamage => (s.your_damage.round() as u32).to_string(),
        MpStatValueText::YourHealing => (s.your_healing.round() as u32).to_string(),
        MpStatValueText::EnemyKills => s.enemy_kills.to_string(),
        MpStatValueText::EnemyDeaths => s.enemy_deaths.to_string(),
        MpStatValueText::EnemyDamage => (s.enemy_damage.round() as u32).to_string(),
        MpStatValueText::EnemyHealing => (s.enemy_healing.round() as u32).to_string(),
    }
}

/// Spawns one label+value stat row; the value `Text` is tagged with `marker`
/// so `update_mp_stat_values` can refresh it reactively.
fn spawn_stat_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: MpStatValueText,
    stats: Option<&super::super::score_stats::MatchStats>,
) {
    parent
        .spawn(Node {
            width: Val::Px(200.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: Val::Px(24.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont::from_font_size(20.0),
                TextColor(SCORE_TEXT_COLOR),
            ));
            row.spawn((
                marker,
                Text::new(stat_value_text(stats, marker)),
                TextFont::from_font_size(20.0),
                TextColor(SCORE_TITLE_COLOR),
            ));
        });
}

/// Spawns one stat column (header + the four rows). `is_enemy` selects the
/// enemy-side markers so the value nodes update from the correct `MatchStats`
/// fields.
fn spawn_stat_column(
    parent: &mut ChildSpawnerCommands,
    header: &str,
    is_enemy: bool,
    stats: Option<&super::super::score_stats::MatchStats>,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Text::new(header),
                TextFont::from_font_size(26.0),
                TextColor(SCORE_TITLE_COLOR),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));
            for (label, your_marker, enemy_marker) in STAT_ROWS {
                let marker = if is_enemy { enemy_marker } else { your_marker };
                spawn_stat_row(col, label, marker, stats);
            }
        });
}

/// Spawns the multiplayer score screen UI: a title over three columns —
/// enemy stats, your stats, and the Rematch/Disconnect buttons.
pub(crate) fn setup_mp_score_screen(
    mut commands: Commands,
    game_outcome: Res<GameOutcome>,
    match_stats: Option<Res<super::super::score_stats::MatchStats>>,
) {
    commands.init_resource::<MpRematchState>();

    let title_text = match *game_outcome {
        GameOutcome::Victory => "VICTORY",
        _ => "DEFEAT",
    };
    let stats = match_stats.as_deref();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(SCORE_BG_COLOR),
            OnMpScoreScreen,
            OnMultiplayerGameScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(title_text),
                TextFont::from_font_size(60.0),
                TextColor(SCORE_TITLE_COLOR),
            ));

            // Subtitle for King death
            if *game_outcome == GameOutcome::DefeatKingDied {
                parent.spawn((
                    Text::new("Your King was slain!"),
                    TextFont::from_font_size(24.0),
                    TextColor(SCORE_TEXT_COLOR),
                ));
            }

            // Three columns: enemy stats | your stats | buttons
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(60.0),
                    margin: UiRect::top(Val::Px(16.0)),
                    ..default()
                })
                .with_children(|columns| {
                    spawn_stat_column(columns, "ENEMY", true, stats);
                    spawn_stat_column(columns, "YOU", false, stats);

                    // Right column: buttons + rematch status text
                    columns
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(15.0),
                            ..default()
                        })
                        .with_children(|buttons| {
                            spawn_button(
                                buttons,
                                "Rematch",
                                MpScoreButtonAction::Rematch,
                                &SCORE_BUTTON_STYLE,
                            );
                            spawn_button(
                                buttons,
                                "Disconnect",
                                MpScoreButtonAction::Disconnect,
                                &SCORE_BUTTON_STYLE,
                            );
                            buttons.spawn((
                                RematchStatusText,
                                Text::new(""),
                                TextFont::from_font_size(18.0),
                                TextColor(SCORE_TEXT_COLOR),
                            ));
                        });
                });
        });
}

/// Refreshes the score-screen stat value nodes whenever `MatchStats` changes —
/// notably when the guest's `WizardStatsReport` fills in the host's enemy column.
pub(crate) fn update_mp_stat_values(
    match_stats: Res<super::super::score_stats::MatchStats>,
    mut values: Query<(&MpStatValueText, &mut Text)>,
) {
    for (marker, mut text) in &mut values {
        **text = stat_value_text(Some(&match_stats), *marker);
    }
}

/// Cleans up score screen entities and resources.
pub(crate) fn cleanup_mp_score_screen(
    mut commands: Commands,
    score_entities: Query<Entity, With<OnMpScoreScreen>>,
) {
    for entity in &score_entities {
        if let Ok(mut ec) = commands.get_entity(entity) {
            ec.try_despawn();
        }
    }
    commands.remove_resource::<MpRematchState>();
    commands.remove_resource::<super::super::score_stats::MatchStats>();
}
