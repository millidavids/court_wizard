#import bevy_ui::ui_vertex_output::UiVertexOutput

struct StarSkyData {
    base_color: vec4<f32>,
    time: f32,
    zoom: f32,
    pan_offset: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> material: StarSkyData;

// Hash function for pseudo-random star placement.
fn hash21(p: vec2<f32>) -> f32 {
    let n = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(n) * 43758.5453);
}

fn hash22(p: vec2<f32>) -> vec2<f32> {
    let n = vec2<f32>(dot(p, vec2<f32>(127.1, 311.7)),
                      dot(p, vec2<f32>(269.5, 183.3)));
    return fract(sin(n) * 43758.5453);
}

// Smooth noise for nebula glow.
fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Layered noise for nebula.
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

// Renders a single star layer with given density and brightness.
fn star_layer(uv: vec2<f32>, scale: f32, brightness: f32, time: f32) -> f32 {
    let grid = uv * scale;
    let cell = floor(grid);
    let local = fract(grid) - 0.5;

    var light = 0.0;

    // Check 3x3 neighborhood so stars near cell edges render correctly.
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let neighbor = vec2<f32>(f32(x), f32(y));
            let cell_id = cell + neighbor;
            let rand = hash22(cell_id);

            // Star position jittered within cell.
            let star_pos = neighbor + rand - 0.5 - local;
            let dist = length(star_pos);

            // Twinkle: each star gets a unique phase from its hash.
            let phase = hash21(cell_id * 17.31) * 6.283;
            let speed = 0.8 + hash21(cell_id * 9.17) * 1.4;
            let twinkle = 0.55 + 0.45 * sin(time * speed + phase);

            // Point-star falloff.
            let size = 0.015 + rand.x * 0.02;
            let star = smoothstep(size, 0.0, dist) * brightness * twinkle;
            light += star;
        }
    }

    return light;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let time = material.time;
    let pan = material.pan_offset;
    let zoom = material.zoom;

    // Single parallax offset — entire star field shifts as one flat layer
    // relative to the spell web. Stars are infinitely far away so they all
    // move together, just slower than the foreground nodes.
    let parallax = 0.03;
    let zoom_factor = 1.0 + (zoom - 1.0) * parallax;
    let uv = (in.uv - 0.5) / zoom_factor + 0.5 - pan * parallax;

    // Start with base background color.
    var col = material.base_color.rgb;

    // Faint nebula glow for atmosphere.
    let nebula_uv = uv * 3.0 + vec2<f32>(time * 0.01, time * 0.007);
    let nebula = fbm(nebula_uv);
    let nebula_color = vec3<f32>(0.08, 0.05, 0.15) * nebula * 0.4;
    col += nebula_color;

    // Star layers — small dim background stars + fewer bright stars.
    let stars_bg = star_layer(uv, 40.0, 0.5, time);
    let stars_mid = star_layer(uv + vec2<f32>(13.7, 7.3), 25.0, 0.7, time);
    let stars_bright = star_layer(uv + vec2<f32>(41.3, 29.1), 12.0, 1.0, time);

    // Warm-white tint for stars.
    let star_color = vec3<f32>(0.85, 0.88, 1.0);
    col += star_color * (stars_bg + stars_mid + stars_bright);

    return vec4<f32>(col, material.base_color.a);
}
