#import bevy_pbr::forward_io::VertexOutput

struct WindSwayUniforms {
    base_color: vec4<f32>,
    uv_params: vec4<f32>,     // xy = scale, zw = offset
    sway_params: vec4<f32>,   // x = time, y = amplitude, z = speed, w = phase
};

@group(3) @binding(0)
var<uniform> material: WindSwayUniforms;

@group(3) @binding(1)
var base_texture: texture_2d<f32>;

@group(3) @binding(2)
var base_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Apply sprite sheet UV transform (scale + offset)
    let uv_scale = material.uv_params.xy;
    let uv_offset = material.uv_params.zw;
    var uv = in.uv * uv_scale + uv_offset;

    // Wind sway: horizontal UV offset that increases with height.
    // uv.y=0 is top of sprite (maximum sway), uv.y=1 is bottom (no sway).
    let height_factor = 1.0 - in.uv.y;
    let height_curve = height_factor * height_factor; // Quadratic falloff

    let t = material.sway_params.x;
    let amplitude = material.sway_params.y;
    let speed = material.sway_params.z;
    let phase = material.sway_params.w;

    // Primary sway + secondary harmonic for organic feel
    let primary = sin(t * speed + phase);
    let secondary = sin(t * speed * 2.3 + phase * 1.7) * 0.4;
    let sway = (primary + secondary) * amplitude * height_curve;

    // Apply sway to U axis, clamped within the sprite's tile
    uv.x = clamp(uv.x + sway, uv_offset.x, uv_offset.x + uv_scale.x);

    // Sample texture and apply tint
    let tex_color = textureSample(base_texture, base_sampler, uv);
    let final_color = tex_color * material.base_color;

    // Alpha mask (discard if alpha < 0.5)
    if final_color.a < 0.5 {
        discard;
    }

    return final_color;
}
