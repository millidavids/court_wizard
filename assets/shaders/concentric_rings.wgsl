#import bevy_ui::ui_vertex_output UiVertexOutput

struct ConcentricRingsMaterial {
    fill_color: vec4<f32>,
    bg_color: vec4<f32>,
    pending_color: vec4<f32>,
    /// Fractional filled level (e.g. 2.6 = 2 full rings + 60% clockwise fill on 3rd).
    filled: f32,
    /// Fractional pending level on top of filled (e.g. 0.8 = 80% clockwise of next ring).
    pending: f32,
    /// Total number of rings.
    total: f32,
}

@group(1) @binding(0)
var<uniform> material: ConcentricRingsMaterial;

const PI: f32 = 3.14159265358979323846;
const TAU: f32 = 6.28318530717958647692;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - 1.0;
    let dist = length(uv);

    // Rings span from inner to outer
    let ring_region = 0.55;
    let outer = 1.0;
    let inner = outer - ring_region;
    let gap_frac = 0.15;

    if dist < inner || dist > outer {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Normalize dist within ring region: 0 = inner, 1 = outer
    let norm = (dist - inner) / ring_region;

    // Determine which ring (0 = innermost, total-1 = outermost)
    let ring_width = 1.0 / material.total;
    let ring_index = floor(norm / ring_width);
    let within_ring = fract(norm / ring_width);

    // Gap between rings
    if within_ring > (1.0 - gap_frac) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Anti-aliased edges
    let edge_smooth = 0.06;
    let ring_alpha = smoothstep(0.0, edge_smooth, within_ring)
                   * smoothstep(1.0 - gap_frac, 1.0 - gap_frac - edge_smooth, within_ring);
    let outer_alpha = 1.0 - smoothstep(outer - 0.02, outer, dist);
    let inner_alpha = smoothstep(inner - 0.02, inner + 0.02, dist);
    let alpha = ring_alpha * outer_alpha * inner_alpha;

    // Clockwise angle from top (same convention as radial_progress.wgsl)
    let angle = atan2(uv.x, -uv.y);
    let norm_angle = (angle + TAU) % TAU;

    let filled_f = material.filled;
    let pending_end = filled_f + material.pending;

    // Fully filled rings
    let filled_whole = floor(filled_f);
    let filled_frac = filled_f - filled_whole;

    let pending_whole = floor(pending_end);
    let pending_frac = pending_end - pending_whole;

    if ring_index < filled_whole {
        // Fully filled ring
        return vec4<f32>(material.fill_color.rgb, material.fill_color.a * alpha);
    } else if ring_index == filled_whole && filled_frac > 0.0 {
        // Partially filled ring — clockwise fill
        let fill_threshold = filled_frac * TAU;
        if norm_angle <= fill_threshold {
            return vec4<f32>(material.fill_color.rgb, material.fill_color.a * alpha);
        }
        // Check if pending covers the rest of this ring
        if ring_index < pending_whole {
            return vec4<f32>(material.pending_color.rgb, material.pending_color.a * alpha);
        } else if ring_index == pending_whole && pending_frac > 0.0 {
            let pending_threshold = pending_frac * TAU;
            if norm_angle <= pending_threshold {
                return vec4<f32>(material.pending_color.rgb, material.pending_color.a * alpha);
            }
        }
        return vec4<f32>(material.bg_color.rgb, material.bg_color.a * alpha);
    } else if ring_index < pending_whole {
        // Fully pending ring
        return vec4<f32>(material.pending_color.rgb, material.pending_color.a * alpha);
    } else if ring_index == pending_whole && pending_frac > 0.0 {
        // Partially pending ring — clockwise fill
        let pending_threshold = pending_frac * TAU;
        if norm_angle <= pending_threshold {
            return vec4<f32>(material.pending_color.rgb, material.pending_color.a * alpha);
        }
        return vec4<f32>(material.bg_color.rgb, material.bg_color.a * alpha);
    } else {
        // Empty ring
        return vec4<f32>(material.bg_color.rgb, material.bg_color.a * alpha);
    }
}
