// src/bin/sinus.rs
use plotly::layout::{Frame, Layout};
use plotly::plot::Traces;
use plotly::{Plot, Surface};
use serde_json::{json, Value};
use std::f64::consts::PI;
use std::fs;
use std::path::{Path, PathBuf};

fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    assert!(n >= 2);
    let step = (end - start) / (n as f64 - 1.0);
    (0..n).map(|i| start + step * i as f64).collect()
}

fn z_matrix(x: &[f64], y: &[f64], phase: f64) -> Vec<Vec<f64>> {
    y.iter()
        .map(|&yy| x.iter().map(|&xx| (xx + phase).sin() * yy.cos()).collect())
        .collect()
}

fn build_plot(grid_n: usize, frames_n: usize) -> Plot {
    let x = linspace(-2.0 * PI, 2.0 * PI, grid_n);
    let y = linspace(-2.0 * PI, 2.0 * PI, grid_n);

    let mut plot = Plot::new();

    let surface0 = Surface::new(z_matrix(&x, &y, 0.0))
        .x(x.clone())
        .y(y.clone())
        .name("z = sin(x+t) * cos(y)");

    plot.add_trace(surface0);

    plot.set_layout(
        Layout::new()
            .title("3D Sinüs Animasyonu")
            .show_legend(false),
    );

    let phase_step = 2.0 * PI / frames_n as f64;

    let mut frames: Vec<Frame> = Vec::with_capacity(frames_n);
    for i in 0..frames_n {
        let phase = i as f64 * phase_step;

        let trace = Surface::new(z_matrix(&x, &y, phase))
            .x(x.clone())
            .y(y.clone())
            .name("z = sin(x+t) * cos(y)");

        let mut traces = Traces::new();
        traces.push(trace); // <-- vec! değil, push()

        frames.push(
            Frame::new()
                .name(format!("f{i}"))
                .traces(vec![0]) // data[0] güncellensin
                .data(traces),
        );
    }

    plot.add_frames(&frames);
    plot
}

fn write_fullscreen_autoplay_html(plot: &Plot, out_html: &Path) -> std::io::Result<()> {
    let offline_js = Plot::offline_js_sources();

    let mut v: Value = serde_json::to_value(plot).expect("plot serialize failed");

    v["layout"]["updatemenus"] = json!([
        {
            "type": "buttons",
            "direction": "left",
            "x": 0.02,
            "y": 0.98,
            "xanchor": "left",
            "yanchor": "top",
            "pad": {"r": 10, "t": 10},
            "showactive": true,
            "buttons": [
                {
                    "label": "▶ Play",
                    "method": "animate",
                    "args": [null, {
                        "mode": "immediate",
                        "fromcurrent": true,
                        "transition": {"duration": 0},
                        "frame": {"duration": 33, "redraw": true}
                    }]
                },
                {
                    "label": "⏸ Pause",
                    "method": "animate",
                    "args": [[null], {
                        "mode": "immediate",
                        "transition": {"duration": 0},
                        "frame": {"duration": 0, "redraw": false}
                    }]
                }
            ]
        }
    ]);

    v["layout"]["margin"] = json!({"l": 0, "r": 0, "t": 50, "b": 0});
    v["layout"]["autosize"] = json!(true);
    v["layout"]["scene"] = json!({"aspectmode": "cube"});

    let fig_json = serde_json::to_string(&v).expect("plot stringify failed");

    let html = format!(
        r#"<!doctype html>
<html lang="tr">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>3D Sinüs Animasyonu</title>
  <style>
    html, body {{ margin:0; padding:0; width:100%; height:100%; overflow:hidden; background:#111; }}
    #plot {{ width:100vw; height:100vh; }}
  </style>
  {offline_js}
</head>
<body>
  <div id="plot"></div>
  <script>
    const fig = {fig_json};
    const data = fig.data || [];
    const layout = fig.layout || {{}};
    const frames = fig.frames || [];
    const config = Object.assign({{ responsive: true }}, fig.configuration || {{}});

    Plotly.newPlot('plot', data, layout, config).then(() => {{
      if (frames.length) Plotly.addFrames('plot', frames);
      Plotly.animate('plot', null, {{
        mode: 'immediate',
        fromcurrent: true,
        transition: {{ duration: 0 }},
        frame: {{ duration: 33, redraw: true }}
      }});
    }});

    window.addEventListener('resize', () => Plotly.Plots.resize('plot'));
  </script>
</body>
</html>
"#,
    );

    fs::write(out_html, html)
}

fn open_in_default_browser(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

fn main() -> std::io::Result<()> {
    let plot = build_plot(70, 180);

    let out_html: PathBuf = std::env::current_dir()?.join("sinus_anim.html");
    write_fullscreen_autoplay_html(&plot, &out_html)?;

    open_in_default_browser(&out_html);
    println!("OK: {}", out_html.display());
    Ok(())
}
