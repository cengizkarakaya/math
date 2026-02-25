// Cargo.toml
// [dependencies]
// ratatui = "0.26"
// crossterm = "0.27"
//
// t01 – Ratatui TUI Simülasyonu (API uyumlu)
// - Frame::area() kullanır (size() yok)
// - Canvas::print sadece (x,y,text) alır (Style yok)
// - f64 tipleri net
// - ToF (2D radar), IMU, INA226, hareketli sinüs paneli
// - Kontroller: q/Esc çıkış | Space pause | r reset | ← → sinüs fazı

use std::{
    error::Error,
    f64::consts::{FRAC_PI_2, TAU},
    io,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::{Line as TLine, Span},
    widgets::{
        Block, Borders, Gauge, Paragraph,
        canvas::{Canvas, Line},
    },
};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

#[derive(Clone)]
struct SimState {
    t0: Instant,
    paused: bool,

    // Sine
    phase: f64,

    // ToF scan
    scan_angle: f64, // deg
    scan_dir: f64,

    // IMU
    roll: f64,
    pitch: f64,
    yaw: f64,

    // INA226
    v: f64,
    a: f64,
    w: f64,

    // History
    v_hist: Vec<f64>,
    a_hist: Vec<f64>,
    w_hist: Vec<f64>,
}

impl SimState {
    fn new() -> Self {
        Self {
            t0: Instant::now(),
            paused: false,
            phase: 0.0,
            scan_angle: 0.0,
            scan_dir: 1.0,
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            v: 12.2,
            a: 0.8,
            w: 9.8,
            v_hist: vec![12.2; 40],
            a_hist: vec![0.8; 40],
            w_hist: vec![9.8; 40],
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    fn step(&mut self, dt: f64) {
        if self.paused {
            return;
        }

        let t = self.t0.elapsed().as_secs_f64();

        // ToF scan (-60..+60)
        let max_deg = 60.0;
        self.scan_angle += self.scan_dir * 90.0 * dt;
        if self.scan_angle > max_deg {
            self.scan_angle = max_deg;
            self.scan_dir = -1.0;
        } else if self.scan_angle < -max_deg {
            self.scan_angle = -max_deg;
            self.scan_dir = 1.0;
        }

        // IMU
        self.roll = (t * 0.8).sin() * 18.0;
        self.pitch = (t * 0.6 + 0.7).sin() * 12.0;
        self.yaw = (t * 0.25).sin() * 45.0;

        // INA226
        self.v = (12.3 + (t * 0.15).sin() * 0.15).clamp(11.5, 12.8);
        self.a = (0.8 + (t * 1.2).sin().abs() * 4.0).clamp(0.0, 6.0);
        self.w = (self.v * self.a).clamp(0.0, 90.0);

        push_hist(&mut self.v_hist, self.v);
        push_hist(&mut self.a_hist, self.a);
        push_hist(&mut self.w_hist, self.w);

        // Sine phase
        self.phase += 2.6 * dt;
        if self.phase > TAU {
            self.phase -= TAU;
        }
    }
}

fn push_hist(hist: &mut Vec<f64>, val: f64) {
    hist.push(val);
    if hist.len() > 40 {
        let _ = hist.remove(0);
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn Error>> {
    let tick_rate = Duration::from_millis(33);
    let mut last_tick = Instant::now();

    let mut s = SimState::new();

    loop {
        terminal.draw(|f| {
            let root = f.area();

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(root);

            let top = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(rows[0]);

            let bottom = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(rows[1]);

            draw_tof(f, top[0], &s);
            draw_imu(f, top[1], &s);
            draw_ina(f, bottom[0], &s);
            draw_sine(f, bottom[1], &s);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char(' ') => s.paused = !s.paused,
                        KeyCode::Char('r') => s.reset(),
                        KeyCode::Left => s.phase -= 0.25,
                        KeyCode::Right => s.phase += 0.25,
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            s.step(tick_rate.as_secs_f64());
            last_tick = Instant::now();
        }
    }
}

fn draw_tof(f: &mut ratatui::Frame, area: Rect, s: &SimState) {
    let block = Block::default()
        .title("ToF Scan (sim)")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let max_range: f64 = 4.0;
    let ang = s.scan_angle.to_radians();

    let obstacles: [(f64, f64); 5] = [(1.3, 1.1), (2.4, 0.3), (3.1, -0.9), (1.9, -1.7), (0.9, 2.2)];

    let ray_dx = ang.cos();
    let ray_dy = ang.sin();

    let mut meas: f64 = max_range;
    for (ox, oy) in obstacles {
        let dot = ox * ray_dx + oy * ray_dy;
        if dot > 0.0 {
            let proj_x = dot * ray_dx;
            let proj_y = dot * ray_dy;
            let dist = ((ox - proj_x).powi(2) + (oy - proj_y).powi(2)).sqrt();
            if dist < 0.25 {
                meas = meas.min(dot);
            }
        }
    }

    let canvas = Canvas::default()
        .x_bounds([-max_range, max_range])
        .y_bounds([-max_range, max_range])
        .marker(symbols::Marker::Braille)
        .paint(move |ctx| {
            // Range circle
            let steps = 80;
            for i in 0..steps {
                let a1 = TAU * (i as f64) / (steps as f64);
                let a2 = TAU * ((i + 1) as f64) / (steps as f64);
                ctx.draw(&Line {
                    x1: max_range * a1.cos(),
                    y1: max_range * a1.sin(),
                    x2: max_range * a2.cos(),
                    y2: max_range * a2.sin(),
                    color: Color::DarkGray,
                });
            }

            // Obstacles
            for (ox, oy) in obstacles {
                ctx.print(ox, oy, "■");
            }

            // Ray
            ctx.draw(&Line {
                x1: 0.0,
                y1: 0.0,
                x2: meas * ang.cos(),
                y2: meas * ang.sin(),
                color: Color::Cyan,
            });

            ctx.print(meas * ang.cos(), meas * ang.sin(), "●");
            ctx.print(
                -max_range + 0.2,
                max_range - 0.3,
                format!("{:+.0}°  {:.2} m", s.scan_angle, meas),
            );
        });

    f.render_widget(canvas, inner);
}

fn draw_imu(f: &mut ratatui::Frame, area: Rect, s: &SimState) {
    let block = Block::default().title("IMU (sim)").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(inner);

    let text = vec![
        TLine::from(vec![
            Span::raw("roll  "),
            Span::styled(
                format!("{:+5.1}°", s.roll),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        TLine::from(vec![
            Span::raw("pitch "),
            Span::styled(
                format!("{:+5.1}°", s.pitch),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        TLine::from(vec![
            Span::raw("yaw   "),
            Span::styled(format!("{:+5.1}°", s.yaw), Style::default().fg(Color::Cyan)),
        ]),
    ];
    f.render_widget(Paragraph::new(text), chunks[0]);

    let pitch_ratio = ((s.pitch + 30.0) / 60.0).clamp(0.0, 1.0);
    let roll_ratio = ((s.roll + 30.0) / 60.0).clamp(0.0, 1.0);

    let bars = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(chunks[1]);

    f.render_widget(
        Gauge::default()
            .block(Block::default().title("pitch").borders(Borders::TOP))
            .ratio(pitch_ratio),
        bars[0],
    );

    f.render_widget(
        Gauge::default()
            .block(Block::default().title("roll").borders(Borders::TOP))
            .ratio(roll_ratio),
        bars[1],
    );
}

fn draw_ina(f: &mut ratatui::Frame, area: Rect, s: &SimState) {
    let block = Block::default().title("INA226 (sim)").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(inner);

    let text = vec![
        TLine::from(format!("V: {:>5.2} V   A: {:>5.2} A", s.v, s.a)),
        TLine::from(format!(
            "P: {:>6.1} W   {}",
            s.w,
            if s.paused { "PAUSED" } else { "RUN" }
        )),
        TLine::from(""),
        TLine::from("Trend (son 40)"),
    ];
    f.render_widget(Paragraph::new(text), chunks[0]);

    let trend = Paragraph::new(vec![
        TLine::from(format!("V {}", spark(&s.v_hist, 11.5, 12.8))),
        TLine::from(format!("A {}", spark(&s.a_hist, 0.0, 6.0))),
        TLine::from(format!("W {}", spark(&s.w_hist, 0.0, 90.0))),
    ]);
    f.render_widget(trend, chunks[1]);
}

fn spark(vals: &[f64], min: f64, max: f64) -> String {
    let ramp = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let mut out = String::new();
    for &v in vals {
        let n = ((v - min) / (max - min)).clamp(0.0, 1.0);
        let idx = (n * (ramp.len() as f64 - 1.0)).round() as usize;
        out.push(ramp[idx]);
    }
    out
}

fn draw_sine(f: &mut ratatui::Frame, area: Rect, s: &SimState) {
    let block = Block::default().title("Sine (sim)").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let x_min = 0.0;
    let x_max = TAU;
    let y_min = -1.2;
    let y_max = 1.2;

    let samples = 200usize;
    let dx = (x_max - x_min) / (samples as f64 - 1.0);

    let mut lines: Vec<Line> = Vec::with_capacity(samples - 1);
    let mut prev = (x_min, (x_min + s.phase).sin());

    for i in 1..samples {
        let x = x_min + dx * i as f64;
        let y = (x + s.phase).sin();
        lines.push(Line {
            x1: prev.0,
            y1: prev.1,
            x2: x,
            y2: y,
            color: Color::Cyan,
        });
        prev = (x, y);
    }

    let phase = s.phase;

    let canvas = Canvas::default()
        .x_bounds([x_min, x_max])
        .y_bounds([y_min, y_max])
        .marker(symbols::Marker::Braille)
        .paint(move |ctx| {
            ctx.draw(&Line {
                x1: x_min,
                y1: 0.0,
                x2: x_max,
                y2: 0.0,
                color: Color::DarkGray,
            });

            for ln in &lines {
                ctx.draw(ln);
            }

            let px = FRAC_PI_2;
            let py = (px + phase).sin();
            ctx.print(px, py, "●");
            ctx.print(x_min + 0.2, y_max - 0.25, format!("phase {:.2} rad", phase));
        });

    f.render_widget(canvas, inner);
}
