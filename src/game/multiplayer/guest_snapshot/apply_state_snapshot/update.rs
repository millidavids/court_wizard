use bevy::prelude::*;

use crate::game::units::archer::ArcherAssets;
use crate::game::units::components::{
    Corpse, Health, OriginalMaterial, RemoteElectricEffect, RemoteFireEffect, RemoteFrostEffect,
    RemotePoisonEffect, RemotePolymorphEffect,
};
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::king::components::SpellShield;
use crate::game::units::king::resources::KingAssets;
use crate::game::units::undead::resources::UndeadAssets;
use crate::game::units::wizard::spells::mark_of_death::components::ActiveMarkOfDeath;
use crate::game::units::wizard::spells::polymorph::constants::SHEEP_COLOR;
use crate::game::units::wizard::spells::polymorph::systems::apply_sheep_visual;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::crdt::CrdtHealth;

use super::super::super::guest_visuals::pick_material;

/// Updates an existing ghost entity whose unit appeared in the snapshot and
/// already has a local counterpart. Applies velocity, position, CRDT health,
/// status effect markers, corpse transitions, polymorph, and combat animation.
///
/// `smelly_ghosts` and `melee_ghosts` are separate overflow queries kept outside
/// the main ghost query to stay under Bevy's query-data arity limit.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_ghost_entity(
    commands: &mut Commands,
    entity: Entity,
    transform: &mut Transform,
    crdt_health: &mut CrdtHealth,
    health: &mut Health,
    velocity: &mut crate::game::components::Velocity,
    has_remote_fire: bool,
    has_remote_frost: bool,
    has_remote_electric: bool,
    has_spell_shield: bool,
    has_corpse: bool,
    has_combat: bool,
    has_remote_poison: bool,
    has_remote_mark: bool,
    has_remote_polymorph: bool,
    smelly_ghosts: &Query<Entity, With<crate::game::units::status_effects::SmellyModifier>>,
    melee_ghosts: &Query<Entity, With<crate::game::units::components::InMelee>>,
    remote_crdt: CrdtHealth,
    pos: Vec3,
    vx: f32,
    vz: f32,
    is_corpse: bool,
    is_king: bool,
    is_archer: bool,
    is_guard: bool,
    is_swordcerer_avatar: bool,
    remote_fire: bool,
    remote_frost: bool,
    remote_electric: bool,
    remote_spell_shield: bool,
    remote_combat: bool,
    remote_poison: bool,
    remote_mark: bool,
    remote_polymorph: bool,
    remote_smelly: bool,
    remote_in_melee: bool,
    team: crate::game::units::components::Team,
    infantry_assets: &InfantryAssets,
    archer_assets: &ArcherAssets,
    king_assets: &KingAssets,
    undead_assets: &UndeadAssets,
    spell_assets: &SpellVisualAssets,
    swordcerer_assets: &crate::game::units::wizard::archetypes::swordcerer::resources::SwordcererAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    // Use the host's AUTHORITATIVE velocity from the snapshot.
    // Synthesising velocity locally from position deltas caused
    // the walking animation to reset to idle every time the
    // host's unit briefly stopped (between targeting decisions,
    // wall-avoidance moments, etc.) — the delta was zero, the
    // synthesised speed was below the animation move threshold,
    // and `update_walking_animation` reset to frame 0. With the
    // host's actual `Velocity` shipped in the snapshot, guest
    // animations match the host's exactly.
    //
    // Defence-in-depth: reject NaN/Inf (which would propagate
    // into `speed_sq` and make every IEEE comparison return
    // false → animation ticks forever and any future system
    // that integrates this velocity into a Transform corrupts
    // position), and cap magnitude at a sane upper bound so a
    // pathological host spike (knockback math error, etc.)
    // doesn't flash through the animation for one frame.
    const MAX_REMOTE_SPEED: f32 = 400.0;
    let raw_x = if vx.is_finite() { vx } else { 0.0 };
    let raw_z = if vz.is_finite() { vz } else { 0.0 };
    let raw_speed_sq = raw_x * raw_x + raw_z * raw_z;
    let (cx, cz) = if raw_speed_sq > MAX_REMOTE_SPEED * MAX_REMOTE_SPEED {
        let scale = MAX_REMOTE_SPEED / raw_speed_sq.sqrt();
        (raw_x * scale, raw_z * scale)
    } else {
        (raw_x, raw_z)
    };
    velocity.x = cx;
    velocity.z = cz;

    transform.translation = pos;

    // Merge CRDT state from host (takes max of each slot)
    crdt_health.merge(&remote_crdt);

    // Re-derive Health from converged CRDT state so damage systems see correct HP
    health.current = crdt_health.current_hp();

    // Material handling on corpse transition.
    //
    // For NON-KING units we DELIBERATELY do NOT swap the
    // material here. The existing block below inserts `Corpse` + `DyingAnimation`,
    // and the shared SP animation pipeline (`update_dying_animation` →
    // `DeathAnimationFinished` → `finalize_dying_to_corpse`)
    // mutates the entity's CURRENT material in place — first
    // setting its `base_color_texture` to the death sheet and
    // ticking the death-frame UV, then converting it to the
    // corpse appearance (corpse tint + `AlphaMode::Blend`).
    // Because the ghost's material is the per-entity allocation
    // from the spawn branch, those `get_mut` writes don't
    // corrupt anything else.
    //
    // The previous version of this branch swapped to a SHARED
    // handle cloned from `infantry_assets.defender_corpse_materials[idx]`
    // (and similar). `update_dying_animation`'s first-frame
    // `materials.get_mut(handle); mat.base_color_texture = death_sheet`
    // then mutated the SHARED asset, replacing the texture for
    // every other ghost or SP corpse holding that variant for
    // the duration of one death sequence. Removing the swap
    // here fixes that corruption.
    //
    // KINGS are special: kings have no death animation in SP
    // (the existing block skips DyingAnimation for `is_king`),
    // so `finalize_dying_to_corpse` never runs on them. They
    // also use a dedicated corpse sprite, not a tint of the
    // alive sprite. We swap directly to a king-corpse handle
    // here. At most one king per team exists, and nothing ever
    // calls `get_mut` on the king-corpse handle, so sharing
    // the handle is safe.
    if is_corpse != has_corpse {
        let mut ec = commands.entity(entity);
        if is_king {
            let new_handle = pick_material(
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
            );
            ec.insert(MeshMaterial3d(new_handle));
        } else if !is_corpse {
            // Corpse → alive: Raise the Dead resurrected this corpse
            // into an undead unit. Swap the corpse material to the alive
            // sprite so it stops rendering as a laid-flat corpse. Uses
            // the CURRENT snapshot team (now `Undead`), so `pick_material`
            // returns the dedicated undead sprite. The sprite mesh is
            // unchanged (sprite units share one quad for both states).
            let new_handle = pick_material(
                infantry_assets,
                archer_assets,
                king_assets,
                undead_assets,
                materials,
                team,
                false,
                is_king,
                is_archer,
                is_guard,
            );
            ec.insert(MeshMaterial3d(new_handle));
            // Raising a unit mid-death-animation would otherwise leave
            // DyingAnimation ticking death-sheet UVs over the new alive
            // sprite, flickering it. Clear it on the corpse→alive edge.
            ec.remove::<crate::game::units::components::DyingAnimation>();
        }
        // Clear stale tint state in both cases — if a tint was
        // active on the alive sprite, the SP corpse finalizer
        // (or, for kings, the snapshot swap above) supersedes
        // it, and we don't want a later tint-restore to put
        // the alive-looking material back on a corpse.
        ec.remove::<OriginalMaterial>();
    }

    // Sync remote status effect visual markers from host
    if remote_fire && !has_remote_fire {
        commands.entity(entity).insert(RemoteFireEffect);
    } else if !remote_fire && has_remote_fire {
        commands.entity(entity).remove::<RemoteFireEffect>();
    }
    if remote_frost && !has_remote_frost {
        commands.entity(entity).insert(RemoteFrostEffect);
    } else if !remote_frost && has_remote_frost {
        commands.entity(entity).remove::<RemoteFrostEffect>();
    }
    if remote_electric && !has_remote_electric {
        commands.entity(entity).insert(RemoteElectricEffect);
    } else if !remote_electric && has_remote_electric {
        commands.entity(entity).remove::<RemoteElectricEffect>();
    }
    if remote_poison && !has_remote_poison {
        commands.entity(entity).insert(RemotePoisonEffect);
    } else if !remote_poison && has_remote_poison {
        commands.entity(entity).remove::<RemotePoisonEffect>();
    }
    // Excremage smelly tint. The real `SmellyModifier` drives the
    // existing brown tint in `update_persistent_effect_visuals`
    // (visual, both peers). Its repulsion lives in host-only velocity
    // systems, so it has no gameplay effect on the guest ghost. Use a
    // separate `.contains` query to keep the main ghost query under
    // Bevy's arity limit.
    let has_remote_smelly = smelly_ghosts.contains(entity);
    if remote_smelly && !has_remote_smelly {
        // Use a very long duration: the ghost's smelly state is driven
        // entirely by the SMELLY snapshot flag (removed below when it
        // clears), so the local `update_timed_modifier` timer must NOT
        // expire it on its own — otherwise the brown tint flickers off
        // for one frame whenever the local timer runs out before the
        // next snapshot reasserts the flag.
        commands
            .entity(entity)
            .insert(crate::game::units::status_effects::SmellyModifier::new(
                1.0e9,
            ));
    } else if !remote_smelly && has_remote_smelly {
        commands
            .entity(entity)
            .remove::<crate::game::units::status_effects::SmellyModifier>();
    }
    // Mirror the host's melee state so the guest's battle-ambience loop
    // scales with on-field combat. `InMelee` holds the OPPOSING team
    // (semantically "in melee with"); on the guest the value is inert
    // (archer-targeting readers are host-only) but kept correct in case
    // a future ghost-side system reads it.
    let has_melee = melee_ghosts.contains(entity);
    if remote_in_melee && !has_melee {
        use crate::game::units::components::Team;
        let melee_with = match team {
            Team::Defenders => Team::Attackers,
            _ => Team::Defenders,
        };
        commands
            .entity(entity)
            .insert(crate::game::units::components::InMelee(melee_with));
    } else if !remote_in_melee && has_melee {
        commands
            .entity(entity)
            .remove::<crate::game::units::components::InMelee>();
    }
    // Mark of Death: insert the BARE `ActiveMarkOfDeath` marker so
    // `spawn_mark_indicators` renders the floating indicator. We do
    // NOT add `MarkedForDeathModifier`/`MarkTalentFlags`, so the
    // doom/executioner/blight gameplay systems never MATCH the ghost:
    // their `any_exist::<ActiveMarkOfDeath>` run-condition still wakes
    // them, but their queries require those absent components, so they
    // iterate nothing. Only the visual indicator renders.
    if remote_mark && !has_remote_mark {
        commands.entity(entity).insert(ActiveMarkOfDeath);
    } else if !remote_mark && has_remote_mark {
        commands.entity(entity).remove::<ActiveMarkOfDeath>();
    }

    // Polymorph: render the ghost as a sheep while the host's unit is
    // polymorphed (host-authoritative). Swap on the off→on edge,
    // restore the unit's normal sprite on the on→off edge. We also
    // strip any local `PolymorphedModifier` the guest's own cast left
    // on the ghost so the unit isn't excluded from `Without<
    // PolymorphedModifier>` spell-target queries forever after revert.
    if remote_polymorph && !has_remote_polymorph {
        let mut ec = commands.entity(entity);
        apply_sheep_visual(&mut ec, materials, spell_assets, SHEEP_COLOR);
        ec.insert(RemotePolymorphEffect);
    } else if !remote_polymorph && has_remote_polymorph {
        // Only restore the alive sprite if the unit is still alive. If
        // it died AS a sheep (is_corpse), the corpse-transition branch
        // above owns the material — swapping to a SHARED corpse handle
        // here would let DyingAnimation's `get_mut` corrupt the corpse
        // appearance of every other entity sharing that handle variant.
        let restored = if is_corpse {
            None
        } else {
            let material = pick_material(
                infantry_assets,
                archer_assets,
                king_assets,
                undead_assets,
                materials,
                team,
                false,
                is_king,
                is_archer,
                is_guard,
            );
            let mesh = if is_king {
                king_assets.sprite_mesh.clone()
            } else if is_archer {
                archer_assets.sprite_mesh.clone()
            } else {
                infantry_assets.sprite_mesh.clone()
            };
            Some((material, mesh))
        };
        let mut ec = commands.entity(entity);
        if let Some((material, mesh)) = restored {
            ec.insert(MeshMaterial3d(material));
            ec.insert(Mesh3d(mesh));
        }
        ec.remove::<RemotePolymorphEffect>();
        ec.remove::<crate::game::units::status_effects::PolymorphedModifier>();
        // Also clear the sheep bounce so a reverted ghost doesn't keep
        // floating (bounce_sheep_units is host-gated, but a guest-cast
        // ghost can carry SheepBounce from its own local application).
        ec.remove::<crate::game::units::wizard::spells::polymorph::sheep_visual::SheepBounce>();
    }

    // Sync spell shield component from host. No visual swap
    // — the king's aura sphere is the constant visual and
    // stays on regardless of shield state (matches SP, where
    // the king has no shield mechanic and the aura is just
    // always there).
    if remote_spell_shield && !has_spell_shield {
        commands.entity(entity).insert(SpellShield);
    } else if !remote_spell_shield && has_spell_shield {
        commands.entity(entity).remove::<SpellShield>();
    }

    // Sync corpse state so spell targeting filters work correctly.
    // On the non-corpse → corpse transition, also kick off the
    // shared `DyingAnimation` so the ghost plays the death frames
    // before settling into its corpse pose — matching the SP
    // visual where units don't pop straight from standing to
    // laid-flat. King has no death sprite sheet (instant corpse
    // swap in SP), so it's skipped here.
    if is_corpse && !has_corpse {
        let mut ec = commands.entity(entity);
        ec.insert(Corpse);
        if !is_king {
            let death_texture = if is_archer {
                archer_assets.death_texture.clone()
            } else {
                infantry_assets.death_texture.clone()
            };
            ec.insert(crate::game::units::components::DyingAnimation::new(
                death_texture,
            ));
        }
    } else if !is_corpse && has_corpse {
        commands.entity(entity).remove::<Corpse>();
    }

    // Mirror CombatAnimation transitions from the host so the
    // ghost plays its swing/shoot animation while the host's
    // unit is mid-attack. Without this the ghost stays on the
    // idle walking frame during melee. We ONLY insert on the
    // off→on edge; we **do not** force-remove on the on→off
    // edge — the shared SP system `update_combat_animation`
    // ticks the local frames and self-removes the component
    // when finished, AND in the same path restores the
    // walking texture / resets the UV transform. A forced
    // `commands.remove` here would bypass that cleanup and
    // leave the ghost's material stuck on the combat sheet
    // with a frozen mid-swing UV until the next attack
    // overwrites it.
    if remote_combat && !has_combat && !is_corpse && !is_king {
        let (combat_tex, walking_tex) = if is_swordcerer_avatar {
            // The Swordcerer avatar has its own attack sheet — without this
            // the guest's ghost permanently swapped to the infantry sheet
            // the first time the host's avatar attacked.
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
            // Infantry + king's guard share the infantry sheet.
            // King has no combat sheet in SP (skipped above).
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
}
