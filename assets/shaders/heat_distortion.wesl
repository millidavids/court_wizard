// Heat distortion post-processing shader
// Applies rippling UV displacement near wall of fire locations.
// Runs as a separate fullscreen pass before the CRT effect.

struct FullscreenVertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct HeatDistortionData {
    count: f32,
    strength: f32,
    time: f32,
    _pad0: f32,
    wall_0_start_x: f32,
    wall_0_start_y: f32,
    wall_0_end_x: f32,
    wall_0_end_y: f32,
    wall_1_start_x: f32,
    wall_1_start_y: f32,
    wall_1_end_x: f32,
    wall_1_end_y: f32,
    wall_2_start_x: f32,
    wall_2_start_y: f32,
    wall_2_end_x: f32,
    wall_2_end_y: f32,
    wall_3_start_x: f32,
    wall_3_start_y: f32,
    wall_3_end_x: f32,
    wall_3_end_y: f32,
    wall_0_radius: f32,
    wall_1_radius: f32,
    wall_2_radius: f32,
    wall_3_radius: f32,
}
@group(0) @binding(2) var<uniform> settings: HeatDistortionData;

/// Distance from point p to line segment a-b.
fn dist_to_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let len_sq = dot(ab, ab);
    // Degenerate segment (point)
    if (len_sq < 0.00001) {
        return length(ap);
    }
    let t = clamp(dot(ap, ab) / len_sq, 0.0, 1.0);
    let closest = a + ab * t;
    return length(p - closest);
}

/// Computes heat distortion offset for a single wall segment.
fn heat_offset(uv: vec2<f32>, wall_start: vec2<f32>, wall_end: vec2<f32>, radius: f32, is_active: f32) -> vec2<f32> {
    let dist = dist_to_segment(uv, wall_start, wall_end);

    // Smooth falloff from wall center to influence radius
    let falloff = smoothstep(radius, 0.0, dist);

    // Gentle, slow sine waves for subtle heat shimmer
    let t = settings.time;
    let wave1 = sin(uv.y * 40.0 + t * 2.0) * 0.4;
    let wave2 = sin(uv.y * 70.0 + t * 3.3 + uv.x * 20.0) * 0.35;
    let wave3 = sin(uv.x * 25.0 + t * 1.5 + uv.y * 15.0) * 0.25;

    let ripple = wave1 + wave2 + wave3;

    // Mostly vertical displacement (heat rises)
    let dx = ripple * settings.strength * 0.3 * falloff * is_active;
    let dy = ripple * settings.strength * falloff * is_active;

    return vec2<f32>(dx, dy);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Early out: no active walls — passthrough
    if (settings.count < 0.5) {
        return textureSample(screen_texture, texture_sampler, in.uv);
    }

    // Accumulate distortion from all active walls (branchless activation)
    let is_active_0 = step(0.5, settings.count);
    let is_active_1 = step(1.5, settings.count);
    let is_active_2 = step(2.5, settings.count);
    let is_active_3 = step(3.5, settings.count);

    var offset = vec2<f32>(0.0, 0.0);
    offset += heat_offset(in.uv,
        vec2<f32>(settings.wall_0_start_x, settings.wall_0_start_y),
        vec2<f32>(settings.wall_0_end_x, settings.wall_0_end_y),
        settings.wall_0_radius, is_active_0);
    offset += heat_offset(in.uv,
        vec2<f32>(settings.wall_1_start_x, settings.wall_1_start_y),
        vec2<f32>(settings.wall_1_end_x, settings.wall_1_end_y),
        settings.wall_1_radius, is_active_1);
    offset += heat_offset(in.uv,
        vec2<f32>(settings.wall_2_start_x, settings.wall_2_start_y),
        vec2<f32>(settings.wall_2_end_x, settings.wall_2_end_y),
        settings.wall_2_radius, is_active_2);
    offset += heat_offset(in.uv,
        vec2<f32>(settings.wall_3_start_x, settings.wall_3_start_y),
        vec2<f32>(settings.wall_3_end_x, settings.wall_3_end_y),
        settings.wall_3_radius, is_active_3);

    let distorted_uv = clamp(in.uv + offset, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
    return textureSample(screen_texture, texture_sampler, distorted_uv);
}
