use lib::*;
use rayon::prelude::*;
use std::{process::Command, time::Instant};
use sysinfo::{Disks, Networks, System};

// ============================================================
// YARDIMCILAR
// ============================================================

fn kib_to_gib(kib: u64) -> f64 {
    kib as f64 / 1024.0 / 1024.0
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;

    if b >= TB {
        format!("{:.2} TB", b / TB)
    } else if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else {
        format!("{:.2} KB", b / KB)
    }
}

fn clean(opt: Option<String>) -> String {
    opt.unwrap_or_else(|| "bilinmiyor".into())
}

// ============================================================
// WINDOWS 11 ALGILAMA (GERÇEK YÖNTEM)
// ============================================================

fn windows_name() -> String {
    let name = System::name().unwrap_or_else(|| "Windows".into());
    let kernel = System::kernel_version().unwrap_or_default();

    if let Ok(build) = kernel.parse::<u32>() {
        if build >= 22000 {
            return format!("Windows 11 (build {})", build);
        }
    }

    format!("{} (build {})", name, kernel)
}

// ============================================================
// RAM HIZI (Windows 11 GERÇEK)
// ============================================================

#[cfg(target_os = "windows")]
fn ram_speed() -> Option<String> {
    let out = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-CimInstance Win32_PhysicalMemory | Select-Object -ExpandProperty Speed",
        ])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&out.stdout);

    for line in text.lines() {
        let t = line.trim();
        if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
            return Some(format!("{} MHz", t));
        }
    }

    None
}

#[cfg(not(target_os = "windows"))]
fn ram_speed() -> Option<String> {
    None
}

// ============================================================
// GPU (DOĞRU DAVRANIŞ)
// ============================================================

fn print_gpu() {
    println!("{}GPU{}", BOLD, RESET);

    let instance = wgpu::Instance::default();

    let adapter = pollster::block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        },
    ));

    let adapter = match adapter {
        Ok(a) => a,
        Err(_) => {
            println!("  {}GPU algılanamadı{}", RED, RESET);
            return;
        }
    };

    let info = adapter.get_info();

    println!("  {}Model{}     : {}", CYAN, RESET, info.name);
    println!("  {}Backend{}   : {:?}", CYAN, RESET, info.backend);
    println!("  {}Type{}      : {:?}", CYAN, RESET, info.device_type);
    println!("  {}VRAM{}      : üretici API gerektirir", CYAN, RESET);
}

// ============================================================
// CPU PRIME BENCH (RAYON)
// ============================================================

fn simple_sieve(limit: usize) -> Vec<usize> {
    let mut p = vec![true; limit + 1];
    p[0] = false;
    p[1] = false;

    let s = (limit as f64).sqrt() as usize;

    for i in 2..=s {
        if p[i] {
            for j in (i * i..=limit).step_by(i) {
                p[j] = false;
            }
        }
    }

    p.iter().enumerate().filter_map(|(i, &v)| v.then_some(i)).collect()
}

fn prime_bench(max: usize) -> usize {
    let base = simple_sieve((max as f64).sqrt() as usize + 1);
    let seg = 1_000_000;

    (0..=max)
        .step_by(seg)
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|start| {
            let end = (start + seg).min(max + 1);
            let mut is_prime = vec![true; end - start];

            for &p in &base {
                let mut first = ((start + p - 1) / p) * p;
                if first < p * p {
                    first = p * p;
                }
                for j in (first..end).step_by(p) {
                    if j >= 2 {
                        is_prime[j - start] = false;
                    }
                }
            }

            is_prime.iter().filter(|&&v| v).count()
        })
        .sum()
}

// ============================================================
// MAIN
// ============================================================

fn main() {
    ust_cizgi();
    let mut sys = System::new_all();
    sys.refresh_all();

    println!("{}SYSTEM BENCHMARK{}", BOLD, RESET);

    #[cfg(target_os = "windows")]
    println!("{}OS{}        : {}", BLUE, RESET, windows_name());

    #[cfg(not(target_os = "windows"))]
    println!("{}OS{}        : {}", BLUE, RESET, clean(System::name()));

    println!("{}Host{}      : {}", BLUE, RESET, clean(System::host_name()));

    println!("\n{}CPU{}", BOLD, RESET);
    println!(
        "  Cores       : {} / {}",
        num_cpus::get(),
        num_cpus::get_physical()
    );

    if let Some(cpu) = sys.cpus().first() {
        println!("  Model       : {}", cpu.brand());
        println!("  Frequency   : {} MHz", cpu.frequency());
    }

    println!("\n{}RAM{}", BOLD, RESET);
    println!(
        "  Total       : {:.2} GB",
        kib_to_gib(sys.total_memory())
    );
    println!(
        "  Available   : {:.2} GB",
        kib_to_gib(sys.available_memory())
    );

    match ram_speed() {
        Some(v) => println!("  Speed       : {}", v),
        None => println!("  Speed       : bilinmiyor"),
    }

    println!("\n{}DISK{}", BOLD, RESET);
    for d in Disks::new_with_refreshed_list().list() {
        println!("  {}{}{}", PURPLE, d.name().to_string_lossy(), RESET);
        println!("    Total     : {}", format_bytes(d.total_space()));
    }

    println!("\n{}NETWORK{}", BOLD, RESET);
    for (name, data) in Networks::new_with_refreshed_list().list() {
        println!(
            "  {}{}{} RX={} TX={}",
            CYAN,
            name,
            RESET,
            format_bytes(data.total_received()),
            format_bytes(data.total_transmitted())
        );
    }

    println!();
    print_gpu();

    println!("\n{}CPU PRIME BENCH{}", BOLD, RESET);

    let max = 10_000_000;
    let t0 = Instant::now();
    let count = prime_bench(max);
    let dt = t0.elapsed();

    println!("  Limit        : {}", max);
    println!("  Prime count  : {}", count);
    println!("  Duration     : {:.2?}", dt);
    println!(
        "  Throughput   : {:.0} prime/s",
        count as f64 / dt.as_secs_f64()
    );

    alt_cizgi();
}
