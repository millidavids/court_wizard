use crate::game::terrain::messages::TerrainDamageMessage;
use crate::game::units::components::{
    Health, Hitbox, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::{Spell, Wizard};
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::disintegrate::constants;
use crate::game::units::wizard::spells::fireball;
use crate::game::units::wizard::spells::utils::{UniqueHitTracker, local_player_team};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

/// System that applies damage to all units hit by disintegrate beams.
///
/// This is a high-risk spell that damages both attackers and defenders,
/// but not the wizard.
#[allow(clippy::too_many_arguments)]
pub fn apply_disintegrate_damage(
    mut commands: Commands,
    mut beam_query: Query<(&mut DisintegrateBeam, &mut UniqueHitTracker)>,
    mut target_query: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        (
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    time: Res<Time>,
    visual_assets: Res<SpellVisualAssets>,
    mut talent_progress: Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    mut terrain_damage: MessageWriter<TerrainDamageMessage>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    for (mut beam, mut hit_tracker) in beam_query.iter_mut() {
        beam.update_damage_timer(time.delta_secs());
        beam.update_time_alive(time.delta_secs());

        // Advance resonance timer and spawn mini-fireballs along beam
        if beam.resonance {
            beam.resonance_timer += time.delta_secs();
            if beam.resonance_timer >= constants::MINI_FIREBALL_INTERVAL {
                beam.resonance_timer -= constants::MINI_FIREBALL_INTERVAL;
                let current_len = beam.current_length();
                if current_len > 1.0 {
                    let scale = beam.mini_spell_scale;
                    let mini_damage = fireball::constants::TOTAL_DAMAGE
                        * constants::MINI_FIREBALL_DAMAGE_FRACTION
                        * scale;
                    let explosion_radius = fireball::constants::EXPLOSION_RADIUS
                        * constants::MINI_FIREBALL_DAMAGE_FRACTION
                        * scale;
                    let visual_radius = 8.0 * scale;
                    // Annihilation beams: spawn fireball at impact point (ground level),
                    // flying outward in XZ. Normal beams: spawn at origin, fly along beam.
                    let (spawn_pos, velocity) = if beam.annihilation {
                        let tip = beam.origin + beam.direction * current_len;
                        let ground_pos = Vec3::new(tip.x, 0.0, tip.z);
                        // Random outward XZ direction
                        let angle = beam.resonance_timer * 137.5; // pseudo-random angle
                        let xz_dir = Vec3::new(angle.cos(), 0.0, angle.sin()).normalize();
                        (
                            ground_pos,
                            xz_dir * fireball::constants::PROJECTILE_SPEED * 0.5,
                        )
                    } else {
                        (
                            beam.origin,
                            beam.direction * fireball::constants::PROJECTILE_SPEED,
                        )
                    };
                    fireball::systems::spawn_fireball_entity(
                        &mut commands,
                        &visual_assets,
                        spawn_pos,
                        velocity,
                        mini_damage,
                        beam.damage_type,
                        explosion_radius,
                        fireball::constants::PROJECTILE_COLLISION_RADIUS * scale,
                        beam.empowerment,
                        visual_radius,
                    );
                }
            }
        }

        if beam.should_damage() {
            let mut hit_count = 0_u32;
            let damage = beam.damage_per_tick();

            if !(beam.annihilation && beam.origin.y > 50.0) {
                let tip = beam.origin + beam.direction * beam.current_length();
                terrain_damage.write(TerrainDamageMessage {
                    position: tip,
                    radius: 0.0,
                    damage,
                    damage_type: beam.damage_type,
                });
            }

            for (entity, transform, hitbox, mut health, mut temp_hp, has_spell_shield, team) in
                target_query.iter_mut()
            {
                if beam.intersects_hitbox_cylinder(
                    transform.translation,
                    hitbox.radius,
                    hitbox.height,
                ) {
                    apply_spell_damage_with_team(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        damage,
                        beam.damage_type,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                    if hit_tracker.track_hit(entity) {
                        hit_count += 1;
                    }
                }
            }

            if hit_count > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::Disintegrate, hit_count);
            }

            beam.reset_damage_timer();
        }
    }
}
