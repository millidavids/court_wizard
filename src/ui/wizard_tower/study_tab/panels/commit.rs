use bevy::prelude::*;

#[cfg(debug_assertions)]
use crate::config::save_data::grant_insight;
use crate::config::save_data::{
    add_insight_bonus_progress, add_spell_research_progress, spend_insight,
};
use crate::game::input::messages::MouseClicked;
use crate::game::insight_bonuses::InsightBonusStat;
use crate::game::messages::{InsightBonusUpgradedMessage, SpellResearchedMessage};
use crate::game::resources::BattleInsightData;

use super::super::super::components::*;
use super::super::super::materials::{
    ConcentricRingsMaterial, RadialProgressMaterial, StarSkyMaterial,
};
use super::spawn::rebuild_study_ui;

/// Handles Commit and Back button actions on the study screen.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_study_button_actions(
    mut commands: Commands,
    mut button_clicked: MessageReader<MouseClicked>,
    button_query: Query<&StudyButtonAction>,
    allocation: Option<Res<InsightAllocation>>,
    battle_insight: Res<BattleInsightData>,
    mut spell_researched: MessageWriter<SpellResearchedMessage>,
    mut insight_upgraded: MessageWriter<InsightBonusUpgradedMessage>,
    left_panel: Query<Entity, With<super::super::super::layout::WizardTowerLeftPanel>>,
    right_panel: Query<Entity, With<super::super::super::layout::WizardTowerRightPanel>>,
    asset_server: Res<AssetServer>,
    mut selected: Option<ResMut<SelectedStudySpell>>,
    mut progress_materials: ResMut<Assets<RadialProgressMaterial>>,
    mut ring_materials: ResMut<Assets<ConcentricRingsMaterial>>,
    mut star_sky_materials: ResMut<Assets<StarSkyMaterial>>,
) {
    for event in button_clicked.read() {
        let Ok(action) = button_query.get(event.button) else {
            continue;
        };

        match action {
            StudyButtonAction::Commit => {
                let Some(alloc) = &allocation else {
                    continue;
                };

                // Clamp bonus allocations to what each node can still absorb
                // BEFORE spending, so insight is never deducted for progress
                // that cannot be banked.
                let bonus_updates: Vec<(&str, u32)> = alloc
                    .bonus_allocations
                    .iter()
                    .map(|(stat, &amount)| {
                        let capacity =
                            InsightBonusStat::remaining_capacity(stat.current_progress());
                        (stat.id(), amount.min(capacity))
                    })
                    .filter(|&(_, amount)| amount > 0)
                    .collect();

                let total: u32 = alloc.allocations.values().sum::<u32>()
                    + bonus_updates.iter().map(|(_, a)| a).sum::<u32>();
                if total == 0 {
                    continue;
                }

                if !spend_insight(total) {
                    continue;
                }

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

                // Bank the insight bonus allocations (insight already spent
                // above). Every point counts toward the node; the level only
                // advances when a whole threshold is crossed.
                if add_insight_bonus_progress(&bonus_updates) {
                    insight_upgraded.write(InsightBonusUpgradedMessage);
                }

                for spell in newly_unlocked {
                    spell_researched.write(SpellResearchedMessage { spell });
                }

                rebuild_study_ui(
                    &mut commands,
                    &left_panel,
                    &right_panel,
                    &mut selected,
                    &battle_insight,
                    &asset_server,
                    &mut progress_materials,
                    &mut ring_materials,
                    &mut star_sky_materials,
                    false, // don't zoom out — keep player's current pan/zoom
                    true,  // keep current selection so left panel refreshes in place
                );
                // Force update_study_detail_panel to re-render the left panel
                // with the spell's new (possibly now-unlocked) state.
                if let Some(sel) = selected.as_mut() {
                    sel.set_changed();
                }
                // `rebuild_study_ui` resets InsightAllocation via deferred
                // commands, so the `alloc` borrow is stale from here on —
                // stop rather than risk re-spending it on another event.
                break;
            }
            #[cfg(debug_assertions)]
            StudyButtonAction::DebugGrantInsight => {
                grant_insight(10000);
                rebuild_study_ui(
                    &mut commands,
                    &left_panel,
                    &right_panel,
                    &mut selected,
                    &battle_insight,
                    &asset_server,
                    &mut progress_materials,
                    &mut ring_materials,
                    &mut star_sky_materials,
                    false,
                    false,
                );
            }
        }
    }
}
