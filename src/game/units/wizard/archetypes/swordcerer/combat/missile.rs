use super::super::components::*;
use super::super::constants::*;
use super::super::resources::{SwordcererAssets, SwordcererPhase, SwordcererState};
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::{Corpse, Team};
use crate::game::units::wizard::components::{LocalWizard, Mana, Wizard};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Handles LT / right-click magic missile firing from the swordcerer avatar.
///
/// Fires in the avatar's current facing direction (most recent movement vector).
/// There is no separate aim input — movement direction IS attack direction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn fire_missile(
    mut mouse_pressed: MessageReader<crate::game::input::messages::MouseRightPressed>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut avatar_query: Query<
        (
            Entity,
            &Transform,
            Option<&SwordcererMissileCooldown>,
            Option<&SwordcererFacing>,
        ),
        (With<SwordcererAvatar>, Without<GuestControlledAvatar>),
    >,
    targets: Query<(Entity, &Transform, &Team), (Without<SwordcererAvatar>, Without<Corpse>)>,
    sfx: Res<SpellSfxAssets>,
    config: Res<GameConfig>,
    state: Res<SwordcererState>,
    mut wizard_query: Query<&mut Mana, (With<Wizard>, With<LocalWizard>)>,
    swordcerer_assets: Res<SwordcererAssets>,
) {
    if state.phase != SwordcererPhase::OnField {
        return;
    }
    if mouse_pressed.read().next().is_none() {
        return;
    }

    let Ok((avatar_entity, avatar_transform, cooldown, facing)) = avatar_query.single_mut() else {
        return;
    };

    if cooldown.is_some_and(|cd| cd.remaining > 0.0) {
        return;
    }

    let Ok(mut mana) = wizard_query.single_mut() else {
        return;
    };
    if !mana.consume(MISSILE_MANA_COST) {
        return;
    }

    let facing_vec = facing.copied().unwrap_or_default().0;
    let direction = Vec3::new(facing_vec.x, 0.0, facing_vec.y).normalize_or_zero();

    // The host's own avatar is always on the Defenders team.
    spawn_avatar_missile(
        &mut commands,
        &visual_assets,
        &sfx,
        &config,
        &swordcerer_assets,
        &targets,
        avatar_entity,
        avatar_transform.translation,
        direction,
        Team::Defenders,
    );
}

/// Spawns a homing magic missile from a Swordcerer avatar toward the nearest
/// enemy unit, plays the cast SFX, and starts the avatar's casting animation +
/// missile cooldown. Shared by the host's own avatar (`fire_missile`) and the
/// guest-controlled avatar (`networking::apply_guest_avatar_input`). The enemy
/// teams are derived from the avatar's own team, so a host (Defenders) avatar
/// targets Attackers/Undead and a guest (Attackers) avatar targets
/// Defenders/Undead. Any mana cost is handled by the caller.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_avatar_missile(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    sfx: &SpellSfxAssets,
    config: &GameConfig,
    swordcerer_assets: &SwordcererAssets,
    targets: &Query<(Entity, &Transform, &Team), (Without<SwordcererAvatar>, Without<Corpse>)>,
    avatar_entity: Entity,
    avatar_pos: Vec3,
    direction: Vec3,
    avatar_team: Team,
) {
    use crate::game::units::wizard::spells::magic_missile::components::{
        MagicMissile, TargetTeams,
    };

    // Enemies are the opposing army plus the always-hostile Undead.
    let (target_teams, enemy_army) = match avatar_team {
        Team::Defenders => (TargetTeams::AttackersAndUndead, Team::Attackers),
        _ => (TargetTeams::DefendersAndUndead, Team::Defenders),
    };

    let spawn_pos = avatar_pos + Vec3::new(0.0, 30.0, 0.0);
    let target = targets
        .iter()
        .filter(|(_, _, team)| **team == enemy_army || **team == Team::Undead)
        .min_by(|a, b| {
            spawn_pos
                .distance(a.1.translation)
                .partial_cmp(&spawn_pos.distance(b.1.translation))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(e, _, _)| e);

    let mut missile = MagicMissile::new(
        direction * MISSILE_SPEED,
        0.0,
        target,
        1.0,
        target_teams,
        3000.0,
        spawn_pos,
    );
    missile.damage = MISSILE_DAMAGE;
    missile.base_homing_strength = MISSILE_HOMING_STRENGTH;

    commands.spawn((
        Mesh3d(visual_assets.magic_missile_mesh.clone()),
        MeshMaterial3d(visual_assets.magic_missile.clone()),
        Transform::from_translation(spawn_pos),
        missile,
        OnGameplayScreen,
    ));

    audio::play_sfx(commands, &sfx.magic_missile_cast, spawn_pos, config, sfx);

    commands.entity(avatar_entity).insert(
        crate::game::units::components::CombatAnimation::new_casting(
            swordcerer_assets.casting_texture.clone(),
            swordcerer_assets.sprite_texture.clone(),
        ),
    );

    commands
        .entity(avatar_entity)
        .insert(SwordcererMissileCooldown {
            remaining: MISSILE_COOLDOWN,
        });
}
