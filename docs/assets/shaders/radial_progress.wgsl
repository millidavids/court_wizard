#import bevy_ui::ui_vertex_output UiVertexOutput

struct RadialProgressMaterial {
    fill_color: vec4<f32>,
    bg_color: vec4<f32>,
    progress: f32,
    ring_width: f32,
}

@group(1) @binding(0)
var<uniform> material: RadialProgressMaterial;

const PI: f32 = 3.14159265358979323846;
const TAU: f32 = 6.28318530717958647692;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // UV centered at (0,0), range [-1, 1]
    let uv = in.uv * 2.0 - 1.0;
    let dist = length(uv);

    // Ring region: between (1 - ring_width) and 1
    let outer = 1.0;
    let inner = 1.0 - material.ring_width;

    if dist < inner || dist > outer {
        // Outside the ring — transparent
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Smooth edges for anti-aliasing
    let edge_smooth = 0.02;
    let ring_alpha = smoothstep(inner - edge_smooth, inner + edge_smooth, dist)
                   * (1.0 - smoothstep(outer - edge_smooth, outer, dist));

    // Angle from top (12 o'clock), clockwise: remap atan2 so 0 = top, increases CW
    let angle = atan2(uv.x, -uv.y); // 0 at top, positive CW
    let norm_angle = (angle + TAU) % TAU; // [0, TAU)
    let fill_threshold = material.progress * TAU;

    if norm_angle <= fill_threshold {
        return vec4<f32>(material.fill_color.rgb, material.fill_color.a * ring_alpha);
    } else {
        return vec4<f32>(material.bg_color.rgb, material.bg_color.a * ring_alpha);
    }
}
