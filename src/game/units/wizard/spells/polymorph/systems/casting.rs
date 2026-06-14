use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard,
};
use super::super::components::PolymorphTalentParams;
use super::super::constants;
use super::core::polymorph_casting_logic;
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Corpse, Health, PolymorphedModifier, Team};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_cursor_to_spell_range_with_origin,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellSoundId;
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> PolymorphTalentParams {
    let mut params = PolymorphTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Polymorph, 0);
    let t2 = talents.get_selection(Spell::Polymorph, 1);
    let t3 = talents.get_selection(Spell::Polymorph, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Extended Transformation
            params.duration = constants::EXTENDED_DURATION;
        }
        Some(1) => {
            // Fragile Form
            params.sheep_hp = constants::FRAGILE_SHEEP_HP;
        }
        Some(2) => {
            // Quick Shapeshift
            params.cast_time_mult = constants::QUICK_SHAPESHIFT_CAST_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.explosive = true,
        Some(1) => params.contagious = true,
        Some(2) => params.pig_form = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.permanent = true,
        Some(1) => params.mass = true,
        Some(2) => params.dire = true,
        _ => {}
    }

    params
}

/// Local wizard polymorph casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_polymorph_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    targets_query: Query<
        (
            Entity,
            &Transform,
            &Health,
            &MeshMaterial3d<StandardMaterial>,
            &Mesh3d,
            &Team,
        ),
        (
            Without<Corpse>,
            Without<PolymorphedModifier>,
            Without<Wizard>,
        ),
    >,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    target_assist: Res<TargetAssistWorldPos>,
    talent_resources: (
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
    ),
    local_origin: Res<LocalSpellOrigin>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let (active_talents, mut talent_progress) = talent_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((_wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Polymorph {
        return;
    }

    let cursor_pos = clamp_cursor_to_spell_range_with_origin(
        input.cursor_pos,
        local_origin.0,
        wizard.spell_range,
        0.0,
    );
    input.cursor_pos = cursor_pos;

    let talent_params = compute_talent_params(active_talents.as_deref());

    let completed = polymorph_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &mut materials,
        &targets_query,
        &talent_params,
        &visual_assets,
        &mut pending_cast_events,
    );

    if completed > 0 {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Transmutation,
            time.elapsed_secs(),
        );
        if let Some(pos) = cursor_pos {
            audio::play_sfx_synced(
                &mut commands,
                &mut pending_cast_events,
                SpellSoundId::PolymorphCast,
                pos,
                &game_config,
                &sfx,
            );
        }
        mouse_state.left_consumed = true;
        if let Some(ref mut progress) = talent_progress {
            progress.increment(Spell::Polymorph, completed);
        }
    }
}
