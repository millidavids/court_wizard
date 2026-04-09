#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct FireExplosionMaterial {
    inner_color: vec4<f32>,
    outer_color: vec4<f32>,
    opacity: f32,
};

@group(3) @binding(0)
var<uniform> material: FireExplosionMaterial;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Fresnel-based gradient: faces pointing at camera = inner color, edges = outer color
    let n = normalize(in.world_normal);
    let view_dir = normalize(view.world_position.xyz - in.world_position.xyz);
    let facing = dot(n, view_dir);
    let dist = 1.0 - clamp(facing, 0.0, 1.0);

    // Same power curve as the cross-plane version
    let t = clamp(pow(dist, 0.4), 0.0, 1.0);
    let color = mix(material.inner_color, material.outer_color, t);

    // Edge fade for natural fire look (slightly translucent center, transparent rim)
    let edge_alpha = clamp(1.0 - pow(dist, 2.0), 0.0, 0.8);
    let alpha = edge_alpha * material.opacity;
    return vec4<f32>(color.rgb, alpha);
}
