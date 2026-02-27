// CRT TV post-processing shader
// Applies barrel distortion, scanlines, RGB subpixel grid, vignette,
// chromatic aberration, screen flicker, rounded corners, phosphor glow,
// and channel-change effects (static, tear, rolling bar).
//
// IMPORTANT: All textureSample calls MUST be at the top of the fragment function
// before any non-uniform control flow. Chrome's Tint WGSL validator is strict.

struct FullscreenVertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct CrtSettings {
    barrel_distortion: f32,
    scanline_intensity: f32,
    scanline_count: f32,
    rgb_grid_intensity: f32,
    vignette_intensity: f32,
    vignette_radius: f32,
    enabled: f32,
    chromatic_aberration: f32,
    flicker_intensity: f32,
    corner_radius: f32,
    glow_intensity: f32,
    time: f32,
    channel_change: f32,
    channel_change_time: f32,
    _padding1: f32,
    _padding2: f32,
}
@group(0) @binding(2) var<uniform> settings: CrtSettings;

fn barrel_distort(uv: vec2<f32>, strength: f32) -> vec2<f32> {
    let centered = uv - vec2<f32>(0.5, 0.5);
    let dist_sq = dot(centered, centered);
    let warped = centered * (1.0 + strength * dist_sq);
    return warped + vec2<f32>(0.5, 0.5);
}

/// Integer hash for pseudo-random values in channel-change effects.
fn hash_u(n: u32) -> f32 {
    var x = n;
    x = x ^ (x >> 16u);
    x = x * 0x45d9f3bu;
    x = x ^ (x >> 16u);
    return f32(x & 0xFFFFu) / 65535.0;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // --- All texture samples MUST happen here, before any branches ---

    // 1. Compute barrel-distorted UV and clamp for safe sampling.
    let distorted_uv = barrel_distort(in.uv, settings.barrel_distortion);
    let safe_uv = clamp(distorted_uv, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));

    // 2. Chromatic aberration: offset R and B channels away from center.
    let ca_dir = normalize(safe_uv - vec2<f32>(0.5, 0.5));
    let ca_dist = length(safe_uv - vec2<f32>(0.5, 0.5));
    let ca_offset = ca_dir * ca_dist * settings.chromatic_aberration;
    let ca_uv_r = clamp(safe_uv + ca_offset, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
    let ca_uv_b = clamp(safe_uv - ca_offset, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));

    // 3. Sample: R at offset+, G at center, B at offset-
    let sample_r = textureSample(screen_texture, texture_sampler, ca_uv_r);
    let sample_g = textureSample(screen_texture, texture_sampler, safe_uv);
    let sample_b = textureSample(screen_texture, texture_sampler, ca_uv_b);

    // 4. Sample 4 glow neighbors at cardinal offsets for cheap bloom.
    let texel_size = 4.0 / vec2<f32>(textureDimensions(screen_texture));
    let glow_up    = textureSample(screen_texture, texture_sampler, clamp(safe_uv + vec2<f32>(0.0, texel_size.y), vec2<f32>(0.0), vec2<f32>(1.0)));
    let glow_down  = textureSample(screen_texture, texture_sampler, clamp(safe_uv - vec2<f32>(0.0, texel_size.y), vec2<f32>(0.0), vec2<f32>(1.0)));
    let glow_left  = textureSample(screen_texture, texture_sampler, clamp(safe_uv - vec2<f32>(texel_size.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0)));
    let glow_right = textureSample(screen_texture, texture_sampler, clamp(safe_uv + vec2<f32>(texel_size.x, 0.0), vec2<f32>(0.0), vec2<f32>(1.0)));

    // 5. Pre-sample for channel-change horizontal tear displacement.
    //    Compute displaced UV before branches (8 tear bands with pseudo-random offsets).
    let cc_band = u32(safe_uv.y * 8.0);
    let cc_seed = cc_band * 7u + u32(settings.channel_change_time * 60.0) * 13u;
    let cc_displacement = (hash_u(cc_seed) - 0.5) * 0.08 * settings.channel_change;
    let cc_displaced_uv = clamp(
        vec2<f32>(safe_uv.x + cc_displacement, safe_uv.y),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );
    let cc_sample = textureSample(screen_texture, texture_sampler, cc_displaced_uv);

    // Extra CA burst samples for channel-change effect.
    let cc_ca_extra = ca_dir * ca_dist * settings.channel_change * 0.015;
    let cc_ca_uv_r = clamp(safe_uv + cc_ca_extra, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
    let cc_ca_uv_b = clamp(safe_uv - cc_ca_extra, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));
    let cc_sample_r = textureSample(screen_texture, texture_sampler, cc_ca_uv_r);
    let cc_sample_b = textureSample(screen_texture, texture_sampler, cc_ca_uv_b);

    // --- All samples done. Safe to branch now. ---

    // 6. Early out if disabled.
    if (settings.enabled < 0.5) {
        return sample_g;
    }

    // 7. Barrel bounds masking (soft fade to black outside curved screen edge).
    let inside_x = smoothstep(0.0, 0.01, distorted_uv.x) * smoothstep(0.0, 0.01, 1.0 - distorted_uv.x);
    let inside_y = smoothstep(0.0, 0.01, distorted_uv.y) * smoothstep(0.0, 0.01, 1.0 - distorted_uv.y);
    let inside = inside_x * inside_y;

    // Compose chromatic aberration color.
    var color = vec4<f32>(
        sample_r.r * inside,
        sample_g.g * inside,
        sample_b.b * inside,
        1.0
    );

    // 8. Phosphor glow: average neighbors, extract brightness, additive blend.
    let glow_avg = (glow_up.rgb + glow_down.rgb + glow_left.rgb + glow_right.rgb) * 0.25;
    let glow_brightness = max(glow_avg.r, max(glow_avg.g, glow_avg.b));
    color = vec4<f32>(color.rgb + glow_avg * glow_brightness * settings.glow_intensity * inside, 1.0);

    // 9. Scanlines (use distorted UV so lines follow barrel curvature).
    let screen_dims = vec2<f32>(textureDimensions(screen_texture));
    let scanline_phase = sin(distorted_uv.y * screen_dims.y * 3.14159 * 2.0 / (1080.0 / settings.scanline_count));
    let scanline = 1.0 - settings.scanline_intensity * (0.5 + 0.5 * scanline_phase);
    color = vec4<f32>(color.rgb * scanline, 1.0);

    // 10. RGB subpixel grid (use distorted UV so pixels follow barrel curvature).
    let dim = 1.0 - settings.rgb_grid_intensity;
    let col_f = fract(distorted_uv.x * screen_dims.x / 3.0) * 3.0;
    let is_r = step(col_f, 1.0);
    let is_g = step(1.0, col_f) * step(col_f, 2.0);
    let is_b = step(2.0, col_f);
    let rgb_mask = vec3<f32>(
        is_r * 1.0 + (1.0 - is_r) * dim,
        is_g * 1.0 + (1.0 - is_g) * dim,
        is_b * 1.0 + (1.0 - is_b) * dim
    );
    color = vec4<f32>(color.rgb * rgb_mask, 1.0);

    // 11. Screen flicker: 60Hz-ish brightness oscillation.
    let flicker = 1.0 - settings.flicker_intensity * 0.5 * (1.0 + sin(settings.time * 120.0));
    color = vec4<f32>(color.rgb * flicker, 1.0);

    // 12. Vignette.
    let center_dist = length(in.uv - vec2<f32>(0.5, 0.5));
    let vignette = smoothstep(settings.vignette_radius, settings.vignette_radius - 0.4, center_dist);
    let vignette_factor = mix(1.0, vignette, settings.vignette_intensity);
    color = vec4<f32>(color.rgb * vignette_factor, 1.0);

    // 13. Rounded corners: rounded box SDF on original UV.
    let corner_uv = abs(in.uv - vec2<f32>(0.5, 0.5));
    let corner_q = corner_uv - (vec2<f32>(0.5, 0.5) - vec2<f32>(settings.corner_radius));
    let corner_sdf = length(max(corner_q, vec2<f32>(0.0, 0.0))) - settings.corner_radius;
    let corner_mask = 1.0 - smoothstep(0.0, 0.01, corner_sdf);
    color = vec4<f32>(color.rgb * corner_mask, 1.0);

    // 14. Channel-change effect (static, tear, rolling bar, CA burst, flash).
    if (settings.channel_change > 0.0) {
        let cc = settings.channel_change;

        // A) Horizontal tear: blend in the displaced sample.
        color = vec4<f32>(mix(color.rgb, cc_sample.rgb * inside, cc * 0.7), 1.0);

        // B) Extra chromatic aberration burst on top.
        let cc_ca_color = vec3<f32>(cc_sample_r.r, cc_sample.g, cc_sample_b.b) * inside;
        color = vec4<f32>(mix(color.rgb, cc_ca_color, cc * 0.4), 1.0);

        // C) Rolling bright horizontal bar sweeping down.
        let bar_pos = fract(settings.channel_change_time * 2.5);
        let bar_dist = abs(safe_uv.y - bar_pos);
        let bar = smoothstep(0.05, 0.0, bar_dist) * cc * 0.5;
        color = vec4<f32>(color.rgb + vec3<f32>(bar), 1.0);

        // D) Brief brightness flash (strongest at peak intensity).
        let flash = cc * cc * 0.15;
        color = vec4<f32>(color.rgb + vec3<f32>(flash), 1.0);
    }

    return color;
}
