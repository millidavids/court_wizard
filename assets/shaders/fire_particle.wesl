#import bevy_pbr::forward_io::VertexOutput

struct FireParticleUniforms {
    time: f32,
};

@group(3) @binding(0)
var<uniform> material: FireParticleUniforms;

// --- Hash-based 2D noise (same as ground_noise.wgsl) ---

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

fn fbm3(p: vec2<f32>) -> f32 {
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

// --- Fire color gradient ---
// Maps heat intensity (0 = cold/transparent, 1 = white-hot) through a fire palette.

fn fire_gradient(heat: f32) -> vec3<f32> {
    // 5-stop gradient: black → red → orange → yellow → white
    let c0 = vec3<f32>(0.0, 0.0, 0.0);       // 0.0 - black/transparent
    let c1 = vec3<f32>(0.6, 0.05, 0.0);       // 0.2 - dark red
    let c2 = vec3<f32>(1.0, 0.35, 0.0);       // 0.4 - orange
    let c3 = vec3<f32>(1.0, 0.7, 0.1);        // 0.7 - yellow
    let c4 = vec3<f32>(1.0, 0.95, 0.8);       // 1.0 - white-hot

    if heat < 0.2 {
        return mix(c0, c1, heat / 0.2);
    } else if heat < 0.4 {
        return mix(c1, c2, (heat - 0.2) / 0.2);
    } else if heat < 0.7 {
        return mix(c2, c3, (heat - 0.4) / 0.3);
    } else {
        return mix(c3, c4, (heat - 0.7) / 0.3);
    }
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let t = material.time;

    // Radial distance from center (0 at center, 1 at edges)
    let radial = length(uv - vec2<f32>(0.5, 0.5)) * 2.0;

    // Use world position as a unique seed per particle.
    // Each particle is at a different world position, so they get different noise patterns.
    let world_seed = in.world_position.xz;

    // Primary noise: scrolling upward for rising fire animation
    let noise_uv = world_seed * 0.07 + vec2<f32>(t * 0.3, -t * 1.2);
    let n1 = fbm3(noise_uv);

    // Secondary noise: slower, larger scale turbulence for shape variation
    let n2 = fbm3(world_seed * 0.03 + vec2<f32>(-t * 0.15, -t * 0.4));

    // Combined noise with turbulence
    let noise = n1 * 0.7 + n2 * 0.3;

    // Shape: radial falloff combined with noise for organic fire shapes
    let radial_falloff = 1.0 - pow(radial, 1.3);
    let shape = radial_falloff * noise;

    // Fire heat intensity: boost center, attenuate edges
    let heat = clamp(shape * 1.8, 0.0, 1.0);

    // Color from gradient (hot fire colors)
    let fire_color = fire_gradient(heat);

    // Emissive boost: hot parts glow (simulates additive-like brightness)
    let emissive = fire_color * smoothstep(0.3, 0.8, heat) * 2.0;

    // Dark smoke layer: visible where fire is cool but radial coverage exists.
    // Smoke is dark gray, partially opaque, shown at low heat values.
    let smoke_color = vec3<f32>(0.03, 0.03, 0.03);
    let smoke_strength = smoothstep(0.25, 0.0, heat) * radial_falloff * noise * 0.8;

    // Blend fire over smoke: fire dominates at high heat, smoke shows at low heat
    let fire_strength = smoothstep(0.08, 0.3, heat);
    let blended_color = mix(smoke_color, fire_color + emissive, fire_strength);

    // Alpha: fire gets full alpha, smoke gets moderate alpha, edges dissolve
    let fire_alpha = smoothstep(0.05, 0.2, shape) * 0.7;
    let smoke_alpha = smoke_strength * 0.5;
    let alpha = max(fire_alpha, smoke_alpha) * smoothstep(1.05, 0.8, radial);

    return vec4<f32>(blended_color, alpha);
}
