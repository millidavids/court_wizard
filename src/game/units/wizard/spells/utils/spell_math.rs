use bevy::prelude::*;

/// Returns the XZ-plane distance between two points (ignoring Y).
pub(crate) fn xz_distance(a: Vec3, b: Vec3) -> f32 {
    Vec3::new(a.x - b.x, 0.0, a.z - b.z).length()
}

/// Tests whether a sphere intersects a vertical cylinder (unit hitbox).
///
/// The cylinder extends from ground (Y=0) to `cylinder_height` at `cylinder_pos`,
/// with the given `cylinder_radius`. The sphere is centered at `sphere_center`
/// with `sphere_radius`.
///
/// Algorithm: clamp the sphere center's Y to the cylinder's Y range, then check
/// whether the 2D (XZ) distance is less than the sum of both radii.
pub(crate) fn sphere_intersects_cylinder(
    sphere_center: Vec3,
    sphere_radius: f32,
    cylinder_pos: Vec3,
    cylinder_radius: f32,
    cylinder_height: f32,
) -> bool {
    // Clamp sphere Y to cylinder vertical range
    let clamped_y = sphere_center
        .y
        .clamp(cylinder_pos.y, cylinder_pos.y + cylinder_height);
    let dy = sphere_center.y - clamped_y;

    // XZ distance between centers
    let dx = sphere_center.x - cylinder_pos.x;
    let dz = sphere_center.z - cylinder_pos.z;
    let xz_dist_sq = dx * dx + dz * dz;

    // Combined horizontal reach
    let combined_radius = sphere_radius + cylinder_radius;

    // Full distance check: horizontal + vertical
    xz_dist_sq + dy * dy <= combined_radius * combined_radius
}

/// Returns the shortest XZ-plane distance from a point to a line segment defined by start/end.
pub(crate) fn distance_to_line_segment_xz(point: Vec3, start: Vec3, end: Vec3) -> f32 {
    let p = Vec2::new(point.x, point.z);
    let a = Vec2::new(start.x, start.z);
    let b = Vec2::new(end.x, end.z);
    let ab = b - a;
    let ap = p - a;
    let ab_len_sq = ab.length_squared();
    if ab_len_sq < 0.0001 {
        return ap.length();
    }
    let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let closest = a + ab * t;
    (p - closest).length()
}

/// Clamps a target position to be within the wizard's spell range using 3D distance.
///
/// If the target is beyond `spell_range` from `wizard_pos`, it is moved along the
/// direction vector to sit exactly at `spell_range` distance.
pub(crate) fn clamp_to_spell_range(target: Vec3, wizard_pos: Vec3, spell_range: f32) -> Vec3 {
    let diff = target - wizard_pos;
    let distance = diff.length();

    if distance > spell_range {
        wizard_pos + diff.normalize() * spell_range
    } else {
        target
    }
}

/// Computes the ground-plane radius of a spell range circle, accounting for wizard height.
///
/// The wizard sits at height `wizard_height` above the ground. A spell with 3D range
/// `spell_range` can reach a ground circle of radius `sqrt(spell_range² - wizard_height²)`.
/// Returns 0.0 if the wizard is higher than the spell range.
pub(crate) fn ground_projected_range(spell_range: f32, wizard_height: f32) -> f32 {
    if wizard_height < spell_range {
        (spell_range * spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    }
}

/// Clamps a target position to be within the wizard's spell range on the ground plane,
/// accounting for the wizard's height above ground and an optional effect radius.
///
/// Uses the Pythagorean theorem to compute the maximum ground-plane radius from the
/// wizard's XZ position. If `effect_radius` is non-zero, the clamp ensures the entire
/// effect circle stays within range.
pub(crate) fn clamp_to_spell_range_ground(
    target: Vec3,
    wizard_pos: Vec3,
    spell_range: f32,
    effect_radius: f32,
) -> Vec3 {
    let max_ground_radius = ground_projected_range(spell_range, wizard_pos.y);

    // Account for effect radius so entire circle stays within range
    let max_center_distance = (max_ground_radius - effect_radius).max(0.0);

    // Calculate XZ plane distance from wizard to target
    let direction = target - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();

    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        wizard_pos + normalized_direction * max_center_distance
    } else {
        target
    }
}

/// Like the (deprecated) un-suffixed variant but accepts an explicit local origin —
/// callers should pass the `LocalSpellOrigin` resource so the clamp is computed
/// from the correct wizard position on both the host and the guest.
pub(crate) fn clamp_cursor_to_spell_range_with_origin(
    cursor_pos: Option<Vec3>,
    local_origin: Vec3,
    spell_range: f32,
    effect_radius: f32,
) -> Option<Vec3> {
    let pos = cursor_pos?;
    Some(clamp_to_spell_range_ground(
        pos,
        local_origin,
        spell_range,
        effect_radius,
    ))
}
