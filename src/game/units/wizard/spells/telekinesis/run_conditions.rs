use bevy::prelude::*;

use super::super::super::components::Spell;
use crate::game::units::wizard::talents::resources::ActiveTalents;

/// Run condition: checks if a specific Telekinesis talent is selected at the given tier and choice.
pub(super) fn has_telekinesis_talent(
    tier: usize,
    choice: u8,
) -> impl Fn(Option<Res<ActiveTalents>>) -> bool + Clone {
    move |active_talents: Option<Res<ActiveTalents>>| {
        active_talents
            .as_ref()
            .and_then(|t| t.get_selection(Spell::Telekinesis, tier))
            == Some(choice)
    }
}
