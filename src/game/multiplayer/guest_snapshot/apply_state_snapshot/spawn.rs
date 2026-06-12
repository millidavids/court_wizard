use bevy::prelude::*;

use crate::game::components::{Billboard, Velocity};
use crate::game::units::archer::ArcherAssets;
use crate::game::units::components::{
    Corpse, FacingDirection, Hitbox, RemoteElectricEffect, RemoteFireEffect, RemoteFrostEffect,
    RemotePoisonEffect, RemotePolymorphEffect, WalkingAnimation,
};
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::king::components::{King, SpellShield};
use crate::game::units::king::resources::KingAssets;
use crate::game::units::undead::resources::UndeadAssets;
use crate::game::units::wizard::spells::mark_of_death::components::ActiveMarkOfDeath;
use crate::game::units::wizard::spells::polymorph::constants::SHEEP_COLOR;
use crate::game::units::wizard::spells::polymorph::systems::apply_sheep_visual;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::crdt::CrdtHealth;

use super::super::super::components::{
    GhostEntity, GhostSwordcererAvatar, OnMultiplayerGameScreen,
};
use super::super::super::guest_visuals::pick_material;

/// Spawns a new ghost entity for a unit that appeared in the snapshot but has
/// no local counterpart yet. Returns the spawned `Entity` so the caller can
/// register it in `NetworkEntityMap`.
///
/// The caller is responsible for calling `entity_map.insert(unit_id, entity)`.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_ghost_entity(
    commands: &mut Commands,
    unit_id: u32,
    remote_crdt: CrdtHealth,
    pos: Vec3,
    is_corpse: bool,
    is_king: bool,
    is_archer: bool,
    is_guard: bool,
    is_swordcerer_avatar: bool,
    remote_fire: bool,
    remote_frost: bool,
    remote_electric: bool,
    remote_poison: bool,
    remote_mark: bool,
    remote_polymorph: bool,
    remote_spell_shield: bool,
    remote_combat: bool,
    team: crate::game::units::components::Team,
    infantry_assets: &InfantryAssets,
    archer_assets: &ArcherAssets,
    king_assets: &KingAssets,
    undead_assets: &UndeadAssets,
    spell_assets: &SpellVisualAssets,
    swordcerer_assets: &crate::game::units::wizard::archetypes::swordcerer::resources::SwordcererAssets,
    materials: &mut Assets<StandardMaterial>,
    game_rng: &mut crate::game::seeded_rng::resources::GameRng,
) -> Entity {
    use crate::game::units::components::Health;
    use crate::networking::entity_map::NetworkEntityId;

    // Spawn new ghost entity with Team, Health, and CrdtHealth for spell targeting.
    // Animation components (Velocity + FacingDirection + WalkingAnimation) let the
    // shared animation systems run for ghosts the same way they run for
    // host-simulated units — Velocity starts at default and is overwritten
    // on subsequent snapshots from the host's authoritative `UnitSnapshot.vx/vz`.
    // Give the ghost the SAME `Hitbox` SP uses for this unit type
    // so SP spell-collision systems (fireball, ice, etc.) running
    // on this peer can land on it. Damage applied to the ghost's
    // `Health` flows through `sync_health_to_crdt` → CRDT slot →
    // peer snapshot → host's authoritative units lose HP. This is
    // the entire reason guest-cast spells now reach host units
    // without any per-spell network-message plumbing.
    //
    // **Known gap (latent):** today MP only spawns infantry / archer
    // / king / king's-guard, all covered by this three-way branch.
    // If MP ever spawns brutes / healers / dispellers / etc., add
    // their unit type to `UnitFlags` and a corresponding branch
    // here — otherwise their ghosts get the infantry hitbox (32u
    // vs e.g. brute's 80u) and spells will miss them.

    let hitbox = if is_king {
        Hitbox::new(
            crate::game::units::king::constants::KING_RADIUS,
            crate::game::units::king::constants::KING_HITBOX_HEIGHT,
        )
    } else if is_archer {
        Hitbox::new(
            crate::game::units::archer::constants::ARCHER_RADIUS,
            crate::game::constants::DEFENDER_HITBOX_HEIGHT,
        )
    } else if is_swordcerer_avatar {
        Hitbox::new(
            crate::game::units::wizard::archetypes::swordcerer::AVATAR_HITBOX_RADIUS,
            crate::game::units::wizard::archetypes::swordcerer::AVATAR_HITBOX_HEIGHT,
        )
    } else {
        // Infantry and king's guards both use the standard unit radius.
        Hitbox::new(
            crate::game::units::infantry::constants::UNIT_RADIUS,
            crate::game::constants::DEFENDER_HITBOX_HEIGHT,
        )
    };

    // Sprite-based units keep `sprite_mesh` for both live and corpse states.
    let mesh_handle = if is_king {
        king_assets.sprite_mesh.clone()
    } else if is_archer {
        archer_assets.sprite_mesh.clone()
    } else if is_swordcerer_avatar {
        swordcerer_assets.sprite_mesh.clone()
    } else {
        infantry_assets.sprite_mesh.clone()
    };

    // Compute the spawn material once. From this point on the
    // animation systems own the ghost's `MeshMaterial3d` — the
    // snapshot loop only touches it again on the alive→corpse
    // transition (see the `is_corpse != has_corpse` branch above).
    let material_handle = if is_swordcerer_avatar {
        crate::game::units::systems::create_default_sprite_material(
            materials,
            swordcerer_assets.sprite_texture.clone(),
            Color::WHITE,
        )
    } else {
        pick_material(
            infantry_assets,
            archer_assets,
            king_assets,
            undead_assets,
            materials,
            team,
            is_corpse,
            is_king,
            is_archer,
            is_guard,
        )
    };

    let initial_health = Health::new(remote_crdt.max_hp);
    let entity = commands
        .spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            Transform::from_translation(pos),
            Billboard,
            GhostEntity,
            team,
            NetworkEntityId(unit_id),
            initial_health,
            remote_crdt,
            hitbox,
            OnMultiplayerGameScreen,
            Velocity::default(),
            FacingDirection::default(),
            // Stagger the walk cycle so a freshly-snapshot army
            // doesn't flip animation frames in unison.
            WalkingAnimation::new_staggered(&mut game_rng.0),
        ))
        .id();

    // Tag the opponent's Swordcerer avatar so the health-bar UI finds it.
    if is_swordcerer_avatar {
        commands.entity(entity).insert(GhostSwordcererAvatar);
    }

    // Tag the ghost king with the `King` marker AND attach the
    // SP-style aura sphere. The HP bar reads
    // `Query<&Health, With<King>>` and otherwise wouldn't match any
    // ghost on the guest. SP simulation systems that touch
    // `With<King>` are host-gated, so adding this marker on the
    // guest only enables UI queries and is safe. The aura visual
    // is the same `explosion_sphere` + `king_aura_sphere` material
    // SP uses, scaled to `KING_AURA_RADIUS` — both peers now see
    // an identical king.
    if is_king {
        commands.entity(entity).insert(King);
        crate::game::units::king::systems::spawn_king_aura_visual(
            commands,
            entity,
            spell_assets,
            OnMultiplayerGameScreen,
        );
    }

    // Mirror spell-shield component state at spawn. No visual is
    // spawned for the shield itself — the aura sphere above is the
    // king's only visual indicator; shield state is invisible to
    // the player (same as SP, which has no shield mechanic at all).
    if remote_spell_shield {
        commands.entity(entity).insert(SpellShield);
    }

    if is_corpse {
        commands.entity(entity).insert(Corpse);
    }

    // Apply effect markers / combat animation at spawn so a late-
    // join ghost doesn't wait one extra snapshot tick for visuals
    // that should be on right now. The on/off edge transitions in
    // the update branch handle subsequent flag changes; without
    // these spawn-time inserts the first frame would show the
    // ghost un-tinted and not mid-swing even when the snapshot
    // says otherwise. We intentionally skip `DyingAnimation` on
    // spawn-as-corpse — the death already happened in the past
    // from this guest's perspective; replaying its frames now
    // would be wrong (the unit just appears already laid out).
    if remote_fire {
        commands.entity(entity).insert(RemoteFireEffect);
    }
    if remote_frost {
        commands.entity(entity).insert(RemoteFrostEffect);
    }
    if remote_electric {
        commands.entity(entity).insert(RemoteElectricEffect);
    }
    if remote_poison {
        commands.entity(entity).insert(RemotePoisonEffect);
    }
    if remote_mark {
        commands.entity(entity).insert(ActiveMarkOfDeath);
    }
    if remote_polymorph {
        let mut ec = commands.entity(entity);
        apply_sheep_visual(&mut ec, materials, spell_assets, SHEEP_COLOR);
        ec.insert(RemotePolymorphEffect);
    }
    if remote_combat && !is_corpse && !is_king {
        let (combat_tex, walking_tex) = if is_swordcerer_avatar {
            (
                swordcerer_assets.attacking_texture.clone(),
                swordcerer_assets.sprite_texture.clone(),
            )
        } else if is_archer {
            (
                archer_assets.shooting_texture.clone(),
                archer_assets.sprite_texture.clone(),
            )
        } else {
            (
                infantry_assets.attacking_texture.clone(),
                infantry_assets.sprite_texture.clone(),
            )
        };
        let anim = if is_archer {
            crate::game::units::components::CombatAnimation::new_shooting(combat_tex, walking_tex)
        } else {
            crate::game::units::components::CombatAnimation::new_attack(combat_tex, walking_tex)
        };
        commands.entity(entity).insert(anim);
    }

    entity
}
