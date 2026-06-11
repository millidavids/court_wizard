//! Ray beam attacks: mind control, fear, teleport, petrification visuals.

pub(super) mod beam_helpers;
pub(super) mod disintegration;
pub(super) mod fear_beam;
pub(super) mod mind_control_beam;
pub(super) mod petrification;
pub(super) mod teleportation;

pub(crate) use disintegration::find_nearest_defender_direction_from;
pub(crate) use disintegration::find_nearest_defender_position;
pub(crate) use disintegration::find_units_in_cone;
pub(crate) use disintegration::update_ray_disintegrate_visuals;
pub(crate) use fear_beam::despawn_fear_beam;
pub(crate) use fear_beam::ray_fear_beam;
pub(crate) use fear_beam::update_ray_fear_visuals;
pub(crate) use mind_control_beam::despawn_mind_control_beam;
pub(crate) use mind_control_beam::ray_mind_control_beam;
pub(crate) use mind_control_beam::update_ray_mind_control_visuals;
pub(crate) use petrification::update_petrified_damage;
pub(crate) use teleportation::ray_teleport_eye;
pub(crate) use teleportation::update_ray_teleport_bubbles;
