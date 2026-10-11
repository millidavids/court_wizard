#import bevy_pbr::forward_io::VertexOutput

struct WallUniforms {
    // x = tile_world_size, y = tile_count, z = tile_index, w = noise_intensity
    tile_params: vec4<f32>,
    // rgb = damage tint color, a = blend factor (0 = healthy, 1 = fully damaged)
    damage_tint: vec4<f32>,
    // rgb = dark stone color (STONE_COLOR_DARK)
    side_dark: vec4<f32>,
    // rgb = light stone color (STONE_COLOR_LIGHT)
    side_light: vec4<f32>,
};

@group(3) @binding(0)
var<uniform> material: WallUniforms;

@group(3) @binding(1)
var base_texture: texture_2d<f32>;

@group(3) @binding(2)
var base_sampler: sampler;

// --- Hash-based 2D noise (same as ground_noise.wgsl) ---

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
    let world_xz = in.world_position.xz;
    let world_normal = normalize(in.world_normal);

    // Face discrimination: top face vs side faces
    let is_top = step(0.7, abs(world_normal.y));

    // --- Top face: battlefield tile texture + ground noise ---
    let tile_world_size = material.tile_params.x;
    let tile_count = material.tile_params.y;
    let tile_index = material.tile_params.z;
    let noise_intensity = material.tile_params.w;

    // World-space UVs that tile seamlessly with the ground
    let tile_u_width = 1.0 / tile_count;
    let tile_u_start = tile_index * tile_u_width;
    let top_uv = vec2<f32>(
        tile_u_start + fract(world_xz.x / tile_world_size) * tile_u_width,
        fract(world_xz.y / tile_world_size),
    );
    let tex_color = textureSample(base_texture, base_sampler, top_uv);

    // Ground noise overlay (same algorithm as ground_noise.wgsl)
    let n1 = fbm2(world_xz * 0.003);
    let n2 = fbm2(world_xz * 0.012 + vec2<f32>(50.0, 80.0));
    let combined = (n1 - 0.5) * 0.7 + (n2 - 0.5) * 0.3;

    let brightness_shift = combined * noise_intensity * 0.6;
    var top_color = tex_color.rgb * (1.0 + brightness_shift);

    // Brown earth tint in dark patches (matches ground_noise.wgsl earth_tint)
    let earth_tint = vec3<f32>(0.35, 0.28, 0.18);
    let earth_factor = max(0.0, -combined) * noise_intensity * 0.3;
    top_color = mix(top_color, tex_color.rgb * earth_tint * 2.0, earth_factor);

    // Yellow-green sun tint in light patches
    let sun_tint = vec3<f32>(1.1, 1.15, 0.95);
    let sun_factor = max(0.0, combined) * noise_intensity * 0.15;
    top_color = mix(top_color, top_color * sun_tint, sun_factor);

    // Dirt border around the top face edges (using mesh UVs for edge detection)
    let border_width = 0.08;
    let edge_dist = min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y));
    // Noise-warped edge for organic look
    let edge_noise = fbm2(world_xz * 0.02 + vec2<f32>(100.0, 200.0));
    let warped_border = border_width * (0.6 + edge_noise * 0.8);
    let border_factor = smoothstep(warped_border, warped_border * 0.3, edge_dist);
    top_color = mix(top_color, material.side_light.rgb, border_factor);

    // --- Side faces: procedural brown perlin noise (colors from uniforms) ---
    let dark_brown = material.side_dark.rgb;
    let light_brown = material.side_light.rgb;

    // Use world XZ + Y for side noise so pattern varies vertically too
    let side_uv = vec2<f32>(
        world_xz.x + world_xz.y,
        in.world_position.y,
    );
    let side_n1 = fbm2(side_uv * 0.005);
    let side_n2 = fbm2(side_uv * 0.018 + vec2<f32>(200.0, 300.0));
    let side_blend = side_n1 * 0.6 + side_n2 * 0.4;

    var side_color = mix(dark_brown, light_brown, side_blend);

    // Add subtle brightness variation
    let side_detail = fbm2(side_uv * 0.03 + vec2<f32>(500.0, 700.0));
    side_color = side_color * (0.85 + side_detail * 0.3);

    // --- Blend top and side based on normal ---
    var color = mix(side_color, top_color, is_top);

    // --- Damage tint ---
    color = mix(color, material.damage_tint.rgb, material.damage_tint.a);

    return vec4<f32>(color, 1.0);
}
