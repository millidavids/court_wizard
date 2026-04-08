#import bevy_ui::ui_vertex_output UiVertexOutput

struct ButtonGradientData {
    top_color: vec4<f32>,
    bottom_color: vec4<f32>,
    /// Highlight intensity at top edge (0.0 = none, 1.0 = full).
    highlight: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
}

@group(1) @binding(0)
var<uniform> material: ButtonGradientData;

// Returns alpha for rounded-corner clipping using the node's border_radius and size.
// border_radius order: top_left, top_right, bottom_right, bottom_left (pixels).
fn rounded_rect_alpha(uv: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    let pixel = uv * size;
    let half = size * 0.5;

    // Pick corner radius based on which quadrant the pixel is in.
    let r = select(
        select(radii.w, radii.z, pixel.x > half.x),
        select(radii.x, radii.y, pixel.x > half.x),
        pixel.y < half.y
    );

    // Signed distance to rounded rectangle edge.
    let q = abs(pixel - half) - half + vec2<f32>(r, r);
    let dist = length(max(q, vec2<f32>(0.0, 0.0))) - r;

    return 1.0 - smoothstep(-1.0, 0.5, dist);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let v = in.uv.y;

    // Vertical gradient from top to bottom.
    var col = mix(material.top_color, material.bottom_color, v);

    // Subtle highlight along the top edge (simulates a light catch).
    let edge_highlight = smoothstep(0.15, 0.0, v) * material.highlight;
    col = col + vec4<f32>(edge_highlight, edge_highlight, edge_highlight * 0.8, 0.0);

    // Clip to rounded corners.
    let alpha = rounded_rect_alpha(in.uv, in.size, in.border_radius);
    col.a *= alpha;

    return col;
}
