#import bevy_pbr::forward_io::VertexOutput

struct FireExplosionMaterial {
    inner_color: vec4<f32>,
    outer_color: vec4<f32>,
};

@group(3) @binding(0)
var<uniform> material: FireExplosionMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv - vec2<f32>(0.5, 0.5)) * 2.0;
    // Power curve: yellow only in the very center, quickly fades to orange
    let t = clamp(pow(dist, 0.4), 0.0, 1.0);
    return mix(material.inner_color, material.outer_color, t);
}
