#import bevy_pbr::forward_io::VertexOutput

struct StoneUniforms {
    dark_color: vec4<f32>,
    light_color: vec4<f32>,
    // mode 0 (circular): x=center_x, y=center_z, z=inner_radius, w=outer_radius
    // mode 1 (wall edge): x=wall_x, y=fade_start, z=fade_end, w=unused
    blend_params: vec4<f32>,
    // x = blend mode (0 = circular, 1 = wall edge)
    blend_mode: vec4<f32>,
};

@group(3) @binding(0)
var<uniform> material: StoneUniforms;

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

    // Color noise
    let noise = fbm2(world_xz * 0.005) * 0.6 + fbm2(world_xz * 0.018 + vec2<f32>(50.0, 80.0)) * 0.4;
    let color = mix(material.dark_color.rgb, material.light_color.rgb, noise);

    // Noise for organic edge warping
    let warp = fbm2(world_xz * 0.004 + vec2<f32>(200.0, 400.0));

    var alpha = 1.0;

    if material.blend_mode.x < 0.5 {
        // Circular blend: fade based on distance from center point
        let center = vec2<f32>(material.blend_params.x, material.blend_params.y);
        let inner_r = material.blend_params.z;
        let outer_r = material.blend_params.w;

        let dist = length(world_xz - center);
        let noisy_outer = outer_r + (warp - 0.5) * outer_r * 0.4;
        alpha = 1.0 - smoothstep(inner_r, noisy_outer, dist);
    } else {
        // Wall edge blend: signed distance from a diagonal line matching the art border
        let line_a = vec2<f32>(1000.0, 1500.0);
        let line_b = vec2<f32>(2100.0, -2800.0);

        let ab = line_b - line_a;
        let ap = world_xz - line_a;
        let signed_dist = (ab.x * ap.y - ab.y * ap.x) / length(ab);

        let fade_start = material.blend_params.y;
        let fade_end = material.blend_params.z;

        let noisy_end = fade_end + (warp - 0.5) * fade_end * 0.7;
        alpha = smoothstep(-noisy_end, -fade_start, signed_dist);

        // Clip near the back wall
        let z_fade = smoothstep(-3050.0, -2850.0, world_xz.y);
        alpha = alpha * z_fade;
    }

    return vec4<f32>(color, alpha);
}
