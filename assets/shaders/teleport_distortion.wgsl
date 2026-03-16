// Teleport spatial distortion post-processing shader.
// Applies radial ripple/warp effects at up to 4 screen-space points.
// Used for teleport cast VFX and persistent Dimensional Rift portals.

struct FullscreenVertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct TeleportDistortionData {
    count: f32,
    time: f32,
    strength: f32,
    frequency: f32,
    speed: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    // Point 0
    point_0_x: f32,
    point_0_y: f32,
    point_0_radius: f32,
    point_0_intensity: f32,
    // Point 1
    point_1_x: f32,
    point_1_y: f32,
    point_1_radius: f32,
    point_1_intensity: f32,
    // Point 2
    point_2_x: f32,
    point_2_y: f32,
    point_2_radius: f32,
    point_2_intensity: f32,
    // Point 3
    point_3_x: f32,
    point_3_y: f32,
    point_3_radius: f32,
    point_3_intensity: f32,
}
@group(0) @binding(2) var<uniform> settings: TeleportDistortionData;

/// Computes branchless radial ripple distortion at a single point.
/// Creates expanding concentric rings that push pixels outward.
fn ripple_offset(uv: vec2<f32>, center: vec2<f32>, radius: f32, intensity: f32, is_active: f32) -> vec2<f32> {
    let delta = uv - center;
    let dist = max(length(delta), 0.001);
    let direction = delta / dist;

    // Normalized distance within influence radius (0 at center, 1 at edge)
    let t = dist / max(radius, 0.001);

    // Fade in from center, fade out at edge
    let falloff = smoothstep(0.0, 0.3, t) * smoothstep(1.0, 0.5, t);

    // Concentric ripple waves expanding outward
    let wave = sin(t * settings.frequency - settings.time * settings.speed);

    return direction * wave * falloff * intensity * settings.strength * is_active;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Branchless activation per point
    let active_0 = step(0.5, settings.count);
    let active_1 = step(1.5, settings.count);
    let active_2 = step(2.5, settings.count);
    let active_3 = step(3.5, settings.count);

    let offset_0 = ripple_offset(in.uv, vec2<f32>(settings.point_0_x, settings.point_0_y), settings.point_0_radius, settings.point_0_intensity, active_0);
    let offset_1 = ripple_offset(in.uv, vec2<f32>(settings.point_1_x, settings.point_1_y), settings.point_1_radius, settings.point_1_intensity, active_1);
    let offset_2 = ripple_offset(in.uv, vec2<f32>(settings.point_2_x, settings.point_2_y), settings.point_2_radius, settings.point_2_intensity, active_2);
    let offset_3 = ripple_offset(in.uv, vec2<f32>(settings.point_3_x, settings.point_3_y), settings.point_3_radius, settings.point_3_intensity, active_3);

    let distorted_uv = clamp(in.uv + offset_0 + offset_1 + offset_2 + offset_3, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));

    return textureSample(screen_texture, texture_sampler, distorted_uv);
}
