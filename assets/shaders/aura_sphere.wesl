#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct AuraMaterial {
    inner_color: vec4<f32>,
    outer_color: vec4<f32>,
    opacity: f32,
    time: f32,
};

@group(3) @binding(0)
var<uniform> material: AuraMaterial;

// --- Procedural noise (same foundation as fire_particle.wgsl) ---

fn hash2(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash2(i);
    let b = hash2(i + vec2<f32>(1.0, 0.0));
    let c = hash2(i + vec2<f32>(0.0, 1.0));
    let d = hash2(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm2(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var freq_p = p;

    value += amplitude * noise2d(freq_p);
    freq_p = freq_p * 2.1;
    amplitude *= 0.5;
    value += amplitude * noise2d(freq_p);
    freq_p = freq_p * 2.1;
    amplitude *= 0.5;
    value += amplitude * noise2d(freq_p);

    return value;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Fresnel for edge glow
    let n = normalize(in.world_normal);
    let v = normalize(view.world_position.xyz - in.world_position.xyz);
    let facing = dot(n, v);
    let fresnel = 1.0 - clamp(facing, 0.0, 1.0);

    // Swirling energy noise using surface normal direction (moves with the sphere)
    let t = material.time;
    let local_dir = normalize(in.world_normal);
    let swirl_uv = local_dir.xz * 2.0
        + vec2<f32>(t * 0.12, t * 0.09);
    let energy1 = fbm2(swirl_uv);
    // Second layer at different scale and speed for depth
    let swirl_uv2 = local_dir.xz * 3.5
        + vec2<f32>(-t * 0.08, t * 0.14);
    let energy2 = fbm2(swirl_uv2);
    // Third layer for extra turbulence
    let swirl_uv3 = local_dir.xz * 5.0
        + vec2<f32>(t * 0.06, -t * 0.11);
    let energy3 = fbm2(swirl_uv3);
    let energy = (energy1 + energy2 + energy3) / 3.0;

    // Subtle edge glow + visible swirling energy bands
    let edge_glow = pow(fresnel, 4.0) * 0.15;
    // Energy bands: threshold the noise to create visible streaks, not uniform fill
    let band = smoothstep(0.35, 0.55, energy);
    let interior = band * (1.0 - fresnel) * 0.12;
    // Gentle pulse
    let pulse = 0.95 + 0.05 * sin(t * 1.5);
    let combined = (edge_glow + interior) * pulse;

    let color = mix(material.inner_color, material.outer_color, fresnel);
    let alpha = combined * material.opacity;
    return vec4<f32>(color.rgb, clamp(alpha, 0.0, 0.03));
}
