use bevy::prelude::*;

use crate::game::units::wizard::spells::black_hole::components::BlackHole;
use crate::networking::snapshot::SpellEffectSnapshot;

use crate::game::multiplayer::components::OnMultiplayerGameScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

pub(crate) fn spawn_black_hole(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
) -> Option<Entity> {
    use crate::game::units::wizard::spells::black_hole::components::BlackHoleTalentParams;
    let extra = effect.extra;
    let flags = effect.flags;
    let max_radius = extra[0];
    let empowerment = extra[1];
    let talent_params = BlackHoleTalentParams {
        event_horizon: flags & (1 << 0) != 0,
        crushing_pressure: flags & (1 << 1) != 0,
        void_siphon: flags & (1 << 2) != 0,
        singularity: flags & (1 << 3) != 0,
        dimensional_rift: flags & (1 << 4) != 0,
        ..BlackHoleTalentParams::default()
    };
    // Icosphere scaled by max_radius * growth_factor in update_black_hole_visuals.
    // The ghost still won't run the damage/pull/etc. systems (those
    // are gated `Without<GhostSpellEffect>` for host-authoritative)
    // — but visual systems and talent-driven extra visuals can now
    // see the correct flags.
    Some(
        commands
            .spawn((
                Mesh3d(assets.black_hole_sphere.clone()),
                MeshMaterial3d(assets.black_hole.clone()),
                Transform::from_translation(pos).with_scale(Vec3::ZERO),
                BlackHole::new(pos, max_radius, empowerment, talent_params),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_arcane_crystal(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
) -> Option<Entity> {
    use crate::game::units::wizard::spells::arcane_crystal::components::{
        ArcaneCrystal, AutoCrystalTimer, CrystalNetwork, CrystalRangeIndicator, PrismaticExplosion,
        ResonanceCascade,
    };
    let extra = effect.extra;
    let flags = effect.flags;
    let range = extra[0];
    let duration = extra[1];
    let empowerment = extra[2];
    let height = 35.0 * empowerment; // CRYSTAL_HEIGHT * empowerment
    let sphere_radius = height / 3.0;
    // Cross-plane sphere scaled to crystal shape
    let mut ec = commands.spawn((
        Mesh3d(assets.cross_plane_sphere.clone()),
        MeshMaterial3d(assets.arcane_crystal.clone()),
        Transform::from_translation(Vec3::new(pos.x, height / 2.0, pos.z)).with_scale(Vec3::new(
            0.7 * sphere_radius,
            1.5 * sphere_radius,
            0.7 * sphere_radius,
        )),
        ArcaneCrystal::new(
            Vec3::new(pos.x, height / 2.0, pos.z),
            range,
            duration,
            range * 0.15,
            empowerment,
        ),
        OnMultiplayerGameScreen,
    ));
    // Re-insert any talent marker components present on the original
    // caster's crystal so the receiving peer's talent visuals / state
    // systems attach correctly.
    if flags & (1 << 0) != 0 {
        ec.insert(ResonanceCascade { absorptions: 0 });
    }
    if flags & (1 << 1) != 0 {
        ec.insert(PrismaticExplosion);
    }
    if flags & (1 << 2) != 0 {
        ec.insert(AutoCrystalTimer { timer: 0.0 });
    }
    if flags & (1 << 3) != 0 {
        ec.insert(CrystalNetwork);
    }
    let crystal_entity = ec.id();
    // Mirror the pink aura range sphere that the local-cast path
    // spawns in `arcane_crystal/setup.rs`. Without this, the
    // receiving peer sees the crystal but not its sphere of
    // influence — the bubble visual is missing on the ghost side.
    ec.commands().spawn((
        Mesh3d(assets.explosion_sphere.clone()),
        MeshMaterial3d(assets.crystal_aura_sphere.clone()),
        Transform::from_translation(Vec3::new(pos.x, 0.0, pos.z)).with_scale(Vec3::splat(range)),
        CrystalRangeIndicator { crystal_entity },
        OnMultiplayerGameScreen,
    ));
    Some(crystal_entity)
}

pub(crate) fn spawn_lightning_rod(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
) -> Option<Entity> {
    use crate::game::units::wizard::spells::lightning_rod::components::{
        LightningRod, LightningRodTalentParams,
    };
    let extra = effect.extra;
    let flags = effect.flags;
    let duration = extra[0];
    let empowerment = extra[1];
    // Unpack the talent booleans sent by the host. The numeric
    // duration/strike-interval/arc-radius/damage multipliers are
    // already baked into the host's authoritative strike damage
    // (which flows back via CRDT), so they stay at default here.
    let talent_params = LightningRodTalentParams {
        chain_reaction: flags & (1 << 0) != 0,
        magnetic_field: flags & (1 << 1) != 0,
        overcharge: flags & (1 << 2) != 0,
        storm_spire: flags & (1 << 3) != 0,
        tesla_coil: flags & (1 << 4) != 0,
        lightning_nexus: flags & (1 << 5) != 0,
        ..LightningRodTalentParams::default()
    };
    // Lightning rod uses a cylinder mesh; create one at spawn.
    // This is a small allocation but rods are rare (1-2 at most).
    let tower_height = 60.0; // TOWER_HEIGHT
    let tower_radius = 8.0; // TOWER_RADIUS
    Some(
        commands
            .spawn((
                Mesh3d(assets.unit_cuboid.clone()),
                MeshMaterial3d(assets.lightning_rod.clone()),
                Transform::from_translation(Vec3::new(pos.x, tower_height / 2.0, pos.z))
                    .with_scale(Vec3::new(
                        tower_radius * 2.0,
                        tower_height,
                        tower_radius * 2.0,
                    )),
                LightningRod::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    duration,
                    empowerment,
                    talent_params,
                ),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}
