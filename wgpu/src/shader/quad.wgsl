struct Globals {
    transform: mat4x4<f32>,
    scale: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;

fn rounded_box_sdf(p: vec2<f32>, size: vec2<f32>, corners: vec4<f32>) -> f32 {
    var box_half = select(corners.yz, corners.xw, p.x > 0.0);
    var corner = select(box_half.y, box_half.x, p.y > 0.0);
    var q = abs(p) - size + corner;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - corner;
}

const PI: f32 = 3.14159265358979;

fn fit_dash_to_length(dash: vec2<f32>, segment_length: f32) -> vec2<f32> {
    let period = dash.x + dash.y;

    if (period <= 0.0 || segment_length <= 0.0) {
        return dash;
    }

    let count = max(round(segment_length / period), 1.0);
    let scale = segment_length / (count * period);

    return dash * scale;
}

fn dotted_corner_alpha(
    d: vec2<f32>,
    half: vec2<f32>,
    radii: vec4<f32>,
    width: f32,
    aa: f32,
) -> f32 {
    let lx = -half.x;
    let rx = half.x;
    let ty = -half.y;
    let by = half.y;
    let radius = width * 0.5;

    var alpha = 0.0;

    if (radii.x == 0.0) {
        alpha = max(alpha, 1.0 - smoothstep(radius - aa, radius + aa, length(d - vec2(lx + radius, ty + radius))));
    }
    if (radii.y == 0.0) {
        alpha = max(alpha, 1.0 - smoothstep(radius - aa, radius + aa, length(d - vec2(rx - radius, ty + radius))));
    }
    if (radii.z == 0.0) {
        alpha = max(alpha, 1.0 - smoothstep(radius - aa, radius + aa, length(d - vec2(rx - radius, by - radius))));
    }
    if (radii.w == 0.0) {
        alpha = max(alpha, 1.0 - smoothstep(radius - aa, radius + aa, length(d - vec2(lx + radius, by - radius))));
    }

    return alpha;
}

fn border_arc_length(d: vec2<f32>, half: vec2<f32>, radii: vec4<f32>) -> f32 {
    let tl = radii.x;
    let tr = radii.y;
    let br = radii.z;
    let bl = radii.w;

    let lx = -half.x;
    let rx = half.x;
    let ty = -half.y;
    let by = half.y;

    let top = 2.0 * half.x - tl - tr;
    let right = 2.0 * half.y - tr - br;
    let bottom = 2.0 * half.x - br - bl;
    let arc_tr = 0.5 * PI * tr;
    let arc_br = 0.5 * PI * br;
    let arc_bl = 0.5 * PI * bl;

    if (tr > 0.0 && d.x > rx - tr && d.y < ty + tr) {
        let v = d - vec2(rx - tr, ty + tr);
        return top + (atan2(v.y, v.x) + 0.5 * PI) * tr;
    }
    if (br > 0.0 && d.x > rx - br && d.y > by - br) {
        let v = d - vec2(rx - br, by - br);
        return top + arc_tr + right + atan2(v.y, v.x) * br;
    }
    if (bl > 0.0 && d.x < lx + bl && d.y > by - bl) {
        let v = d - vec2(lx + bl, by - bl);
        return top + arc_tr + right + arc_br + bottom + (atan2(v.y, v.x) - 0.5 * PI) * bl;
    }
    if (tl > 0.0 && d.x < lx + tl && d.y < ty + tl) {
        let v = d - vec2(lx + tl, ty + tl);
        var t = atan2(v.y, v.x);
        if (t < 0.0) {
            t = t + 2.0 * PI;
        }
        return top + arc_tr + right + arc_br + bottom + arc_bl
            + (2.0 * half.y - bl - tl) + (t - PI) * tl;
    }

    let dl = d.x - lx;
    let dr = rx - d.x;
    let dt = d.y - ty;
    let db = by - d.y;
    let m = min(min(dl, dr), min(dt, db));

    if (m == dt) {
        return d.x - (lx + tl);
    }
    if (m == dr) {
        return top + arc_tr + (d.y - (ty + tr));
    }
    if (m == db) {
        return top + arc_tr + right + arc_br + ((rx - br) - d.x);
    }
    return top + arc_tr + right + arc_br + bottom + arc_bl + ((by - bl) - d.y);
}

fn border_segment_alpha(
    local: f32,
    segment_length: f32,
    style: u32,
    dash: vec2<f32>,
    dist: f32,
    width: f32,
    aa: f32,
    centered: bool,
) -> f32 {
    if (segment_length <= 0.0) {
        return 1.0;
    }

    let fitted = fit_dash_to_length(dash, segment_length);
    let period = max(fitted.x + fitted.y, 0.001);

    if (style == 2u) {
        let phase = local - floor(local / period) * period;
        let along = min(phase, period - phase);
        let across = dist + width * 0.5;
        let radius = width * 0.5;

        return 1.0 - smoothstep(radius - aa, radius + aa, length(vec2(along, across)));
    }

    let offset = select(0.0, fitted.x * 0.5, centered);
    let phase = (local + offset) - floor((local + offset) / period) * period;

    return smoothstep(-aa, aa, phase)
        * (1.0 - smoothstep(fitted.x - aa, fitted.x + aa, phase));
}

// Alpha multiplier for the border at a fragment, masking out the gaps of a
// dashed or dotted border. Returns 1.0 for a solid border (style 0).
// `dist` is the signed distance to the quad edge and `width` the border width.
fn border_dash_alpha(
    d: vec2<f32>,
    half: vec2<f32>,
    radii: vec4<f32>,
    style: u32,
    dash: vec2<f32>,
    dist: f32,
    width: f32,
) -> f32 {
    if (style == 0u) {
        return 1.0;
    }

    let aa = 0.7;
    let tl = radii.x;
    let tr = radii.y;
    let br = radii.z;
    let bl = radii.w;

    let top = 2.0 * half.x - tl - tr;
    let right = 2.0 * half.y - tr - br;
    let bottom = 2.0 * half.x - br - bl;
    let left = 2.0 * half.y - bl - tl;

    let arc_tr = 0.5 * PI * tr;
    let arc_br = 0.5 * PI * br;
    let arc_bl = 0.5 * PI * bl;
    let arc_tl = 0.5 * PI * tl;
    let perimeter = top + arc_tr + right + arc_br + bottom + arc_bl + left + arc_tl;
    let s = border_arc_length(d, half, radii);

    var first = perimeter;
    var last = 0.0;
    var splits = 0u;

    if (tl == 0.0) {
        first = 0.0;
        splits = splits + 1u;
    }
    if (tr == 0.0) {
        first = min(first, top);
        last = max(last, top);
        splits = splits + 1u;
    }
    if (br == 0.0) {
        let b = top + arc_tr + right;
        first = min(first, b);
        last = max(last, b);
        splits = splits + 1u;
    }
    if (bl == 0.0) {
        let b = top + arc_tr + right + arc_br + bottom;
        first = min(first, b);
        last = max(last, b);
        splits = splits + 1u;
    }

    var local = s;
    var segment_length = perimeter;
    var centered = false;

    if (splits > 0u) {
        var previous = last;
        var next = first + perimeter;

        if (s >= first) {
            previous = first;
        }

        if (tl == 0.0) {
            if (s >= 0.0) {
                previous = 0.0;
            }
        }
        if (tr == 0.0) {
            let b = top;
            if (s >= b) {
                previous = b;
            } else if (s >= first && b < next) {
                next = b;
            }
        }
        if (br == 0.0) {
            let b = top + arc_tr + right;
            if (s >= b) {
                previous = b;
            } else if (s >= first && b < next) {
                next = b;
            }
        }
        if (bl == 0.0) {
            let b = top + arc_tr + right + arc_br + bottom;
            if (s >= b) {
                previous = b;
            } else if (s >= first && b < next) {
                next = b;
            }
        }

        local = s - previous;
        if (local < 0.0) {
            local = local + perimeter;
        }

        segment_length = next - previous;
        centered = true;
    }

    var alpha = border_segment_alpha(local, segment_length, style, dash, dist, width, aa, centered);

    if (style == 2u) {
        alpha = max(alpha, dotted_corner_alpha(d, half, radii, width, aa));
    }

    return alpha;
}
