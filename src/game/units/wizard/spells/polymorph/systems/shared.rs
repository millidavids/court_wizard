use super::super::components::{
    ContagiousBaas, DireSheep, ExplosiveSheep, PermanentLivestock, PigForm,
};
use super::super::sheep_visual::SheepBounce;
use crate::game::units::components::PolymorphedModifier;
use crate::game::units::systems::create_sprite_material;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Removes all polymorph-related talent components from an entity.
pub fn strip_polymorph_components(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<PolymorphedModifier>()
        .remove::<ExplosiveSheep>()
        .remove::<ContagiousBaas>()
        .remove::<PigForm>()
        .remove::<PermanentLivestock>()
        .remove::<DireSheep>()
        .remove::<SheepBounce>();
    // NOTE: do NOT remove Billboard — every unit spawns billboarded, so the
    // sheep-state insert is a no-op and stripping it here left reverted units
    // facing a fixed direction (a flat sliver) for the rest of the match.
}

/// Swaps an entity's mesh + material to the sheep sprite. Shared by the SP cast
/// path, the multiplayer host status-receiver, and the multiplayer guest ghost
/// renderer so all three render an identical sheep. Callers add the gameplay bits
/// (`PolymorphedModifier`, `Health`, removing `AttackTiming`) and `SheepBounce`/
/// `Billboard` as appropriate — this helper is the pure visual swap.
pub fn apply_sheep_visual(
    entity_cmds: &mut EntityCommands,
    materials: &mut Assets<StandardMaterial>,
    visual_assets: &SpellVisualAssets,
    color: Color,
) {
    let sheep_material = create_sprite_material(
        materials,
        visual_assets.sheep_icon.clone(),
        color,
        Vec2::ONE,
        Vec2::ZERO,
    );
    entity_cmds.insert((
        MeshMaterial3d(sheep_material),
        Mesh3d(visual_assets.sheep_mesh.clone()),
    ));
}
