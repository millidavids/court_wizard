#import bevy_ui::ui_vertex_output::UiVertexOutput

struct ArcaneRuneData {
    color: vec4<f32>,
    time: f32,
    opacity: f32,
    unlocked_count: f32,
    _padding: f32,
}

@group(1) @binding(0)
var<uniform> material: ArcaneRuneData;

const PI: f32 = 3.14159265358979;
const TAU: f32 = 6.28318530718;

// ── Utility functions ───────────────────────────────────────────────────────

fn rotate2d(p: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(p.x * c - p.y * s, p.x * s + p.y * c);
}

fn glow(d: f32, softness: f32) -> f32 {
    return smoothstep(softness, 0.0, d);
}

fn ring_sdf(dist: f32, radius: f32, thickness: f32) -> f32 {
    return abs(dist - radius) - thickness * 0.5;
}

fn line_segment_sdf(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

fn fmod(x: f32, y: f32) -> f32 {
    return x - floor(x / y) * y;
}

// ── Always-visible: concentric rings ────────────────────────────────────────

fn all_rings(p: vec2<f32>, time: f32) -> f32 {
    var intensity = 0.0;
    let rp_outer = rotate2d(p, time * 0.05);
    let dist_outer = length(rp_outer);
    intensity += glow(ring_sdf(dist_outer, 0.42, 0.0005), 0.0015);
    intensity += glow(ring_sdf(dist_outer, 0.44, 0.0005), 0.001);
    let dist_mid = length(rotate2d(p, -time * 0.08));
    intensity += glow(ring_sdf(dist_mid, 0.32, 0.0005), 0.0015);
    let dist_inner = length(rotate2d(p, time * 0.03));
    intensity += glow(ring_sdf(dist_inner, 0.22, 0.0005), 0.0015);
    let dist_core = length(rotate2d(p, -time * 0.12));
    intensity += glow(ring_sdf(dist_core, 0.06, 0.0005), 0.001);
    return intensity;
}

// ── Middle ring ticks: count = n, from 1 to 31 ─────────────────────────────
// n >= 1

fn middle_ticks(p: vec2<f32>, time: f32, count: f32) -> f32 {
    let rp = rotate2d(p, -time * 0.08);
    let dist = length(rp);
    let angle = atan2(rp.y, rp.x) + PI;
    let norm_tick = angle / TAU * count;
    let tick_frac = fract(norm_tick);
    let tick_dist = min(tick_frac, 1.0 - tick_frac) * TAU / count * dist;
    let tick_radial = smoothstep(0.34, 0.325, dist) * smoothstep(0.295, 0.31, dist);
    return glow(tick_dist, 0.0008) * tick_radial;
}

// ── Outer ring ticks: count = n, from 1 to 31 ──────────────────────────────
// n >= 4

fn outer_ticks(p: vec2<f32>, time: f32, count: f32) -> f32 {
    let rp = rotate2d(p, time * 0.05);
    let dist = length(rp);
    let angle = atan2(rp.y, rp.x) + PI;

    let norm_tick = angle / TAU * count;
    let tick_frac = fract(norm_tick);
    let tick_dist = min(tick_frac, 1.0 - tick_frac) * TAU / count * dist;
    let tick_radial = smoothstep(0.44, 0.425, dist) * smoothstep(0.395, 0.41, dist);

    return glow(tick_dist, 0.0008) * tick_radial;
}

// ── Central polygon: grows from 2 vertices (line) to 5 (pentagram) ─────────
// n=7: 2-line, n=8: triangle, n=9: square, n=10+: pentagram
// n >= 7, sides = clamp(n - 5, 2, 5)

fn central_polygon(p: vec2<f32>, time: f32, sides: i32) -> f32 {
    let rp = rotate2d(p, -time * 0.12);
    var intensity = 0.0;
    let radius = 0.40;

    // For 5 sides: connect every-other vertex (pentagram star)
    // For 2-4: regular polygon (connect adjacent)
    if sides >= 5 {
        for (var i: i32 = 0; i < 5; i++) {
            let a1 = f32(i) / 5.0 * TAU;
            let a2 = f32((i + 2) % 5) / 5.0 * TAU;
            let p1 = vec2<f32>(cos(a1), sin(a1)) * radius;
            let p2 = vec2<f32>(cos(a2), sin(a2)) * radius;
            intensity += glow(line_segment_sdf(rp, p1, p2), 0.001);
        }
    } else {
        for (var i: i32 = 0; i < 4; i++) {
            if i >= sides { break; }
            let a1 = f32(i) / f32(sides) * TAU;
            let a2 = f32((i + 1) % sides) / f32(sides) * TAU;
            let p1 = vec2<f32>(cos(a1), sin(a1)) * radius;
            let p2 = vec2<f32>(cos(a2), sin(a2)) * radius;
            intensity += glow(line_segment_sdf(rp, p1, p2), 0.001);
        }
    }

    return intensity;
}

// ── Metatron's Cube: circles + lines built progressively ────────────────────
// n=10: center circle
// n=11-16: inner circles 1-6 (lines from center as each appears)
// n=17-22: outer circles 1-6 (lines from adjacent inner as each appears)
// n >= 10

fn metatron_cube(p: vec2<f32>, time: f32, n: f32) -> f32 {
    let rp = rotate2d(p, time * 0.02);
    var intensity = 0.0;

    let r_circle = 0.10;
    let inner_r = 0.20;
    let outer_r = 0.40;
    let origin = vec2<f32>(0.0, 0.0);

    let inner_count = clamp(n - 10.0, 0.0, 6.0);
    let outer_count = clamp(n - 16.0, 0.0, 6.0);

    // Center circle (always, since n >= 10)
    intensity += glow(ring_sdf(length(rp), r_circle, 0.0005), 0.0015);

    // Inner ring circles + lines from center
    for (var i: i32 = 0; i < 6; i++) {
        if f32(i) >= inner_count { break; }
        let a = f32(i) / 6.0 * TAU;
        let center = vec2<f32>(cos(a), sin(a)) * inner_r;
        intensity += glow(abs(length(rp - center) - r_circle), 0.0015);
        intensity += glow(line_segment_sdf(rp, origin, center), 0.002);
    }

    // Outer ring circles + lines from center + lines from adjacent inner
    for (var i: i32 = 0; i < 6; i++) {
        if f32(i) >= outer_count { break; }
        let a_out = f32(i) / 6.0 * TAU + PI / 6.0;
        let c_out = vec2<f32>(cos(a_out), sin(a_out)) * outer_r;
        intensity += glow(abs(length(rp - c_out) - r_circle), 0.0015);
        intensity += glow(line_segment_sdf(rp, origin, c_out), 0.002);

        // Lines from the two adjacent inner circles (if they exist)
        let idx1 = i;
        let idx2 = (i + 5) % 6;
        if f32(idx1) < inner_count {
            let a_in1 = f32(idx1) / 6.0 * TAU;
            let c_in1 = vec2<f32>(cos(a_in1), sin(a_in1)) * inner_r;
            intensity += glow(line_segment_sdf(rp, c_in1, c_out), 0.002);
        }
        if f32(idx2) < inner_count {
            let a_in2 = f32(idx2) / 6.0 * TAU;
            let c_in2 = vec2<f32>(cos(a_in2), sin(a_in2)) * inner_r;
            intensity += glow(line_segment_sdf(rp, c_in2, c_out), 0.002);
        }
    }

    return intensity;
}

// ── Runic spiral: arms grow from 1 to 3 ─────────────────────────────────────
// n=17: 1 arm, n=19: 2 arms, n=21+: 3 arms
// n >= 17

fn runic_spiral(p: vec2<f32>, time: f32, arm_count: i32) -> f32 {
    let rp = rotate2d(p, -time * 0.03);
    let dist = length(rp);

    if dist < 0.30 || dist > 0.44 {
        return 0.0;
    }

    let angle = atan2(rp.y, rp.x);
    let r_min = 0.32;
    let r_max = 0.42;
    let turns = 2.5;
    let b = (r_max - r_min) / (turns * TAU);
    let spiral_theta = (dist - r_min) / b;
    let mod_amp = 0.003;
    let mod_freq = 12.0;

    var intensity = 0.0;
    for (var arm: i32 = 0; arm < 3; arm++) {
        if arm >= arm_count { break; }
        let arm_offset = f32(arm) / 3.0 * TAU;
        let diff = angle - spiral_theta - arm_offset;
        let wrapped = diff - round(diff / TAU) * TAU;
        let arc_dist = abs(wrapped) * dist;
        let wobble = mod_amp * sin(mod_freq * spiral_theta);
        let d = abs(arc_dist - wobble);
        intensity += glow(d, 0.001);
    }

    let mask = smoothstep(0.30, 0.32, dist) * smoothstep(0.44, 0.42, dist);
    return intensity * mask;
}

// ── Hypotrochoid spirograph: cusps grow from 3 to 7 ────────────────────────
// n=23: 3 cusps, n=24: 4, ..., n=27+: 7
// n >= 23

fn hypotrochoid(p: vec2<f32>, time: f32, cusps: f32) -> f32 {
    let rp = rotate2d(p, -time * 0.05);
    let dist = length(rp);

    if dist < 0.28 || dist > 0.46 {
        return 0.0;
    }

    let R_big = 0.37;
    let r_small = R_big / cusps;
    let d_pen = r_small * 0.85;
    let diff_r = R_big - r_small;
    let ratio = diff_r / r_small;

    var min_d = 1.0;
    let N = 60;
    var prev = vec2<f32>(diff_r + d_pen, 0.0);
    for (var i: i32 = 1; i <= N; i++) {
        let t = f32(i) / f32(N) * TAU;
        let curr = vec2<f32>(
            diff_r * cos(t) + d_pen * cos(ratio * t),
            diff_r * sin(t) - d_pen * sin(ratio * t),
        );
        min_d = min(min_d, line_segment_sdf(rp, prev, curr));
        prev = curr;
    }

    let mask = smoothstep(0.28, 0.30, dist) * smoothstep(0.46, 0.44, dist);
    return glow(min_d, 0.0015) * mask;
}

// ── Polygon inside a circle: 2=line, 3=tri, 4=sq, 5=star, 6=Star of David ──

fn polygon_at(p: vec2<f32>, center: vec2<f32>, radius: f32, spin: f32, sides: i32) -> f32 {
    let local = rotate2d(p - center, spin);
    var intensity = 0.0;

    if sides == 6 {
        // Star of David: two overlapping triangles
        for (var i: i32 = 0; i < 3; i++) {
            let a1 = f32(i) / 3.0 * TAU;
            let a2 = f32((i + 1) % 3) / 3.0 * TAU;
            let p1 = vec2<f32>(cos(a1), sin(a1)) * radius;
            let p2 = vec2<f32>(cos(a2), sin(a2)) * radius;
            intensity += glow(line_segment_sdf(local, p1, p2), 0.001);
        }
        for (var i: i32 = 0; i < 3; i++) {
            let a1 = f32(i) / 3.0 * TAU + PI / 3.0;
            let a2 = f32((i + 1) % 3) / 3.0 * TAU + PI / 3.0;
            let p1 = vec2<f32>(cos(a1), sin(a1)) * radius;
            let p2 = vec2<f32>(cos(a2), sin(a2)) * radius;
            intensity += glow(line_segment_sdf(local, p1, p2), 0.001);
        }
    } else if sides == 5 {
        // Pentagram: connect every 2nd vertex
        for (var i: i32 = 0; i < 5; i++) {
            let a1 = f32(i) / 5.0 * TAU;
            let a2 = f32((i + 2) % 5) / 5.0 * TAU;
            let p1 = vec2<f32>(cos(a1), sin(a1)) * radius;
            let p2 = vec2<f32>(cos(a2), sin(a2)) * radius;
            intensity += glow(line_segment_sdf(local, p1, p2), 0.001);
        }
    } else {
        // Regular polygon: 2, 3, or 4 sides
        for (var i: i32 = 0; i < 4; i++) {
            if i >= sides { break; }
            let a1 = f32(i) / f32(sides) * TAU;
            let a2 = f32((i + 1) % sides) / f32(sides) * TAU;
            let p1 = vec2<f32>(cos(a1), sin(a1)) * radius;
            let p2 = vec2<f32>(cos(a2), sin(a2)) * radius;
            intensity += glow(line_segment_sdf(local, p1, p2), 0.001);
        }
    }

    return intensity;
}

// ── Stars inside Metatron circles: polygon sides grow 2→6 ───────────────────
// n=22: 2-line, n=23: triangle, n=24: square, n=25: pentagram, n=26+: Star of David
// Only draws inside circles that exist in the cube.
// n >= 22

fn metatron_stars(p: vec2<f32>, time: f32, n: f32) -> f32 {
    let rp = rotate2d(p, time * 0.02);
    var intensity = 0.0;

    let r_circle = 0.10;
    let star_r = r_circle * 0.95;
    let inner_r = 0.20;
    let outer_r = 0.40;
    let cw_speed = time * 0.12;
    let ccw_speed = -time * 0.12;

    let sides = i32(clamp(n - 20.0, 2.0, 6.0));
    let inner_count = clamp(n - 10.0, 0.0, 6.0);
    let outer_count = clamp(n - 16.0, 0.0, 6.0);

    // Center circle: CCW
    intensity += polygon_at(rp, vec2<f32>(0.0, 0.0), star_r, ccw_speed, sides);

    // Inner circles: CW (only in circles that exist)
    for (var i: i32 = 0; i < 6; i++) {
        if f32(i) >= inner_count { break; }
        let a = f32(i) / 6.0 * TAU;
        let center = vec2<f32>(cos(a), sin(a)) * inner_r;
        intensity += polygon_at(rp, center, star_r, cw_speed, sides);
    }

    // Outer circles: CCW (only in circles that exist)
    for (var i: i32 = 0; i < 6; i++) {
        if f32(i) >= outer_count { break; }
        let a = f32(i) / 6.0 * TAU + PI / 6.0;
        let center = vec2<f32>(cos(a), sin(a)) * outer_r;
        intensity += polygon_at(rp, center, star_r, ccw_speed, sides);
    }

    return intensity;
}

// ── Fragment main ──────────────────────────────────────────────────────────
//
// Every spell unlock (n=1 to 31) changes at least one visible element.
//
// n=1-3:   middle ticks grow (1→3)
// n=4-6:   outer ticks appear + middle continues (4→6, outer 1→3)
// n=7:     central polygon: 2-point line
// n=8:     central polygon: triangle
// n=9:     central polygon: square
// n=10:    central polygon: pentagram + Metatron center circle
// n=11-16: Metatron inner circles appear one by one (+ lines from center)
// n=17:    runic spiral arm 1 + Metatron outer circle 1
// n=18:    Metatron outer circle 2
// n=19:    runic spiral arm 2 + Metatron outer circle 3
// n=20:    Metatron outer circle 4
// n=21:    runic spiral arm 3 + Metatron outer circle 5
// n=22:    Metatron outer circle 6 + inner stars: 2-point lines
// n=23:    hypotrochoid 3 cusps + inner stars: triangles
// n=24:    hypotrochoid 4 cusps + inner stars: squares
// n=25:    hypotrochoid 5 cusps + inner stars: pentagrams
// n=26:    hypotrochoid 6 cusps + inner stars: Stars of David
// n=27:    hypotrochoid 7 cusps
// n=28-31: ticks continue growing to 31

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let aspect = in.size.x / max(in.size.y, 1.0);
    var uv = in.uv - 0.5;
    uv.x *= aspect;

    let time = material.time;
    let n = material.unlocked_count;

    // Concentric rings always visible
    var intensity = all_rings(uv, time);

    // Middle ticks: count = n (n >= 1)
    if n >= 1.0 {
        intensity += middle_ticks(uv, time, n);
    }

    // Outer ticks: count = n (n >= 4)
    if n >= 4.0 {
        intensity += outer_ticks(uv, time, n);
    }

    // Central polygon: sides grow 2→5 (n >= 7)
    if n >= 7.0 {
        let sides = i32(clamp(n - 5.0, 2.0, 5.0));
        intensity += central_polygon(uv, time, sides);
    }

    // Metatron's Cube: circles + lines built progressively (n >= 10)
    if n >= 10.0 {
        intensity += metatron_cube(uv, time, n);
    }

    // Runic spiral: arms 1→3 (n >= 17)
    if n >= 17.0 {
        let arms = i32(clamp(floor((n - 15.0) / 2.0), 1.0, 3.0));
        intensity += runic_spiral(uv, time, arms);
    }

    // Stars inside Metatron circles: polygon sides 2→6 (n >= 22)
    if n >= 22.0 {
        intensity += metatron_stars(uv, time, n);
    }

    // Hypotrochoid spirograph: cusps 3→7 (n >= 23)
    if n >= 23.0 {
        let cusps = clamp(n - 20.0, 3.0, 7.0);
        intensity += hypotrochoid(uv, time, cusps);
    }

    intensity = clamp(intensity, 0.0, 1.0);

    let col = material.color.rgb * intensity;
    let alpha = intensity * material.opacity;

    return vec4<f32>(col, alpha);
}
