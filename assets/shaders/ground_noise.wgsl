#import bevy_pbr::forward_io::VertexOutput

struct GroundUniforms {
    // x = base noise intensity, y = forest noise intensity,
    // z = forest Z start (full forest below this), w = blend range
    params: vec4<f32>,
};

@group(3) @binding(0)
var<uniform> material: GroundUniforms;

@group(3) @binding(1)
var base_texture: texture_2d<f32>;

@group(3) @binding(2)
var base_sampler: sampler;

// --- Hash-based 2D noise ---

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
    let tex_color = textureSample(base_texture, base_sampler, in.uv);
    let world_xz = in.world_position.xz;

    // Compute noise intensity from world Z position — smooth gradient from
    // forest (high intensity) to open field (base intensity).
    // Use noise to make the transition edge irregular rather than a straight line.
    let base_intensity = material.params.x;
    let forest_intensity = material.params.y;
    let forest_z = material.params.z;
    let blend_range = material.params.w;

    let edge_noise = fbm2(world_xz * 0.005 + vec2<f32>(300.0, 500.0));
    let noisy_offset = (edge_noise - 0.5) * blend_range * 0.5;
    let blend_t = smoothstep(forest_z, forest_z + blend_range + noisy_offset, world_xz.y);
    let intensity = mix(forest_intensity, base_intensity, blend_t);

    // Primary noise: broad color variation
    let n1 = fbm2(world_xz * 0.003);

    // Secondary noise: finer detail
    let n2 = fbm2(world_xz * 0.012 + vec2<f32>(50.0, 80.0));

    // Combined noise centered around 0
    let combined = (n1 - 0.5) * 0.7 + (n2 - 0.5) * 0.3;

    // Darken/lighten the base color
    let brightness_shift = combined * intensity * 0.6;
    var color = tex_color.rgb * (1.0 + brightness_shift);

    // Brown tint in darker patches (bare earth/dirt)
    let earth_tint = vec3<f32>(0.35, 0.28, 0.18);
    let earth_factor = max(0.0, -combined) * intensity * 0.3;
    color = mix(color, tex_color.rgb * earth_tint * 2.0, earth_factor);

    // Yellow-green tint in lighter patches (sun-lit grass)
    let sun_tint = vec3<f32>(1.1, 1.15, 0.95);
    let sun_factor = max(0.0, combined) * intensity * 0.15;
    color = mix(color, color * sun_tint, sun_factor);

    // Darken near the back wall (Z → -3000) to deepen the forest floor
    let back_wall_z = -3000.0;
    let darken_range = 800.0;
    let back_darken = smoothstep(back_wall_z + darken_range, back_wall_z, world_xz.y) * 0.3;
    color = color * (1.0 - back_darken);

    return vec4<f32>(color, tex_color.a);
}
