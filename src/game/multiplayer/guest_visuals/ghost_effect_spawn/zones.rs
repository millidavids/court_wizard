use bevy::prelude::*;

use crate::game::units::wizard::spells::entangle::components::EntangleGroundEffect;
use crate::game::units::wizard::spells::fog_cloud::components::FogCloudZone;
use crate::game::units::wizard::spells::grease::components::GreaseZone;
use crate::game::units::wizard::spells::healing_plume::components::HealingPlumeZone;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::plague_wind::components::PlagueWindCloud;
use crate::game::units::wizard::spells::spike_growth::components::{
    SpikeGrowthTalentParams, SpikeGrowthZone,
};
use crate::networking::snapshot::SpellEffectSnapshot;

use crate::game::multiplayer::components::OnMultiplayerGameScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

pub(crate) fn spawn_spike_growth_zone(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
) -> Option<Entity> {
    let extra = effect.extra;
    let radius = extra[0];
    let duration = extra[1];
    Some(
        commands
            .spawn((
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z)),
                SpikeGrowthZone::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    radius,
                    0.0,
                    1.0,
                    0.0,
                    0.0,
                    duration,
                    SpikeGrowthTalentParams::default(),
                ),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_healing_plume_zone(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
    flat_rotation: Quat,
) -> Option<Entity> {
    let extra = effect.extra;
    let radius = extra[0];
    let duration = extra[1];
    let material = materials.add(materials.get(&assets.healing_plume_zone)?.clone());
    Some(
        commands
            .spawn((
                Mesh3d(assets.unit_circle.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(radius)),
                HealingPlumeZone::new(Vec3::new(pos.x, 0.0, pos.z), radius, 0.0, 1.0, duration),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_entangle_ground(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
) -> Option<Entity> {
    let extra = effect.extra;
    let duration = extra[1];
    // Spawn the visible vine rings (same as single-player) so the entangle
    // zone isn't invisible on the opposing client. RNG here is cosmetic.
    crate::game::units::wizard::spells::entangle::vines::spawn_vine_toruses(
        &mut rand::rng(),
        commands,
        assets,
        materials,
        Vec3::new(pos.x, 0.0, pos.z),
        120.0,
        duration,
        OnMultiplayerGameScreen,
    );
    Some(
        commands
            .spawn((
                Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z)),
                Visibility::Hidden,
                EntangleGroundEffect::new(
                    duration,
                    Vec3::new(pos.x, 1.0, pos.z),
                    120.0,
                    crate::game::units::wizard::spells::entangle::components::EntangleTalentParams::default(),
                ),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_fog_cloud_zone(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
    flat_rotation: Quat,
) -> Option<Entity> {
    use crate::game::units::wizard::spells::fog_cloud::components::{
        BlindingMistZone, ChokingFogZone, ConcealingVeilZone, DisorientingVaporsZone,
        PhantomFogZone, RollingFogZone,
    };
    let extra = effect.extra;
    let flags = effect.flags;
    let radius = extra[0];
    let duration = extra[1];
    // Bug fix: ghost zone previously got `evasion_chance=0.0` and
    // `evasion_refresh_duration=0.0`, making the fog do nothing on
    // the remote peer regardless of caster. The collector now packs
    // these in `extra[2]` and `extra[3]` so the ghost matches the
    // caster's values.
    let evasion_chance = extra[2];
    let evasion_refresh_duration = extra[3];
    let material = materials.add(materials.get(&assets.fog_cloud_zone)?.clone());
    let mut ec = commands.spawn((
        Mesh3d(assets.unit_circle.clone()),
        MeshMaterial3d(material),
        Transform::from_translation(Vec3::new(pos.x, 1.0, pos.z))
            .with_rotation(flat_rotation)
            .with_scale(Vec3::splat(radius)),
        FogCloudZone::new(
            Vec3::new(pos.x, 0.0, pos.z),
            radius,
            evasion_chance,
            evasion_refresh_duration,
            1.0,
            duration,
        ),
        OnMultiplayerGameScreen,
    ));
    // Insert FogCloud talent marker components based on the host's
    // packed flags. Gameplay-authoritative behavior (Choking DPS,
    // Rolling drift, Phantom spawns) is host-only, so these markers
    // on the ghost are mostly for visual consistency / system
    // existence-checks. Default field values are fine — the host
    // ticks the real ones.
    if flags & (1 << 0) != 0 {
        ec.insert(BlindingMistZone);
    }
    if flags & (1 << 1) != 0 {
        ec.insert(ConcealingVeilZone);
    }
    if flags & (1 << 2) != 0 {
        ec.insert(DisorientingVaporsZone);
    }
    if flags & (1 << 3) != 0 {
        ec.insert(PhantomFogZone { spawn_timer: 0.0 });
    }
    if flags & (1 << 4) != 0 {
        ec.insert(ChokingFogZone::new(0.0, 1.0));
    }
    if flags & (1 << 5) != 0 {
        ec.insert(RollingFogZone { speed: 0.0 });
    }
    Some(ec.id())
}

pub(crate) fn spawn_grease_zone(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
    flat_rotation: Quat,
) -> Option<Entity> {
    let extra = effect.extra;
    let radius = extra[0];
    let duration = extra[1];
    let mut base_mat = materials.get(&assets.grease_zone)?.clone();
    base_mat.alpha_mode = bevy::material::AlphaMode::Mask(0.01);
    let material = materials.add(base_mat);
    Some(
        commands
            .spawn((
                Mesh3d(assets.unit_circle.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 2.0, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(radius)),
                GreaseZone::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    radius,
                    0.0,
                    0.0,
                    1.0,
                    duration,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    Default::default(),
                ),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_grease_fire(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
    assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
    flat_rotation: Quat,
) -> Option<Entity> {
    let extra = effect.extra;
    // Fire overlay is a second circle on top of the grease zone.
    // Scale is updated every frame from the snapshot (fire spread animation).
    let scale = extra[0].max(0.01);
    let material = materials.add(materials.get(&assets.grease_fire)?.clone());
    Some(
        commands
            .spawn((
                Mesh3d(assets.unit_circle.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, 1.1, pos.z))
                    .with_rotation(flat_rotation)
                    .with_scale(Vec3::splat(scale)),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_plague_wind_cloud(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
) -> Option<Entity> {
    use crate::game::units::wizard::spells::plague_wind::components::PlagueWindTalentParams;
    let extra = effect.extra;
    let flags = effect.flags;
    let radius = extra[0];
    let duration = extra[1];
    let speed = extra[2];
    let direction_angle = extra[3];
    let direction = Vec3::new(direction_angle.sin(), 0.0, direction_angle.cos());
    // Unpack the talent boolean flags sent by the host (see the
    // matching collector arm in `spell_sync.rs::collect_spell_effect_snapshots`).
    // Numeric talent multipliers are kept at default — they are
    // already baked into the host's authoritative damage values
    // that flow back via the CRDT pipeline, so reproducing them on
    // the ghost would double-count.
    let talent_params = PlagueWindTalentParams {
        plague_carrier: flags & (1 << 0) != 0,
        toxic_weakness: flags & (1 << 1) != 0,
        choking_gas: flags & (1 << 2) != 0,
        pandemic: flags & (1 << 3) != 0,
        twin_plumes: flags & (1 << 4) != 0,
        necrotic_rot: flags & (1 << 5) != 0,
        ..PlagueWindTalentParams::default()
    };
    // No mesh — the cloud's visual is the shared green `plague_smoke`
    // particle system (`emit_plague_cloud_particles` runs on every
    // PlagueWindCloud, including this ghost), matching single-player. The
    // old flat green disc was a placeholder.
    Some(
        commands
            .spawn((
                Transform::from_translation(Vec3::new(pos.x, 0.0, pos.z)),
                PlagueWindCloud::new(
                    Vec3::new(pos.x, 0.0, pos.z),
                    radius,
                    0.0,
                    1.0,
                    duration,
                    speed,
                    direction,
                    talent_params,
                ),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}

pub(crate) fn spawn_meteor_ground_fire(
    commands: &mut Commands,
    effect: &SpellEffectSnapshot,
    pos: Vec3,
) -> Option<Entity> {
    let extra = effect.extra;
    let radius = extra[0];
    let duration = extra[1];
    // No mesh — single-player spawns the burning patch mesh-less and
    // renders it through the shared fire-particle system. The flat
    // orange disc was a ghost-only placeholder that sat on top of the
    // particles (the user's "old orange circle").
    Some(
        commands
            .spawn((
                Transform::from_translation(Vec3::new(pos.x, 0.5, pos.z))
                    .with_scale(Vec3::splat(radius)),
                Visibility::default(),
                MeteorGroundFire::new(Vec3::new(pos.x, 0.0, pos.z), radius, 0.0, 1.0, duration),
                OnMultiplayerGameScreen,
            ))
            .id(),
    )
}
