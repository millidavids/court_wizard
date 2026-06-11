use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::OgreAssets;
use super::charge_visuals::ogre_combat_animation;
use crate::game::pathfinding::StagingAttacker;
use crate::game::terrain::boulder::constants::ROCK_THROW_COOLDOWN;
use crate::game::terrain::boulder::messages::BoulderThrownMessage;
use crate::game::units::boss::components::Boss;
use crate::game::units::brute::components::RockThrowCooldown;
use crate::game::units::components::{
    BanishedModifier, CombatAnimation, Corpse, FrozenSolidModifier, PolymorphedModifier,
    RootedModifier, SickenedModifier, SleepModifier, Sleepwalking, Team,
};

/// Ogre rock throw — picks a target enemy within range and starts the throwing animation.
/// The boulder is launched when the animation finishes (see `ogre_throw_release`).
/// Skipped during charge phases or if already winding up.
#[allow(clippy::type_complexity)]
pub fn ogre_rock_throw(
    time: Res<Time>,
    ogre_assets: Res<OgreAssets>,
    game_config: Res<crate::config::GameConfig>,
    mut commands: Commands,
    mut bosses: Query<
        (
            Entity,
            &Transform,
            &Team,
            &OgreChargeState,
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
        (
            With<Boss>,
            Without<Corpse>,
            Without<OgreThrowWindup>,
            Without<CombatAnimation>,
        ),
    >,
    targets: Query<
        (&Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
) {
    use crate::game::terrain::boulder::constants::ROCK_THROW_RANGE;

    let delta = time.delta_secs();

    for (
        entity,
        boss_transform,
        boss_team,
        charge_state,
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
    ) in &mut bosses
    {
        if charge_state.is_movement_locked() {
            continue;
        }
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
            boss_transform.translation,
            boss_team,
            ROCK_THROW_RANGE,
            &targets,
        ) {
            // Play grunt sound effect
            crate::game::units::wizard::spells::audio::play_sfx_scaled(
                &mut commands,
                &ogre_assets.grunt_sfx,
                boss_transform.translation,
                &game_config,
                1.0,
            );

            // Start throwing animation and store target for release
            commands.entity(entity).insert((
                OgreThrowWindup {
                    target: target_pos,
                    sprite_index: 1,
                },
                ogre_combat_animation(
                    OGRE_THROWING_DIRECTION_ROWS,
                    ogre_assets.throwing_texture.clone(),
                    ogre_assets.walking_texture.clone(),
                ),
            ));
            cooldown.reset(ROCK_THROW_COOLDOWN);
        }
    }
}

/// Fires the boulder when the throwing animation finishes.
/// Detects completion by checking for `OgreThrowWindup` without `CombatAnimation`
/// (the shared animation system removes `CombatAnimation` when it's done).
pub fn ogre_throw_release(
    mut commands: Commands,
    mut rock_events: MessageWriter<BoulderThrownMessage>,
    bosses: Query<
        (Entity, &Transform, &OgreThrowWindup),
        (With<Boss>, Without<CombatAnimation>, Without<Corpse>),
    >,
) {
    for (entity, boss_transform, windup) in &bosses {
        rock_events.write(BoulderThrownMessage {
            origin: boss_transform.translation,
            target: windup.target,
            sprite_index: windup.sprite_index,
        });
        commands.entity(entity).remove::<OgreThrowWindup>();
    }
}
