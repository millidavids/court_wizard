#import bevy_ui::ui_vertex_output::UiVertexOutput

struct FrostedGlassData {
    tint_color: vec4<f32>,
    /// Noise intensity (0.0 = clean glass, 1.0 = heavy frost).
    frost_intensity: f32,
    /// Scale of the frost noise pattern.
    noise_scale: f32,
    _padding1: f32,
    _padding2: f32,
}

@group(1) @binding(0)
var<uniform> material: FrostedGlassData;

fn hash21(p: vec2<f32>) -> f32 {
    let n = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(n) * 43758.5453);
}

// Smooth value noise with quintic interpolation.
fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);

    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var pos = p;
    for (var i = 0; i < 4; i++) {
        val += amp * value_noise(pos);
        pos *= 2.0;
        amp *= 0.5;
    }
    return val;
}

// Domain-warped noise for organic frost pattern.
fn frost_noise(p: vec2<f32>) -> f32 {
    let warp = vec2<f32>(
        fbm(p + vec2<f32>(0.0, 0.0)),
        fbm(p + vec2<f32>(5.2, 1.3)),
    );
    return fbm(p + warp * 0.3);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let tint = material.tint_color;
    let scale = material.noise_scale;
    let intensity = material.frost_intensity;

    // Two noise layers at different scales for varied frost texture.
    let coarse = frost_noise(uv * scale);
    let fine = frost_noise(uv * scale * 2.5 + vec2<f32>(17.3, 41.7));
    let noise = coarse * 0.6 + fine * 0.4;

    // Strong frost — bright speckled patches over lighter tint.
    let frost = noise * intensity * 0.4;
    var col = tint.rgb + vec3<f32>(frost * 0.75, frost * 0.8, frost * 1.0);

    // Very subtle edge darkening.
    let center = uv - 0.5;
    let dist = length(center * vec2<f32>(1.1, 1.0));
    let vignette = 1.0 - smoothstep(0.4, 0.8, dist) * 0.08;
    col *= vignette;

    return vec4<f32>(col, tint.a);
}
