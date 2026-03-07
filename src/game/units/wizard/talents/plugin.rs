use bevy::prelude::*;

use crate::state::{AppState, InGameState};

use super::resources::{ActiveTalents, BattleTalentProgress};

pub struct TalentsPlugin;

impl Plugin for TalentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InGameState::Running), init_talent_resources)
            .add_systems(OnExit(AppState::InGame), cleanup_talent_resources)
            .add_systems(OnExit(AppState::MultiplayerGame), cleanup_talent_resources);
    }
}

/// Initialize talent resources when entering gameplay.
fn init_talent_resources(mut commands: Commands) {
    commands.insert_resource(ActiveTalents::from_save());
    commands.insert_resource(BattleTalentProgress::default());
}

/// Clean up talent resources when leaving the game.
fn cleanup_talent_resources(mut commands: Commands) {
    commands.remove_resource::<ActiveTalents>();
    commands.remove_resource::<BattleTalentProgress>();
}
