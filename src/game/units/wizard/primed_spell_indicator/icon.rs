//! The floating icon entity: its marker components, tuning, spawn, and bob.

use bevy::prelude::*;

use crate::game::constants::UNIT_SCALE;
use crate::game::units::boss::utils::sinusoidal_bob;
use crate::game::units::wizard::components::{LocalWizard, Wizard};
use crate::game::units::wizard::constants::WIZARD_SPRITE_HEIGHT;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Marks the icon quad floating above the local wizard's head.
///
/// Holds the texture it is currently showing so the material is only rewritten
/// when the icon actually changes — `PrimedSpell` and `GunState` are both
/// dirtied every frame by bookkeeping, so change detection is too noisy to
/// gate on.
#[derive(Component, Default)]
pub(super) struct PrimedSpellIcon(pub Option<Handle<Image>>);

/// Marks the slightly larger white copy sitting behind the icon.
///
/// Same texture, so the icon's own alpha shapes it — only the rim peeks out
/// past the icon, which reads as a soft corona.
#[derive(Component)]
pub(super) struct PrimedSpellIconCorona;

/// Icon edge length in world units.
pub(super) const ICON_SIZE: f32 = 14.0 * UNIT_SCALE;

/// Resting height above the wizard's origin. The sprite is centered on its
/// transform, so its head is at `WIZARD_SPRITE_HEIGHT / 2.0` (132); this sits
/// just clear of it at the top of the bob.
///
/// Height is not purely cosmetic here: the camera looks down at the wizard, so
/// raising the icon also shortens its distance along the camera's forward axis,
/// which shrinks the projected half-width and pushes it *sideways* toward the
/// screen edge. Keeping it low is half of what keeps it on screen.
pub(super) const ICON_BASE_Y: f32 = WIZARD_SPRITE_HEIGHT / 2.0 + 10.0 * UNIT_SCALE;

/// How far to slide the icon from directly-overhead toward the middle of the
/// battlefield, in world units.
///
/// The wizard stands in the far corner, close to the camera, so an icon parked
/// straight above its head projects past the left edge of the letterboxed 16:9
/// viewport. This is deliberately small — just enough to clear the edge. Larger
/// values send the icon drifting up and inward over the battlefield, where it
/// stops reading as belonging to the wizard.
pub(super) const ICON_CENTER_NUDGE: f32 = 20.0 * UNIT_SCALE;

/// Corona size relative to the icon.
const CORONA_SCALE: f32 = 1.22;

/// Corona opacity — a hint of glow, not a halo.
const CORONA_ALPHA: f32 = 0.35;

/// How far behind the icon the corona sits, in the icon's local space.
///
/// Local `-Z` points away from the camera (the parent billboard maps `NEG_Z` to
/// camera-forward), so this keeps the corona reliably sorted behind the icon.
const CORONA_Z_OFFSET: f32 = -0.02;

pub(super) const BOB_AMPLITUDE: f32 = 3.0 * UNIT_SCALE;

/// Calmer than the ingredient drops' 1.5 Hz — this one is on screen constantly.
pub(super) const BOB_FREQUENCY_HZ: f32 = 1.0;

/// Attaches the icon to the local wizard as a child.
///
/// Child rather than an independent follower: `Children` is a linked-spawn
/// relationship, so the icon despawns with the wizard and needs no screen
/// marker of its own. It also deliberately carries no `Billboard` — the parent
/// wizard has one, and `update_billboards` writes a Y-axis-only yaw, which
/// hands the child its camera-facing rotation for free.
///
/// The wizard spawns during `AppState::Loading`, when this system's run
/// condition is still false. That is fine: Bevy does not advance a skipped
/// system's `last_run` tick, so `Added` is still true on the first frame this
/// actually runs. `setup_spell_range_indicator` relies on the same behavior.
pub(super) fn spawn_primed_spell_icon(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    wizard_query: Query<Entity, (Added<Wizard>, With<LocalWizard>)>,
) {
    for wizard in wizard_query.iter() {
        // Both materials must be per-entity: `sync_primed_spell_icon` swaps
        // their `base_color_texture`, so neither can be a shared handle.
        let icon_mat = materials.add(icon_material(Color::WHITE));
        let corona_mat = materials.add(icon_material(Color::WHITE.with_alpha(CORONA_ALPHA)));

        let icon = commands
            .spawn((
                Mesh3d(visual_assets.unit_rect.clone()),
                MeshMaterial3d(icon_mat),
                // Position is rewritten every frame by `bob_primed_spell_icon`.
                Transform::from_xyz(0.0, ICON_BASE_Y, 0.0).with_scale(Vec3::splat(ICON_SIZE)),
                // Stays hidden until something is actually primed.
                Visibility::Hidden,
                PrimedSpellIcon::default(),
            ))
            .id();

        let corona = commands
            .spawn((
                Mesh3d(visual_assets.unit_rect.clone()),
                MeshMaterial3d(corona_mat),
                Transform::from_xyz(0.0, 0.0, CORONA_Z_OFFSET)
                    .with_scale(Vec3::splat(CORONA_SCALE)),
                PrimedSpellIconCorona,
            ))
            .id();

        commands.entity(icon).add_child(corona);
        commands.entity(wizard).add_child(icon);
    }
}

/// The unlit, blended material both quads use, tinted by `base_color`.
fn icon_material(base_color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color,
        base_color_texture: None, // filled in by `sync_primed_spell_icon`
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    }
}

/// Holds the icon at its offset above the wizard and bobs it on a sine wave.
///
/// [`ICON_CENTER_NUDGE`] is a *world*-space offset, but the icon is a child of
/// a wizard whose rotation the billboard rewrites every frame — so the nudge is
/// counter-rotated into the parent's frame here. The vertical part needs no
/// such treatment: the billboard yaw is about Y, which leaves Y untouched.
pub(super) fn bob_primed_spell_icon(
    time: Res<Time>,
    wizards: Query<&Transform, (With<LocalWizard>, Without<PrimedSpellIcon>)>,
    mut icons: Query<&mut Transform, With<PrimedSpellIcon>>,
) {
    let Ok(wizard) = wizards.single() else {
        return;
    };

    let toward_center =
        Vec3::new(-wizard.translation.x, 0.0, -wizard.translation.z).normalize_or_zero();
    let local_nudge = wizard.rotation.inverse() * (toward_center * ICON_CENTER_NUDGE);
    let height = ICON_BASE_Y + sinusoidal_bob(BOB_AMPLITUDE, BOB_FREQUENCY_HZ, time.elapsed_secs());

    for mut transform in icons.iter_mut() {
        transform.translation = local_nudge + Vec3::Y * height;
    }
}
