#import bevy_ui::ui_vertex_output::UiVertexOutput

struct ParchmentData {
    base_color: vec4<f32>,
    /// How much noise texture variation to apply (0.0 = solid, 1.0 = full).
    texture_strength: f32,
    /// Edge darkening intensity (0.0 = none, 1.0 = strong vignette).
    vignette_strength: f32,
    /// Scale of the noise pattern (higher = finer grain).
    noise_scale: f32,
    _padding: f32,
}

@group(1) @binding(0)
var<uniform> material: ParchmentData;

// Hash for pseudo-random values.
fn hash21(p: vec2<f32>) -> f32 {
    let n = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(n) * 43758.5453);
}

// Smooth value noise with quintic interpolation (no visible grid edges).
fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    // Quintic Hermite: 6t^5 - 15t^4 + 10t^3 (C2 continuous, no grid artifacts).
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);

    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Layered noise (fractal Brownian motion).
fn fbm(p: vec2<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var pos = p;
    for (var i = 0; i < 5; i++) {
        val += amp * value_noise(pos);
        pos *= 2.0;
        amp *= 0.5;
    }
    return val;
}

// Domain-warped FBM: uses noise to distort its own coordinates,
// completely eliminating grid-aligned artifacts.
fn warped_fbm(p: vec2<f32>) -> f32 {
    let warp = vec2<f32>(
        fbm(p + vec2<f32>(0.0, 0.0)),
        fbm(p + vec2<f32>(5.2, 1.3)),
    );
    return fbm(p + warp * 0.4);
}

// Returns alpha for rounded-corner clipping using the node's border_radius and size.
fn rounded_rect_alpha(uv: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    let pixel = uv * size;
    let half = size * 0.5;
    let r = select(
        select(radii.w, radii.z, pixel.x > half.x),
        select(radii.x, radii.y, pixel.x > half.x),
        pixel.y < half.y
    );
    let q = abs(pixel - half) - half + vec2<f32>(r, r);
    let dist = length(max(q, vec2<f32>(0.0, 0.0))) - r;
    return 1.0 - smoothstep(-1.0, 0.5, dist);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let scale = material.noise_scale;
    let base = material.base_color;

    // Domain-warped noise for organic parchment texture (no grid artifacts).
    let texture = warped_fbm(uv * scale);

    // Linear additive bias — lighter patches on dark backgrounds.
    let light_bias = texture * material.texture_strength;
    // Warm-tinted variation: more red/yellow, less blue (parchment feel).
    var col = base.rgb + vec3<f32>(light_bias * 0.8, light_bias * 0.65, light_bias * 0.35);

    // Edge vignette — darken toward edges for depth.
    let center = uv - 0.5;
    let dist = length(center * vec2<f32>(1.2, 1.0));
    let vignette = 1.0 - smoothstep(0.3, 0.7, dist) * material.vignette_strength;
    col *= vignette;

    // Clip to rounded corners.
    let alpha = rounded_rect_alpha(uv, in.size, in.border_radius);

    return vec4<f32>(col, base.a * alpha);
}
