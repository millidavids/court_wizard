use bevy::prelude::*;

use crate::game::units::wizard::components::Spell;

/// Definition of a node in the spell graph.
pub(super) struct SpellNodeDef {
    /// The spell this node represents, or `None` for the central anchor.
    pub spell: Option<Spell>,
    /// Graph-space coordinates, (0,0) = center.
    pub position: Vec2,
}

/// Definition of an edge in the spell graph.
pub(super) struct SpellEdgeDef {
    /// Source spell (`None` = central anchor node).
    pub from_spell: Option<Spell>,
    /// Destination spell.
    pub to_spell: Spell,
}

/// Pushes overlapping nodes apart using iterative repulsion.
/// Preserves overall topology while ensuring minimum spacing.
fn separate_overlapping_nodes(nodes: &mut [SpellNodeDef]) {
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
/// Layout centered at origin:
/// - (0, 0): Central "Free" anchor node (visual hub, not a spell)
/// - Near center: MagicMissile + Telekinesis (free defaults)
/// - Radial clusters extending outward per element chain
/// - Gate spells: radiate from center at varying distances
/// - After placement, overlapping nodes are pushed apart
pub(super) fn build_spell_graph() -> (Vec<SpellNodeDef>, Vec<SpellEdgeDef>) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Central anchor node
    nodes.push(SpellNodeDef {
        spell: None,
        position: Vec2::ZERO,
    });

    // -----------------------------------------------------------------------
    // Free default spells (near center)
    // -----------------------------------------------------------------------
    nodes.push(SpellNodeDef {
        spell: Some(Spell::MagicMissile),
        position: Vec2::new(-80.0, -50.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: None,
        to_spell: Spell::MagicMissile,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::Telekinesis),
        position: Vec2::new(80.0, -50.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: None,
        to_spell: Spell::Telekinesis,
    });

    // -----------------------------------------------------------------------
    // Fire chain (upper-right): Disintegrate → Fireball → WallOfFire → MeteorFall
    // -----------------------------------------------------------------------
    let fire_base = Vec2::new(180.0, -160.0);
    let fire_step = Vec2::new(100.0, -70.0);

    nodes.push(SpellNodeDef {
        spell: Some(Spell::Disintegrate),
        position: fire_base,
    });
    edges.push(SpellEdgeDef {
        from_spell: None,
        to_spell: Spell::Disintegrate,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::Fireball),
        position: fire_base + fire_step,
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::Disintegrate),
        to_spell: Spell::Fireball,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::WallOfFire),
        position: fire_base + fire_step * 2.0,
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::Fireball),
        to_spell: Spell::WallOfFire,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::MeteorFall),
        position: fire_base + fire_step * 3.0,
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::WallOfFire),
        to_spell: Spell::MeteorFall,
    });

    // -----------------------------------------------------------------------
    // Electric chain (right): ChainLightning → LightningRod
    // -----------------------------------------------------------------------
    nodes.push(SpellNodeDef {
        spell: Some(Spell::ChainLightning),
        position: Vec2::new(300.0, 10.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: None,
        to_spell: Spell::ChainLightning,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::LightningRod),
        position: Vec2::new(430.0, 40.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::ChainLightning),
        to_spell: Spell::LightningRod,
    });

    // -----------------------------------------------------------------------
    // Necrotic chain (lower-right): FingerOfDeath → {RaiseTheDead, MarkOfDeath → PlagueWind}
    // -----------------------------------------------------------------------
    let necro_base = Vec2::new(200.0, 180.0);

    nodes.push(SpellNodeDef {
        spell: Some(Spell::FingerOfDeath),
        position: necro_base,
    });
    edges.push(SpellEdgeDef {
        from_spell: None,
        to_spell: Spell::FingerOfDeath,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::RaiseTheDead),
        position: necro_base + Vec2::new(140.0, -60.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::FingerOfDeath),
        to_spell: Spell::RaiseTheDead,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::MarkOfDeath),
        position: necro_base + Vec2::new(140.0, 60.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::FingerOfDeath),
        to_spell: Spell::MarkOfDeath,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::PlagueWind),
        position: necro_base + Vec2::new(280.0, 60.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::MarkOfDeath),
        to_spell: Spell::PlagueWind,
    });

    // -----------------------------------------------------------------------
    // Force chain (lower-left): GuardianCircle → {Haste → BattleHymn, BerserkerRage}
    // -----------------------------------------------------------------------
    let force_base = Vec2::new(-180.0, 200.0);

    nodes.push(SpellNodeDef {
        spell: Some(Spell::GuardianCircle),
        position: force_base,
    });
    edges.push(SpellEdgeDef {
        from_spell: None,
        to_spell: Spell::GuardianCircle,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::Haste),
        position: force_base + Vec2::new(-130.0, 90.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::GuardianCircle),
        to_spell: Spell::Haste,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::BattleHymn),
        position: force_base + Vec2::new(-260.0, 140.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::Haste),
        to_spell: Spell::BattleHymn,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::BerserkerRage),
        position: force_base + Vec2::new(-130.0, -50.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::GuardianCircle),
        to_spell: Spell::BerserkerRage,
    });

    // -----------------------------------------------------------------------
    // Nature chain (upper-left): Grease → Entangle → {SpikeGrowth, HealingPlume}
    // -----------------------------------------------------------------------
    let nature_base = Vec2::new(-200.0, -160.0);

    nodes.push(SpellNodeDef {
        spell: Some(Spell::Grease),
        position: nature_base,
    });
    edges.push(SpellEdgeDef {
        from_spell: None,
        to_spell: Spell::Grease,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::Entangle),
        position: nature_base + Vec2::new(-130.0, -70.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::Grease),
        to_spell: Spell::Entangle,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::SpikeGrowth),
        position: nature_base + Vec2::new(-260.0, -30.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::Entangle),
        to_spell: Spell::SpikeGrowth,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::HealingPlume),
        position: nature_base + Vec2::new(-260.0, -120.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::Entangle),
        to_spell: Spell::HealingPlume,
    });

    // -----------------------------------------------------------------------
    // Earth chain (left): WallOfStone → Teleport
    // -----------------------------------------------------------------------
    nodes.push(SpellNodeDef {
        spell: Some(Spell::WallOfStone),
        position: Vec2::new(-300.0, 40.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: None,
        to_spell: Spell::WallOfStone,
    });

    nodes.push(SpellNodeDef {
        spell: Some(Spell::Teleport),
        position: Vec2::new(-430.0, 20.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::WallOfStone),
        to_spell: Spell::Teleport,
    });

    // -----------------------------------------------------------------------
    // Gate spells (radiate from center, distance proportional to required_total_spells)
    // -----------------------------------------------------------------------
    // Angles chosen to fill gaps between element chains.
    // Using negative Y = up, positive Y = down in UI coordinates.
    let gate_spells: &[(Spell, f32)] = &[
        (Spell::Dispel, 350.0_f32.to_radians()),        // 3 req - close, near fire
        (Spell::Squall, 80.0_f32.to_radians()),          // 4 req - between electric/necrotic
        (Spell::FogCloud, 130.0_f32.to_radians()),       // 5 req - between necrotic/force
        (Spell::Sleep, 160.0_f32.to_radians()),           // 6 req - between force/earth
        (Spell::ArcaneCrystal, 310.0_f32.to_radians()),  // 6 req - between fire/nature
        (Spell::Banishment, 230.0_f32.to_radians()),     // 7 req - between earth/nature
        (Spell::BlackHole, 260.0_f32.to_radians()),      // 8 req - far left-down
        (Spell::Polymorph, 280.0_f32.to_radians()),      // 8 req - far left-up
    ];

    for &(spell, angle) in gate_spells {
        let required = spell.required_total_spells();
        let distance = 140.0 + required as f32 * 35.0;
        let position = Vec2::new(angle.cos() * distance, angle.sin() * distance);

        nodes.push(SpellNodeDef {
            spell: Some(spell),
            position,
        });
        edges.push(SpellEdgeDef {
            from_spell: None,
            to_spell: spell,
        });
    }

    // MindControl branches from Sleep (positioned further out along Sleep's direction)
    let sleep_pos = nodes
        .iter()
        .find(|n| n.spell == Some(Spell::Sleep))
        .map(|n| n.position)
        .unwrap_or_default();
    nodes.push(SpellNodeDef {
        spell: Some(Spell::MindControl),
        position: sleep_pos + Vec2::new(-110.0, 70.0),
    });
    edges.push(SpellEdgeDef {
        from_spell: Some(Spell::Sleep),
        to_spell: Spell::MindControl,
    });

    // Push apart any nodes that overlap
    separate_overlapping_nodes(&mut nodes);

    (nodes, edges)
}
