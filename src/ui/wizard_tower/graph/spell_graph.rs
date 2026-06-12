use crate::game::units::wizard::components::Spell;

use super::routing::compute_edge_waypoints;
use super::types::{SpellEdgeDef, SpellNodeDef};

/// Pushes overlapping nodes apart using iterative repulsion.
/// Preserves overall topology while ensuring minimum spacing.
fn separate_overlapping_nodes(nodes: &mut [SpellNodeDef]) {
    use bevy::prelude::Vec2;

    let min_distance: f32 = 90.0;

    for _ in 0..80 {
        let mut forces = vec![Vec2::ZERO; nodes.len()];
        let mut any_overlap = false;

        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let delta = nodes[i].position - nodes[j].position;
                let dist = delta.length();
                if dist < min_distance {
                    any_overlap = true;
                    let overlap = min_distance - dist;
                    let dir = if dist > 0.1 {
                        delta / dist
                    } else {
                        // Nudge in a consistent direction for exactly overlapping nodes
                        Vec2::new(1.0, 0.3)
                    };
                    forces[i] += dir * overlap * 0.5;
                    forces[j] -= dir * overlap * 0.5;
                }
            }
        }

        if !any_overlap {
            break;
        }

        for (i, node) in nodes.iter_mut().enumerate() {
            // Don't move the center anchor node
            if node.spell.is_none() {
                continue;
            }
            node.position += forces[i];
        }
    }
}

/// Builds the full spell graph with node positions and edges.
///
/// Radial web layout centered at origin with 4 category roots:
///
/// - (0, 0): Central "Free" anchor node (visual hub, not a spell)
/// - Offense (MagicMissile): upper-right quadrant
/// - Control (Entangle): upper-left quadrant
/// - Support (GuardianCircle): lower-left quadrant
/// - Utility (Telekinesis): lower-right quadrant
///
/// Nodes are placed using polar coordinates (angle + distance from center)
/// so every branch radiates outward like a web.
pub(crate) fn build_spell_graph() -> (Vec<SpellNodeDef>, Vec<SpellEdgeDef>) {
    use bevy::prelude::Vec2;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Place a node at (angle°, distance) from origin and connect it
    let add = |nodes: &mut Vec<SpellNodeDef>,
               edges: &mut Vec<SpellEdgeDef>,
               spell: Spell,
               angle_deg: f32,
               distance: f32,
               from: Option<Spell>| {
        let angle = angle_deg.to_radians();
        let pos = Vec2::new(angle.cos() * distance, angle.sin() * distance);
        nodes.push(SpellNodeDef {
            spell: Some(spell),
            position: pos,
        });
        edges.push(SpellEdgeDef {
            from_spell: from,
            to_spell: spell,
            waypoints: Vec::new(),
        });
    };

    // Ring distances from center
    let r1: f32 = 110.0; // Root category nodes
    let r2: f32 = 230.0; // Depth 2
    let r3: f32 = 350.0; // Depth 3
    let r4: f32 = 470.0; // Depth 4

    // Central anchor node
    nodes.push(SpellNodeDef {
        spell: None,
        position: Vec2::ZERO,
    });

    // -----------------------------------------------------------------------
    // Offense (upper-right quadrant, centered around -45°)
    // -----------------------------------------------------------------------
    add(&mut nodes, &mut edges, Spell::MagicMissile, -45.0, r1, None);

    // MagicMissile → 4 children fanning from -70° to -15°
    add(
        &mut nodes,
        &mut edges,
        Spell::PlagueWind,
        -68.0,
        r2,
        Some(Spell::MagicMissile),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Disintegrate,
        -53.0,
        r2,
        Some(Spell::MagicMissile),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::ChainLightning,
        -35.0,
        r2,
        Some(Spell::MagicMissile),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::FingerOfDeath,
        -15.0,
        r2,
        Some(Spell::MagicMissile),
    );

    // Disintegrate → Fireball → MeteorFall
    add(
        &mut nodes,
        &mut edges,
        Spell::Fireball,
        -56.0,
        r3,
        Some(Spell::Disintegrate),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::MeteorFall,
        -58.0,
        r4,
        Some(Spell::Fireball),
    );

    // ChainLightning → LightningRod
    add(
        &mut nodes,
        &mut edges,
        Spell::LightningRod,
        -32.0,
        r3,
        Some(Spell::ChainLightning),
    );

    // FingerOfDeath → MarkOfDeath
    add(
        &mut nodes,
        &mut edges,
        Spell::MarkOfDeath,
        -12.0,
        r3,
        Some(Spell::FingerOfDeath),
    );

    // -----------------------------------------------------------------------
    // Control (upper-left quadrant, centered around -135°)
    // -----------------------------------------------------------------------
    add(&mut nodes, &mut edges, Spell::Entangle, -135.0, r1, None);

    // Entangle → 3 children fanning from -158° to -115°
    add(
        &mut nodes,
        &mut edges,
        Spell::Grease,
        -157.0,
        r2,
        Some(Spell::Entangle),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::SpikeGrowth,
        -135.0,
        r2,
        Some(Spell::Entangle),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Sleep,
        -116.0,
        r2,
        Some(Spell::Entangle),
    );

    // Grease → WallOfFire
    add(
        &mut nodes,
        &mut edges,
        Spell::WallOfFire,
        -160.0,
        r3,
        Some(Spell::Grease),
    );

    // SpikeGrowth → WallOfStone, Squall
    add(
        &mut nodes,
        &mut edges,
        Spell::WallOfStone,
        -141.0,
        r3,
        Some(Spell::SpikeGrowth),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Squall,
        -129.0,
        r3,
        Some(Spell::SpikeGrowth),
    );

    // Sleep → MindControl, Polymorph → BlackHole
    add(
        &mut nodes,
        &mut edges,
        Spell::MindControl,
        -122.0,
        r3,
        Some(Spell::Sleep),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Polymorph,
        -110.0,
        r3,
        Some(Spell::Sleep),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::BlackHole,
        -108.0,
        r4,
        Some(Spell::Polymorph),
    );

    // -----------------------------------------------------------------------
    // Support (lower-left quadrant, centered around 135°)
    // -----------------------------------------------------------------------
    add(
        &mut nodes,
        &mut edges,
        Spell::GuardianCircle,
        135.0,
        r1,
        None,
    );

    // GuardianCircle → 3 children fanning from 115° to 158°
    add(
        &mut nodes,
        &mut edges,
        Spell::BattleHymn,
        118.0,
        r2,
        Some(Spell::GuardianCircle),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::FogCloud,
        140.0,
        r2,
        Some(Spell::GuardianCircle),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::BerserkerRage,
        158.0,
        r2,
        Some(Spell::GuardianCircle),
    );

    // BattleHymn → HealingPlume, Haste
    add(
        &mut nodes,
        &mut edges,
        Spell::HealingPlume,
        110.0,
        r3,
        Some(Spell::BattleHymn),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Haste,
        125.0,
        r3,
        Some(Spell::BattleHymn),
    );

    // FogCloud → Teleport
    add(
        &mut nodes,
        &mut edges,
        Spell::Teleport,
        143.0,
        r3,
        Some(Spell::FogCloud),
    );

    // BerserkerRage → RaiseTheDead
    add(
        &mut nodes,
        &mut edges,
        Spell::RaiseTheDead,
        162.0,
        r3,
        Some(Spell::BerserkerRage),
    );

    // -----------------------------------------------------------------------
    // Utility (lower-right quadrant, centered around 45°)
    // -----------------------------------------------------------------------
    add(&mut nodes, &mut edges, Spell::Telekinesis, 45.0, r1, None);

    // Telekinesis → 3 children fanning from 28° to 62°
    add(
        &mut nodes,
        &mut edges,
        Spell::Dispel,
        28.0,
        r2,
        Some(Spell::Telekinesis),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::Banishment,
        45.0,
        r2,
        Some(Spell::Telekinesis),
    );
    add(
        &mut nodes,
        &mut edges,
        Spell::ArcaneCrystal,
        62.0,
        r2,
        Some(Spell::Telekinesis),
    );

    // Push apart any nodes that overlap
    separate_overlapping_nodes(&mut nodes);

    // Compute edge waypoints (clipped to node edges, routed around obstacles)
    compute_edge_waypoints(&nodes, &mut edges);

    (nodes, edges)
}
