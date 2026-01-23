use std::fs::File;
use std::io::Write;

#[derive(Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    fn max(self, other: f32) -> Vec2 {
        Self {
            x: self.x.max(other),
            y: self.y.max(other),
        }
    }
    fn abs(self) -> Vec2 {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }
}

fn length(v: Vec2) -> f32 {
    v.length()
}

fn sd_rounded_box(p: Vec2, b: Vec2, r: f32) -> f32 {
    let q = p.abs();
    let q_minus_b = Vec2::new(q.x - b.x, q.y - b.y);
    let l = length(q_minus_b.max(0.0));
    let m = (q.x - b.x).max(q.y - b.y).min(0.0);
    l + m - r
}

fn sd_circle(p: Vec2, r: f32) -> f32 {
    length(p) - r
}

fn sd_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let pa = Vec2::new(p.x - a.x, p.y - a.y);
    let ba = Vec2::new(b.x - a.x, b.y - a.y);
    let h = (pa.x * ba.x + pa.y * ba.y) / (ba.x * ba.x + ba.y * ba.y);
    let h_clamped = h.max(0.0).min(1.0);
    let d = Vec2::new(pa.x - ba.x * h_clamped, pa.y - ba.y * h_clamped);
    length(d)
}

fn op_union(d1: f32, d2: f32) -> f32 {
    d1.min(d2)
}

fn op_sub(d1: f32, d2: f32) -> f32 {
    d1.max(-d2)
}

fn scene(p: Vec2) -> (f32, (u8, u8, u8)) {
    let col_bg = (43, 45, 49);
    let col_lock = (34, 211, 238);
    let col_shackle = (200, 200, 200);

    let d_base = sd_rounded_box(Vec2::new(p.x - 0.5, p.y - 0.5), Vec2::new(0.42, 0.42), 0.15);

    let d_body = sd_rounded_box(Vec2::new(p.x - 0.5, p.y - 0.6), Vec2::new(0.25, 0.20), 0.05);

    let p_shackle = Vec2::new(p.x - 0.5, p.y - 0.45);
    let d_circle = sd_circle(p_shackle, 0.14);
    let d_inner = sd_circle(p_shackle, 0.07);
    let d_ring = op_sub(d_circle, d_inner);
    let d_arch = op_sub(d_ring, p.y - 0.45);

    let p_check = Vec2::new(p.x, p.y);
    let d_seg1 = sd_segment(p_check, Vec2::new(0.40, 0.60), Vec2::new(0.48, 0.68));
    let d_seg2 = sd_segment(p_check, Vec2::new(0.48, 0.68), Vec2::new(0.62, 0.50));
    let d_check = op_union(d_seg1, d_seg2) - 0.04;

    let d_main = d_base;
    let d_check_shape = d_check;

    if d_main > 0.0 {
        return (d_main, (0, 0, 0));
    }

    if d_check_shape < 0.0 {
        return (d_check_shape, (16, 185, 129));
    }

    if d_body < 0.0 {
        return (d_body, col_lock);
    }

    if d_arch < 0.0 {
        return (d_arch, col_shackle);
    }

    (d_main, col_bg)
}

fn sample_pixel(x: u32, y: u32, w: u32, h: u32) -> (u8, u8, u8, u8) {
    let samples = 4;
    let step = 1.0 / samples as f32;
    let mut r_acc = 0.0;
    let mut g_acc = 0.0;
    let mut b_acc = 0.0;
    let mut a_acc = 0.0;

    for i in 0..samples {
        for j in 0..samples {
            let u = (x as f32 + (i as f32 + 0.5) * step) / w as f32;
            let v = (y as f32 + (j as f32 + 0.5) * step) / h as f32;

            let (dist, col) = scene(Vec2::new(u, v));
            let alpha = if dist <= 0.0 { 1.0 } else { 0.0 };

            r_acc += col.0 as f32 * alpha;
            g_acc += col.1 as f32 * alpha;
            b_acc += col.2 as f32 * alpha;
            a_acc += alpha;
        }
    }

    let count = (samples * samples) as f32;
    (
        (r_acc / count) as u8,
        (g_acc / count) as u8,
        (b_acc / count) as u8,
        (a_acc / count * 255.0) as u8,
    )
}

fn main() {
    let sizes = [16, 20, 32, 128, 256];

    for &size in &sizes {
        let mut pixels = Vec::with_capacity((size * size * 4) as usize);
        let mut raw_pixels = Vec::with_capacity((size * size * 4) as usize);

        for y in 0..size {
            for x in 0..size {
                let (r, g, b, a) = sample_pixel(x, y, size, size);
                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(a);

                raw_pixels.push(r);
                raw_pixels.push(g);
                raw_pixels.push(b);
                raw_pixels.push(a);
            }
        }

        let path = format!("assets/icon-tray-{}.png", size);
        if size >= 128 {
            let path_large = format!("assets/icon-{}.png", size);
            if let Err(e) =
                lodepng::encode32_file(&path_large, &pixels, size as usize, size as usize)
            {
                eprintln!("Error writing {}: {}", path_large, e);
            }
        } else {
            if let Err(e) = lodepng::encode32_file(&path, &pixels, size as usize, size as usize) {
                eprintln!("Error writing {}: {}", path, e);
            }
        }

        if size == 32 {
            let raw_path = "assets/icon-tray-32.rgba";
            let mut file = File::create(raw_path).unwrap();
            file.write_all(&raw_pixels).unwrap();
        }
    }

    let svg_content = r##"<svg width="256" height="256" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
      <feDropShadow dx="0" dy="2" stdDeviation="2" flood-color="#000" flood-opacity="0.3"/>
    </filter>
  </defs>
  <rect x="8" y="8" width="84" height="84" rx="15" fill="#2B2D31" />
  <path d="M 36 45 L 36 38 A 14 14 0 0 1 64 38 L 64 45" stroke="#C8C8C8" stroke-width="8" fill="none" stroke-linecap="round" />
  <rect x="25" y="45" width="50" height="40" rx="5" fill="#22D3EE" filter="url(#shadow)" />
  <path d="M 40 60 L 48 68 L 62 50" stroke="#10B981" stroke-width="8" fill="none" stroke-linecap="round" stroke-linejoin="round" />
 </svg>"##;

    let mut svg_file = File::create("assets/icon.svg").unwrap();
    svg_file.write_all(svg_content.as_bytes()).unwrap();
}
