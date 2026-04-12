// High contrast post-processing shader.
// Applies sigmoidal contrast curve + saturation boost to improve
// visibility for low-vision players and bright ambient conditions.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct HighContrastData {
    strength: f32,
    enabled: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(2) var<uniform> settings: HighContrastData;

const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const CONTRAST_GAIN: f32 = 10.0;
const SATURATION_BOOST: f32 = 0.4;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);

    if settings.enabled < 0.5 {
        return color;
    }

    let rgb = color.rgb;
    let luma = dot(rgb, LUMA);

    // Sigmoidal contrast on luminance
    let contrasted_luma = 1.0 / (1.0 + exp(-CONTRAST_GAIN * settings.strength * (luma - 0.5)));

    // Scale RGB to preserve hue while applying new luminance
    let scale = select(contrasted_luma / luma, 0.0, luma < 0.001);
    let contrasted_rgb = rgb * scale;

    // Boost saturation
    let gray = vec3<f32>(contrasted_luma);
    let sat_factor = 1.0 + SATURATION_BOOST * settings.strength;
    let saturated = mix(gray, contrasted_rgb, sat_factor);

    let result = clamp(saturated, vec3<f32>(0.0), vec3<f32>(1.0));
    let final_rgb = mix(rgb, result, settings.strength);

    return vec4<f32>(final_rgb, color.a);
}
