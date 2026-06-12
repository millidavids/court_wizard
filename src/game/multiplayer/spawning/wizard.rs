//! Multiplayer wizard spawn helper.

use bevy::prelude::*;

use crate::game::components::Billboard;
use crate::game::units::components::{Health, Hitbox, MovementSpeed};
use crate::game::units::wizard::components::*;
use crate::game::units::wizard::constants;
use crate::game::units::wizard::spells::magic_missile_constants;
use crate::networking::resources::PeerRole;

use super::super::components::OnMultiplayerGameScreen;

/// Spawns a wizard at the given position for multiplayer.
///
/// `is_host_wizard` indicates whether this is the host's wizard (castle 1) or
/// the guest's wizard (castle 2). Combined with `role`, this determines which
/// marker components are added:
/// - Host wizard + Host role → `LocalWizard` (host controls this wizard)
/// - Guest wizard + Guest role → `LocalWizard` (guest controls this wizard)
/// - Guest wizard + Host role → `GuestWizard` (host simulates guest's spells)
// `pub(in crate::game)` so the single-player loading queue can spawn the co-op
// guest wizard proxy beside the host (co-op host runs the SP loading path).
#[allow(clippy::too_many_arguments)]
pub(in crate::game) fn spawn_mp_wizard(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    wizard_type: crate::config::WizardType,
    role: PeerRole,
    is_host_wizard: bool,
    coop: bool,
    wizard_assets: &WizardAssets,
) {
    let hitbox = Hitbox::new(constants::HITBOX_RADIUS, constants::HITBOX_HEIGHT);

    // Create a quad mesh matching the sprite aspect ratio
    let quad_mesh = Rectangle::new(
        constants::WIZARD_SPRITE_WIDTH,
        constants::WIZARD_SPRITE_HEIGHT,
    );

    // UV transform for first frame: scale to 1/3 to show only one cell
    let grid_size = constants::WIZARD_SPRITE_GRID_SIZE as f32;
    let frame_scale = 1.0 / grid_size;
    let uv_transform = bevy::math::Affine2::from_scale(Vec2::splat(frame_scale));

    // The co-op GUEST wizard uses a distinct idle sheet so the two players are
    // visually distinguishable; everyone else (host, versus) uses the default.
    let sprite_texture = if coop && !is_host_wizard {
        wizard_assets.guest_sprite_texture.clone()
    } else {
        wizard_assets.sprite_texture.clone()
    };
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(sprite_texture),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        uv_transform,
        ..default()
    });

    // Apply archetype-identity stat bonuses via the shared helper so MP and SP
    // stay in sync (BoringOleMage, Shepherd, …). Insight/progression bonuses are
    // intentionally omitted in MP to keep matches balanced.
    let mut wizard = Wizard::new(constants::DEFAULT_SPELL_RANGE);
    crate::game::units::wizard::systems::apply_archetype_stat_bonuses(&mut wizard, wizard_type);
    // Versus-only spell-range buff (+5% reach). Applies to every archetype. Co-op
    // plays on the single-player battlefield, so it keeps SP range to match
    // single-player balance (and stay consistent with the co-op host's own wizard,
    // which is spawned via the SP `setup_wizard` with no buff). The Arcanorouter
    // recomputes its range each frame in `apply_bonuses_to_wizard_stats`, which
    // applies the same versus-only multiplier, so this spawn-time value is just its
    // first frame.
    if !coop {
        wizard.spell_range *= constants::MP_SPELL_RANGE_MULTIPLIER;
    }

    let mut entity_commands = commands.spawn((
        Mesh3d(meshes.add(quad_mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(position),
        hitbox,
        Health::new(constants::HEALTH),
        MovementSpeed(0.0),
        Mana::new(constants::MANA),
        ManaRegen::new(constants::MANA_REGEN),
        CastingState::new(),
        wizard,
        WizardAnimation::new(),
        Billboard,
        OnMultiplayerGameScreen,
    ));

    // Skip priming the default spell for archetypes that shouldn't start with
    // Magic Missile (mirrors single-player `setup_wizard`): Warglock fires guns,
    // the Randomancer only casts what its roulette rolls, and the Shepherd can't
    // cast offensive spells. Without this guard their clicks would fall through
    // to casting Magic Missile.
    if !matches!(
        wizard_type,
        crate::config::WizardType::Warglock
            | crate::config::WizardType::Randomancer
            | crate::config::WizardType::Shepherd
    ) {
        entity_commands.insert(magic_missile_constants::PRIMED_MAGIC_MISSILE);
    }

    // Wizards are spawned locally on both peers and synced via spell visuals,
    // not snapshots — keep them out of the snapshot stream (the co-op guest
    // wizard proxy carries `Team::Defenders`, which would otherwise be ghosted).
    entity_commands.insert(crate::game::multiplayer::components::NoSnapshot);

    // Add wizard role markers
    // LocalWizard: the wizard this player controls (host's own or guest's own)
    // GuestWizard: the guest's wizard as seen by the host (for spell command processing)
    if (is_host_wizard && role == PeerRole::Host) || (!is_host_wizard && role == PeerRole::Guest) {
        entity_commands.insert(LocalWizard);
    }
    if !is_host_wizard && role == PeerRole::Host {
        entity_commands.insert(GuestWizard);
    }

    if wizard_type == crate::config::WizardType::Arcanorouter {
        entity_commands.insert(
            crate::game::units::wizard::archetypes::arcanorouter::ArcanoRouterBonuses::default(),
        );
    }
}
