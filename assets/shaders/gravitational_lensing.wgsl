// Gravitational lensing post-processing shader
// Applies UV distortion, center darkening, and screen darkening for black holes.
// Slots 0-1: black holes (lensing + center darkening).
// Slots 2-3: rift endpoints (lensing only, no darkening).
// Runs as a separate fullscreen pass before the CRT effect.

struct FullscreenVertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct LensingData {
    lensing_count: f32,
    lensing_strength: f32,
    lensing_darkening: f32,
    lensing_0_x: f32,
    lensing_0_y: f32,
    lensing_0_radius: f32,
    lensing_1_x: f32,
    lensing_1_y: f32,
    lensing_1_radius: f32,
    lensing_2_x: f32,
    lensing_2_y: f32,
    lensing_2_radius: f32,
    lensing_3_x: f32,
    lensing_3_y: f32,
    lensing_3_radius: f32,
}
@group(0) @binding(2) var<uniform> settings: LensingData;

/// Computes branchless UV offset pulling toward a lensing center.
/// Creates a ring-shaped distortion: no effect at the center,
/// peaks partway out, and fades to zero at the influence radius edge.
fn lensing_offset(uv: vec2<f32>, center: vec2<f32>, radius: f32, is_active: f32) -> vec2<f32> {
    let to_center = center - uv;
    let dist = max(length(to_center), 0.001);
    // Inner dead zone: no distortion inside the visual center (~40% of influence radius)
    let inner_edge = radius * 0.4;
    let inner_fade = smoothstep(0.0, inner_edge, dist);
    // Outer falloff: distortion fades to zero closer to center
    let outer_fade = smoothstep(radius * 0.6, inner_edge, dist);
    let t = inner_fade * outer_fade;
    let direction = to_center / dist;
    return direction * t * settings.lensing_strength * is_active;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Gravitational lensing: pull UVs toward centers (branchless).
    let active_0 = step(0.5, settings.lensing_count);
    let active_1 = step(1.5, settings.lensing_count);
    let active_2 = step(2.5, settings.lensing_count);
    let active_3 = step(3.5, settings.lensing_count);

    let lens_0 = lensing_offset(in.uv, vec2<f32>(settings.lensing_0_x, settings.lensing_0_y), settings.lensing_0_radius, active_0);
    let lens_1 = lensing_offset(in.uv, vec2<f32>(settings.lensing_1_x, settings.lensing_1_y), settings.lensing_1_radius, active_1);
    let lens_2 = lensing_offset(in.uv, vec2<f32>(settings.lensing_2_x, settings.lensing_2_y), settings.lensing_2_radius, active_2);
    let lens_3 = lensing_offset(in.uv, vec2<f32>(settings.lensing_3_x, settings.lensing_3_y), settings.lensing_3_radius, active_3);

    let lensed_uv = clamp(in.uv + lens_0 + lens_1 + lens_2 + lens_3, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));

    // Sample the screen with distorted UVs.
    var color = textureSample(screen_texture, texture_sampler, lensed_uv);

    // --- Post-sample effects (safe to branch) ---

    // Gradually darken screen as black holes grow.
    if (settings.lensing_darkening > 0.0) {
        color = vec4<f32>(color.rgb * (1.0 - settings.lensing_darkening), 1.0);
    }

    // Darken center of black holes to pure black (slots 0-1 only, not rifts).
    // Check radius > 0 instead of lensing_count to avoid false darkening when
    // only rift endpoints (slots 2-3) are active and slots 0-1 are empty.
    if (settings.lensing_0_radius > 0.001) {
        let bh0_center = vec2<f32>(settings.lensing_0_x, settings.lensing_0_y);
        let bh0_inner = settings.lensing_0_radius * 0.3;
        let bh0_dist = length(in.uv - bh0_center);
        let bh0_dark = smoothstep(0.0, bh0_inner, bh0_dist);
        color = vec4<f32>(color.rgb * bh0_dark, 1.0);
    }
    if (settings.lensing_1_radius > 0.001) {
        let bh1_center = vec2<f32>(settings.lensing_1_x, settings.lensing_1_y);
        let bh1_inner = settings.lensing_1_radius * 0.3;
        let bh1_dist = length(in.uv - bh1_center);
        let bh1_dark = smoothstep(0.0, bh1_inner, bh1_dist);
        color = vec4<f32>(color.rgb * bh1_dark, 1.0);
    }
    // Slots 2-3 (rifts): lensing only, no center darkening.

    return color;
}
