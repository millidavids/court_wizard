use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use crate::config::GameConfig;
use crate::game::constants::*;
use crate::game::units::boss::utils::is_on_screen;
use crate::game::units::components::{
    BanishedModifier, Corpse, FrozenSolidModifier, Hitbox, RootedModifier, SickenedModifier,
    SleepModifier, Sleepwalking, Team,
};
use crate::game::units::wizard::spells::audio::{SpellSfxAssets, play_sfx_scaled};
use crate::game::units::wizard::spells::teleport::vfx_systems::spawn_teleport_vfx;
use crate::game::units::wizard::spells::utils::ground_projected_range;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Teleport system: teleports the Dark Mage away when enemy units get into melee range.
/// Has a cooldown to prevent constant teleporting.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn dark_mage_teleport(
    time: Res<Time>,
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    visual_assets: Res<SpellVisualAssets>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut bosses: Query<
        (
            &mut Transform,
            &mut DarkMageTeleportTimer,
            &DarkMageState,
            &Hitbox,
            &Team,
            (
                Option<&RootedModifier>,
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                Option<&crate::game::units::components::Petrified>,
            ),
        ),
        (With<DarkMage>, Without<Corpse>),
    >,
    nearby_units: Query<
        (&Transform, &Hitbox, &Team),
        (
            Without<DarkMage>,
            Without<Corpse>,
            Without<BanishedModifier>,
        ),
    >,
) {
    let Ok((camera, camera_global)) = camera_query.single() else {
        return;
    };

    let delta = time.delta_secs();

    for (
        mut transform,
        mut teleport_timer,
        state,
        hitbox,
        boss_team,
        (rooted, sleeping, sleepwalking, banished, sickened, frozen, stunned, petrified),
    ) in &mut bosses
    {
        // Don't teleport while approaching
        if matches!(state, DarkMageState::Approaching) {
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
        ) {
            continue;
        }

        teleport_timer.tick(delta);

        if !teleport_timer.is_ready() {
            continue;
        }

        // Check if any enemy is in melee range
        let boss_pos = transform.translation;
        let melee_range = (hitbox.radius * ATTACK_RANGE_MULTIPLIER) * 1.5;
        let mut enemy_nearby = false;

        for (unit_transform, unit_hitbox, unit_team) in &nearby_units {
            if !boss_team.is_enemy(unit_team) {
                continue;
            }
            let dx = unit_transform.translation.x - boss_pos.x;
            let dz = unit_transform.translation.z - boss_pos.z;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist <= melee_range + unit_hitbox.radius {
                enemy_nearby = true;
                break;
            }
        }

        if !enemy_nearby {
            continue;
        }

        teleport_timer.reset(TELEPORT_COOLDOWN);

        let castle_xz = Vec2::new(CASTLE_POSITION.x, CASTLE_POSITION.z);
        let wizard_xz = Vec2::new(WIZARD_POSITION.x, WIZARD_POSITION.z);
        let wizard_ground_range = ground_projected_range(
            crate::game::units::wizard::constants::DEFAULT_SPELL_RANGE,
            WIZARD_POSITION.y,
        );
        let hover_y = DARK_MAGE_SPRITE_HEIGHT / 2.0 + DARK_MAGE_FLOAT_BASE_OFFSET;

        let mut chosen_dest: Option<Vec3> = None;
        for _ in 0..30 {
            let x = VISIBLE_MIN_X + game_rng.0.random::<f32>() * (VISIBLE_MAX_X - VISIBLE_MIN_X);
            let z = VISIBLE_MIN_Z + game_rng.0.random::<f32>() * (VISIBLE_MAX_Z - VISIBLE_MIN_Z);
            let candidate = Vec2::new(x, z);

            let dist_from_current = ((x - boss_pos.x).powi(2) + (z - boss_pos.z).powi(2)).sqrt();
            let dist_from_castle = candidate.distance(castle_xz);
            let dist_from_wizard = candidate.distance(wizard_xz);

            if dist_from_current < TELEPORT_MIN_DISTANCE
                || dist_from_castle < TELEPORT_MIN_CASTLE_DISTANCE
                || dist_from_wizard > wizard_ground_range
            {
                continue;
            }

            let world_pos = Vec3::new(x, hover_y, z);
            if !is_on_screen(camera, camera_global, world_pos, TELEPORT_NDC_MARGIN) {
                continue;
            }

            chosen_dest = Some(world_pos);
            break;
        }

        let Some(dest_pos) = chosen_dest else {
            continue;
        };

        transform.translation = dest_pos;

        vfx::systems::spawn_aura_bubble_contracting(
            &mut commands,
            &visual_assets,
            visual_assets.teleport_aura_sphere.clone(),
            boss_pos,
            TELEPORT_BUBBLE_RADIUS,
            1.0,
        );
        vfx::systems::spawn_aura_bubble(
            &mut commands,
            &visual_assets,
            visual_assets.teleport_aura_sphere.clone(),
            dest_pos,
            TELEPORT_BUBBLE_RADIUS,
            1.5,
        );
        spawn_teleport_vfx(&mut commands, boss_pos, dest_pos, TELEPORT_BUBBLE_RADIUS);

        play_sfx_scaled(
            &mut commands,
            &sfx.teleport_cast,
            boss_pos,
            &game_config,
            1.0,
        );
    }
}
