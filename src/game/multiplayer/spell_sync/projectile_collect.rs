use bevy::prelude::*;

use crate::game::units::wizard::spells::chain_lightning::components::ChainLightningArc;
use crate::game::units::wizard::spells::finger_of_death::components::FingerOfDeathBeam;
use crate::game::units::wizard::spells::fireball::components::Fireball;
use crate::game::units::wizard::spells::lightning_bolt::LightningBolt;
use crate::game::units::wizard::spells::lightning_rod::components::{
    LightningRodArc, LightningStrike,
};
use crate::game::units::wizard::spells::meteor_fall::components::MeteorProjectile;
use crate::game::units::wizard::spells::squall::components::IceProjectile;
use crate::networking::snapshot::{SpellArcSnapshot, SpellProjectileSnapshot, SpellSnapshotData};

/// Collects ephemeral spell projectiles and arcs into the snapshot data.
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn collect_spell_projectile_snapshots(
    mut spell_data: ResMut<SpellSnapshotData>,
    fireballs: Query<&Transform, With<Fireball>>,
    ice_projectiles: Query<&Transform, With<IceProjectile>>,
    meteor_projectiles: Query<&Transform, With<MeteorProjectile>>,
    chain_arcs: Query<&ChainLightningArc>,
    lightning_strikes: Query<(&LightningStrike, &LightningBolt)>,
    lightning_rod_arcs: Query<&LightningRodArc>,
    fod_beams: Query<&FingerOfDeathBeam>,
    dispel_projectiles: Query<
        &Transform,
        With<crate::game::units::wizard::spells::dispel::components::DispelProjectile>,
    >,
) {
    spell_data.spell_projectiles.clear();
    spell_data.spell_arcs.clear();

    // Read the visual radius the local caster used straight from
    // `Transform.scale` — both SP and ghost spawns set this via
    // `.with_scale(Vec3::splat(visual_radius))`, so it's a faithful
    // capture of talent + empowerment scaling without re-deriving.
    for t in &fireballs {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 0,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
            scale: t.scale.x,
        });
    }

    for t in &ice_projectiles {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 1,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
            scale: t.scale.x,
        });
    }

    for t in &meteor_projectiles {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 2,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
            scale: t.scale.x,
        });
    }

    for t in &dispel_projectiles {
        spell_data.spell_projectiles.push(SpellProjectileSnapshot {
            kind: 3,
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
            scale: t.scale.x,
        });
    }

    for arc in &chain_arcs {
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 0,
            ox: arc.start.x,
            oy: arc.start.y,
            oz: arc.start.z,
            tx: arc.end.x,
            ty: arc.end.y,
            tz: arc.end.z,
        });
    }

    for (strike, bolt) in &lightning_strikes {
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 1,
            ox: bolt.end.x,
            oy: bolt.end.y,
            oz: bolt.end.z,
            tx: strike.target_pos.x,
            ty: strike.target_pos.y,
            tz: strike.target_pos.z,
        });
    }

    // Crystal beams are now DisintegrateBeam entities and crystal arcs are
    // ChainLightningArc entities. ChainLightningArc ships above as kind=0; every
    // DisintegrateBeam (real disintegrate + crystal beam) ships via the dedicated
    // `BeamSnapshot` path in `send_spell_visual_snapshot` (carrying width), so no
    // duplicate kind=6 arc is emitted here.

    for beam in &fod_beams {
        // Grow with the cast like single-player: send the CURRENT visual length
        // (logical length + overshoot, scaled by cast progress) so the guest's cone
        // grows during the channel instead of popping in at full length.
        use crate::game::units::wizard::spells::finger_of_death::constants as fod;
        let progress_scale = if beam.has_fired {
            1.0
        } else {
            beam.cast_progress
        };
        let visual_len = (beam.length + fod::BEAM_VISUAL_OVERSHOOT) * progress_scale;
        let end = beam.origin + beam.direction * visual_len;
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 4,
            ox: beam.origin.x,
            oy: beam.origin.y,
            oz: beam.origin.z,
            tx: end.x,
            ty: end.y,
            tz: end.z,
        });
    }

    for arc in &lightning_rod_arcs {
        spell_data.spell_arcs.push(SpellArcSnapshot {
            kind: 5,
            ox: arc.start.x,
            oy: arc.start.y,
            oz: arc.start.z,
            tx: arc.end.x,
            ty: arc.end.y,
            tz: arc.end.z,
        });
    }
}
