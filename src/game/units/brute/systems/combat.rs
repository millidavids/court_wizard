use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use crate::game::pathfinding::StagingAttacker;
use crate::game::terrain::boulder::constants::{
    BOULDER_SPRITE_COUNT, ROCK_THROW_COOLDOWN, ROCK_THROW_RANGE,
};
use crate::game::terrain::boulder::messages::BoulderThrownMessage;
use crate::game::units::components::{
    BanishedModifier, Corpse, FrozenSolidModifier, PolymorphedModifier, RootedModifier,
    SickenedModifier, SleepModifier, Sleepwalking, Team,
};
use crate::game::units::wizard::components::Wizard;

/// Brute rock throw — picks a target enemy within range and throws a rock at them.
#[allow(clippy::type_complexity)]
pub(in crate::game) fn brute_rock_throw(
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut rock_events: MessageWriter<BoulderThrownMessage>,
    mut brutes: Query<
        (
            &Transform,
            &Team,
            &mut RockThrowCooldown,
            (
                Option<&RootedModifier>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                Option<&crate::game::units::components::Petrified>,
                Option<&PolymorphedModifier>,
            ),
        ),
        (With<Brute>, Without<Corpse>),
    >,
    targets: Query<
        (&Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
            Without<Wizard>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (
        brute_transform,
        brute_team,
        mut cooldown,
        (
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
            polymorphed,
        ),
    ) in &mut brutes
    {
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
        ) || polymorphed.is_some()
        {
            continue;
        }

        cooldown.tick(delta);
        if !cooldown.is_ready() {
            continue;
        }

        if let Some(target_pos) = crate::game::units::systems::find_closest_enemy_in_range(
            brute_transform.translation,
            brute_team,
            ROCK_THROW_RANGE,
            &targets,
        ) {
            rock_events.write(BoulderThrownMessage {
                origin: brute_transform.translation,
                target: target_pos,
                sprite_index: game_rng.0.random_range(0..BOULDER_SPRITE_COUNT as u8),
            });
            cooldown.reset(ROCK_THROW_COOLDOWN);
        }
    }
}
