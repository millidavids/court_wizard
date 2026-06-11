use bevy::prelude::*;

use crate::game::messages::TalentTierUnlockedMessage;
use crate::state::{AppState, InGameState, MultiplayerGameState};

use super::systems::{cleanup_talent_resources, init_talent_resources};

pub struct TalentsPlugin;

impl Plugin for TalentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TalentTierUnlockedMessage>()
            .add_systems(OnEnter(InGameState::Running), init_talent_resources)
            // Multiplayer also needs ActiveTalents + BattleTalentProgress —
            // many spell systems (e.g. ArcaneCrystal's detect_*_hits,
            // PlagueWind talent progress, etc.) take `ResMut<BattleTalentProgress>`
            // as a non-optional system param, and the system PANICS if the
            // resource is missing. Without this, host and guest both crash
            // the first time they fire any such system in MP.
            .add_systems(
                OnEnter(MultiplayerGameState::Running),
                init_talent_resources,
            )
            .add_systems(OnExit(AppState::InGame), cleanup_talent_resources)
            .add_systems(OnExit(AppState::MultiplayerGame), cleanup_talent_resources);
    }
}
