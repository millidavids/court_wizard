// Colorblind correction post-processing shader.
// Implements daltonization (Machado et al. 2009) to correct colors
// for protanopia, deuteranopia, and tritanopia.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct ColorblindData {
    sim_row0: vec4<f32>,
    sim_row1: vec4<f32>,
    sim_row2: vec4<f32>,
    err_row0: vec4<f32>,
    err_row1: vec4<f32>,
    err_row2: vec4<f32>,
    strength: f32,
    enabled: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(2) var<uniform> settings: ColorblindData;

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(0.0)), vec3<f32>(2.2));
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.2));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(screen_texture, texture_sampler, in.uv);

    if settings.enabled < 0.5 {
        return color;
    }

    let linear = srgb_to_linear(color.rgb);

    let sim = mat3x3<f32>(
        settings.sim_row0.xyz,
        settings.sim_row1.xyz,
        settings.sim_row2.xyz,
    );
    let simulated = sim * linear;
    let error = linear - simulated;

    let err_mat = mat3x3<f32>(
        settings.err_row0.xyz,
        settings.err_row1.xyz,
        settings.err_row2.xyz,
    );
    let correction = err_mat * error;

    let corrected = clamp(linear + correction * settings.strength, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(linear_to_srgb(corrected), color.a);
}
