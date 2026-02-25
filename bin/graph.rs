use std::error::Error;
use std::fs;

use plotly::common::{Font, Line, Mode, Title};
use plotly::configuration::Configuration;
use plotly::layout::{Axis, AspectMode, Camera, CameraCenter, Eye, LayoutScene, Margin, Up};
use plotly::{Layout, Plot, Scatter3D};
use plotters::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {
    // -------------------------
    // 1) Lorenz (RK4)
    // -------------------------
    let sigma = 10.0_f64;
    let rho = 28.0_f64;
    let beta = 8.0_f64 / 3.0_f64;

    let mut x = 0.0_f64;
    let mut y = 1.0_f64;
    let mut z = 1.05_f64;

    let dt = 0.01_f64;
    let steps: usize = 25_000;

    let mut xs = Vec::with_capacity(steps);
    let mut ys = Vec::with_capacity(steps);
    let mut zs = Vec::with_capacity(steps);

    for _ in 0..steps {
        xs.push(x);
        ys.push(y);
        zs.push(z);

        let (k1x, k1y, k1z) = lorenz(x, y, z, sigma, rho, beta);
        let (k2x, k2y, k2z) = lorenz(
            x + 0.5 * dt * k1x,
            y + 0.5 * dt * k1y,
            z + 0.5 * dt * k1z,
            sigma,
            rho,
            beta,
        );
        let (k3x, k3y, k3z) = lorenz(
            x + 0.5 * dt * k2x,
            y + 0.5 * dt * k2y,
            z + 0.5 * dt * k2z,
            sigma,
            rho,
            beta,
        );
        let (k4x, k4y, k4z) =
            lorenz(x + dt * k3x, y + dt * k3y, z + dt * k3z, sigma, rho, beta);

        x += dt * (k1x + 2.0 * k2x + 2.0 * k3x + k4x) / 6.0;
        y += dt * (k1y + 2.0 * k2y + 2.0 * k3y + k4y) / 6.0;
        z += dt * (k1z + 2.0 * k2z + 2.0 * k3z + k4z) / 6.0;
    }

    let burn_in = 2_000usize.min(xs.len());
    let (xs, ys, zs) = (&xs[burn_in..], &ys[burn_in..], &zs[burn_in..]);

    // -------------------------
    // 2) Plotters PNG (dark, plot neon değil; yumuşak “glow”)
    // -------------------------
    let out_png = "lorenz_xz_dark.png";
    write_plotters_png_dark(out_png, xs, zs)?;
    println!("PNG üretildi: {out_png}");

    // -------------------------
    // 3) Plotly 3D HTML
    //    - Eksenler/grid NEON
    //    - Attractor çizgisi NEON değil
    //    - Çok renkli; renk “seviye” (z) ile ton değiştirir
    // -------------------------
    let out_html = "lorenz_3d_dark.html";

    // HTML şişmesin
    let max_points = 18_000usize;
    let stride = (xs.len() / max_points).max(1);
    let (x3, y3, z3) = downsample_xyz(xs, ys, zs, stride);

    let stats = Stats::from_xyz(&x3, &y3, &z3);

    let mut plot = Plot::new();

    // “Seviye” = z normalize (0..1). Segmentleri renklendiriyoruz.
    let (zmin, zmax) = minmax(&z3);
    add_level_colored_segments(
        &mut plot,
        &x3,
        &y3,
        &z3,
        220,      // segment sayısı (performans/kalite)
        2.0,      // çizgi kalınlığı
        zmin,
        zmax,
    );

    let layout = Layout::new()
        .title(Title::with_text("Lorenz Attractor — level-colored (z)"))
        .show_legend(false)
        .auto_size(true)
        .margin(Margin::new().left(0).right(0).bottom(0).top(56))
        .paper_background_color("#070A12")
        .plot_background_color("#070A12")
        .font(Font::new().color("#E8ECF6").size(16))
        .scene(neon_axes_scene_camera());

    plot.set_layout(layout);

    let config = Configuration::new()
        .responsive(true)
        .autosizable(true)
        .fill_frame(false)
        .scroll_zoom(true)
        .display_logo(false);

    plot.set_configuration(config);

    let html = plot.to_html();
    let stats_html = build_stats_html(sigma, rho, beta, dt, steps, burn_in, stride, &stats);
    let html = inject_dark_css_and_stats(html, &stats_html);
    fs::write(out_html, html)?;

    println!("HTML üretildi: {out_html}");
    println!("\nAçmak için (PowerShell):");
    println!("  start .\\{out_html}");
    println!("  start .\\{out_png}");

    Ok(())
}

fn lorenz(x: f64, y: f64, z: f64, sigma: f64, rho: f64, beta: f64) -> (f64, f64, f64) {
    let dx = sigma * (y - x);
    let dy = x * (rho - z) - y;
    let dz = x * y - beta * z;
    (dx, dy, dz)
}

fn write_plotters_png_dark(out_png: &str, xs: &[f64], zs: &[f64]) -> Result<(), Box<dyn Error>> {
    let (min_x, max_x) = minmax(xs);
    let (min_z, max_z) = minmax(zs);

    let root = BitMapBackend::new(out_png, (1400, 900)).into_drawing_area();
    root.fill(&RGBColor(7, 10, 18))?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Lorenz Attractor (x-z projeksiyonu) — Plotters (dark)",
            ("sans-serif", 34),
        )
        .margin(18)
        .x_label_area_size(55)
        .y_label_area_size(60)
        .build_cartesian_2d(min_x..max_x, min_z..max_z)?;

    let axis = RGBColor(232, 236, 246);
    let grid = RGBColor(60, 80, 110).mix(0.25);

    chart
        .configure_mesh()
        .x_desc("x")
        .y_desc("z")
        .label_style(("sans-serif", 18).with_color(axis))
        .axis_desc_style(("sans-serif", 20).with_color(axis))
        .bold_line_style(axis.mix(0.30))
        .light_line_style(grid)
        .draw()?;

    let points = xs.iter().zip(zs.iter()).map(|(&xv, &zv)| (xv, zv));

    // Neon değil: daha “soft” renkler
    chart.draw_series(
        points
            .clone()
            .map(|p| Circle::new(p, 1, RGBColor(120, 140, 255).mix(0.045).filled())),
    )?;
    chart.draw_series(
        points
            .clone()
            .map(|p| Circle::new(p, 1, RGBColor(190, 120, 255).mix(0.055).filled())),
    )?;
    chart.draw_series(points.map(|p| Circle::new(p, 1, RGBColor(240, 245, 255).mix(0.065).filled())))?;

    root.present()?;
    Ok(())
}

fn downsample_xyz(xs: &[f64], ys: &[f64], zs: &[f64], stride: usize) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut x3 = Vec::with_capacity(xs.len() / stride + 1);
    let mut y3 = Vec::with_capacity(ys.len() / stride + 1);
    let mut z3 = Vec::with_capacity(zs.len() / stride + 1);

    for i in (0..xs.len()).step_by(stride) {
        x3.push(xs[i]);
        y3.push(ys[i]);
        z3.push(zs[i]);
    }
    (x3, y3, z3)
}

fn minmax(slice: &[f64]) -> (f64, f64) {
    let mut min_v = f64::INFINITY;
    let mut max_v = f64::NEG_INFINITY;
    for &v in slice {
        min_v = min_v.min(v);
        max_v = max_v.max(v);
    }
    let pad = (max_v - min_v).abs() * 0.03;
    (min_v - pad, max_v + pad)
}

fn neon_axes_scene_camera() -> LayoutScene {
    let camera = Camera::new()
        .center(CameraCenter::new().x(0.0).y(0.0).z(0.0))
        .eye(Eye::new().x(1.9).y(1.9).z(1.2))
        .up(Up::new().x(0.0).y(0.0).z(1.0));

    let tickfont = Font::new().color("#DCE2F2").size(12);
    let titlefont = Font::new().color("#EEF2FF").size(14);

    // Neon eksen stili (sadece eksenler/grid neon)
    let axis = |t: &str| {
        Axis::new()
            .title(Title::with_text(t).font(titlefont.clone()))
            .tick_font(tickfont.clone())
            .grid_color("rgba(0, 225, 255, 0.18)")
            .zero_line_color("rgba(175, 90, 255, 0.34)")
            .line_color("rgba(0, 225, 255, 0.50)")
            .show_grid(true)
    };

    LayoutScene::new()
        .aspect_mode(AspectMode::Data)
        .camera(camera)
        .x_axis(axis("x"))
        .y_axis(axis("y"))
        .z_axis(axis("z"))
}

fn add_level_colored_segments(
    plot: &mut Plot,
    x: &[f64],
    y: &[f64],
    z: &[f64],
    segments: usize,
    width: f64,
    zmin: f64,
    zmax: f64,
) {
    if x.len() < 2 {
        return;
    }

    let n = x.len();
    let segments = segments.max(1);
    let chunk = (n / segments).max(2);

    let mut start = 0usize;
    while start + 1 < n {
        let end = (start + chunk).min(n);

        // segment “seviye” = segment z ortalaması
        let level = segment_level(&z[start..end], zmin, zmax);
        let color = level_color_css(level);

        let trace = Scatter3D::new(x[start..end].to_vec(), y[start..end].to_vec(), z[start..end].to_vec())
            .mode(Mode::Lines)
            .line(Line::new().width(width).color(color));

        plot.add_trace(trace);

        if end == n {
            break;
        }
        start = end - 1;
    }
}

fn segment_level(seg_z: &[f64], zmin: f64, zmax: f64) -> f64 {
    if seg_z.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0;
    for &v in seg_z {
        sum += v;
    }
    let mean = sum / (seg_z.len() as f64);
    normalize(mean, zmin, zmax)
}

fn normalize(v: f64, lo: f64, hi: f64) -> f64 {
    let denom = (hi - lo).abs();
    if denom < 1e-12 {
        return 0.0;
    }
    ((v - lo) / denom).clamp(0.0, 1.0)
}

// Çok renkli ama neon değil: HSV'de ton kaydır (hue) + orta doygunluk
fn level_color_css(level: f64) -> String {
    // Hue: 250° -> 30° (mavi-mor -> turuncu)
    let hue = 250.0 + (30.0 - 250.0) * level;
    let sat = 0.75;   // neon değil
    let val = 0.90;   // canlı ama patlamayan
    let (r, g, b) = hsv_to_rgb(hue, sat, val);
    format!("rgb({r},{g},{b})")
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let h = ((h % 360.0) + 360.0) % 360.0;
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);

    let c = v * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = match h {
        h if (0.0..60.0).contains(&h) => (c, x, 0.0),
        h if (60.0..120.0).contains(&h) => (x, c, 0.0),
        h if (120.0..180.0).contains(&h) => (0.0, c, x),
        h if (180.0..240.0).contains(&h) => (0.0, x, c),
        h if (240.0..300.0).contains(&h) => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (r, g, b)
}

#[derive(Clone, Debug)]
struct Stats {
    n: usize,
    min_x: f64,
    max_x: f64,
    mean_x: f64,
    std_x: f64,
    min_y: f64,
    max_y: f64,
    mean_y: f64,
    std_y: f64,
    min_z: f64,
    max_z: f64,
    mean_z: f64,
    std_z: f64,
    last_x: f64,
    last_y: f64,
    last_z: f64,
}

impl Stats {
    fn from_xyz(x: &[f64], y: &[f64], z: &[f64]) -> Self {
        let (min_x, max_x, mean_x, std_x, last_x) = stats_1d(x);
        let (min_y, max_y, mean_y, std_y, last_y) = stats_1d(y);
        let (min_z, max_z, mean_z, std_z, last_z) = stats_1d(z);

        Self {
            n: x.len(),
            min_x,
            max_x,
            mean_x,
            std_x,
            min_y,
            max_y,
            mean_y,
            std_y,
            min_z,
            max_z,
            mean_z,
            std_z,
            last_x,
            last_y,
            last_z,
        }
    }
}

fn stats_1d(v: &[f64]) -> (f64, f64, f64, f64, f64) {
    let n = v.len().max(1) as f64;

    let mut min_v = f64::INFINITY;
    let mut max_v = f64::NEG_INFINITY;
    let mut sum = 0.0;

    for &x in v {
        min_v = min_v.min(x);
        max_v = max_v.max(x);
        sum += x;
    }
    let mean = sum / n;

    let mut var = 0.0;
    for &x in v {
        let d = x - mean;
        var += d * d;
    }
    let std = (var / n).sqrt();
    let last = *v.last().unwrap_or(&0.0);

    (min_v, max_v, mean, std, last)
}

fn build_stats_html(
    sigma: f64,
    rho: f64,
    beta: f64,
    dt: f64,
    steps: usize,
    burn_in: usize,
    stride: usize,
    s: &Stats,
) -> String {
    let lorenz_calls = steps as u64 * 4;

    format!(
        r#"
<div id="stats-panel">
  <div class="stats-title">Run stats</div>

  <div class="stats-grid">
    <div class="k">σ, ρ, β</div><div class="v">{sigma:.3}, {rho:.3}, {beta:.6}</div>
    <div class="k">dt / steps</div><div class="v">{dt:.4} / {steps}</div>
    <div class="k">burn-in</div><div class="v">{burn_in}</div>
    <div class="k">points used</div><div class="v">{n}</div>
    <div class="k">stride</div><div class="v">{stride}</div>
    <div class="k">lorenz calls</div><div class="v">{lorenz_calls}</div>

    <div class="k">x min..max</div><div class="v">{min_x:.3} .. {max_x:.3}</div>
    <div class="k">x mean ± std</div><div class="v">{mean_x:.3} ± {std_x:.3}</div>

    <div class="k">y min..max</div><div class="v">{min_y:.3} .. {max_y:.3}</div>
    <div class="k">y mean ± std</div><div class="v">{mean_y:.3} ± {std_y:.3}</div>

    <div class="k">z min..max</div><div class="v">{min_z:.3} .. {max_z:.3}</div>
    <div class="k">z mean ± std</div><div class="v">{mean_z:.3} ± {std_z:.3}</div>

    <div class="k">last (x,y,z)</div><div class="v">({last_x:.3}, {last_y:.3}, {last_z:.3})</div>
  </div>

  <div class="hint">
    Wheel=zoom • Drag=rotate • Shift+Drag=pan
  </div>
</div>
"#,
        sigma = sigma,
        rho = rho,
        beta = beta,
        dt = dt,
        steps = steps,
        burn_in = burn_in,
        n = s.n,
        stride = stride,
        lorenz_calls = lorenz_calls,
        min_x = s.min_x,
        max_x = s.max_x,
        mean_x = s.mean_x,
        std_x = s.std_x,
        min_y = s.min_y,
        max_y = s.max_y,
        mean_y = s.mean_y,
        std_y = s.std_y,
        min_z = s.min_z,
        max_z = s.max_z,
        mean_z = s.mean_z,
        std_z = s.std_z,
        last_x = s.last_x,
        last_y = s.last_y,
        last_z = s.last_z,
    )
}

fn inject_dark_css_and_stats(mut html: String, stats_html: &str) -> String {
    let css = r#"
<style>
  :root{
    --bg0:#070A12;
    --fg:#E8ECF6;
    --muted:#AEB8D6;
    --neon:#00E1FF;
  }
  html, body { height: 100%; width: 100%; margin: 0; background: var(--bg0); }
  body{
    display:flex;
    align-items:center;
    justify-content:center;
    background:
      radial-gradient(1200px 800px at 45% 35%, rgba(0,225,255,0.10), rgba(7,10,18,1) 60%),
      radial-gradient(900px 600px at 70% 70%, rgba(175,90,255,0.08), rgba(7,10,18,0) 60%);
    color: var(--fg);
    font-family: ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Arial, "Noto Sans", sans-serif;
  }
  .plotly-graph-div{
    width: 96vw !important;
    height: 92vh !important;
    max-width: 1700px !important;
    max-height: 980px !important;
    min-width: 900px !important;
    min-height: 600px !important;
    border-radius: 18px;
    overflow: hidden;
    box-shadow: 0 18px 55px rgba(0,0,0,0.60);
    outline: 1px solid rgba(0,225,255,0.12);
    position: relative;
  }

  #stats-panel{
    position: fixed;
    top: 18px;
    right: 18px;
    width: 330px;
    padding: 14px 14px 12px;
    border-radius: 14px;
    background: rgba(11,16,32,0.72);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(0,225,255,0.18);
    box-shadow: 0 16px 45px rgba(0,0,0,0.45);
    color: var(--fg);
  }
  .stats-title{
    font-weight: 700;
    letter-spacing: 0.2px;
    margin-bottom: 10px;
    display:flex;
    align-items:center;
    gap:10px;
  }
  .stats-title::before{
    content:"";
    width: 10px;
    height: 10px;
    border-radius: 999px;
    background: var(--neon);
    box-shadow: 0 0 18px rgba(0,225,255,0.55);
    display:inline-block;
  }
  .stats-grid{
    display:grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px 10px;
    font-size: 12.5px;
    line-height: 1.25;
  }
  .stats-grid .k{ color: var(--muted); }
  .stats-grid .v{ color: var(--fg); text-align: right; font-variant-numeric: tabular-nums; }
  .hint{
    margin-top: 10px;
    color: rgba(232,236,246,0.78);
    font-size: 12px;
  }
</style>
"#;

    if let Some(head_idx) = html.find("</head>") {
        html.insert_str(head_idx, css);
    } else {
        html = format!("{css}{html}");
    }

    if let Some(body_idx) = html.find("<body") {
        if let Some(tag_end) = html[body_idx..].find('>') {
            let insert_at = body_idx + tag_end + 1;
            html.insert_str(insert_at, stats_html);
            return html;
        }
    }

    html.push_str(stats_html);
    html
}
