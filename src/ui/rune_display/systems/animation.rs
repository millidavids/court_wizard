use bevy::prelude::*;

use super::super::components::*;

/// System to handle the activated spell name fade timer.
pub(crate) fn update_spell_name_fade(
    time: Res<Time>,
    mut commands: Commands,
    mut fade_query: Query<
        (Entity, &mut SpellNameFadeTimer, &mut TextColor),
        With<ActivatedSpellText>,
    >,
    mut shadow_query: Query<
        (&mut Text, &mut TextColor),
        (With<ActivatedSpellTextShadow>, Without<ActivatedSpellText>),
    >,
) {
    for (entity, mut timer, mut color) in &mut fade_query {
        timer.elapsed += time.delta_secs();

        let alpha = (1.0 - (timer.elapsed / timer.duration)).max(0.0);
        color.0.set_alpha(alpha);

        // Fade shadow in sync
        if let Ok((_, mut shadow_color)) = shadow_query.single_mut() {
            shadow_color.0.set_alpha(alpha * 0.5);
        }

        if timer.elapsed >= timer.duration {
            commands.entity(entity).remove::<SpellNameFadeTimer>();
            color.0.set_alpha(0.0);

            // Clear both texts
            if let Ok((mut shadow_text, mut shadow_color)) = shadow_query.single_mut() {
                **shadow_text = "".to_string();
                shadow_color.0.set_alpha(0.0);
            }
        }
    }
}
