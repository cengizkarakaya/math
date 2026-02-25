use lib::{alt_cizgi, ust_cizgi};
use rayon::prelude::*;
//use std::ffi::OsStr;
use std::{
    fmt,
    time::{Duration, Instant},
};
use sysinfo::{Disks, Networks, System};
#[derive(Debug, Clone)]
struct BenchResult {
    name: &'static str,
    duration: Duration,
    score: f64,
    unit: &'static str,
}

impl fmt::Display for BenchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<22}  süre: {:>10}  skor: {:>12.3} {}",
            self.name,
            humantime::format_duration(self.duration),
            self.score,
            self.unit
        )
    }
}

fn print_system_report() {
    // sysinfo: mümkün olduğunca taşınabilir rapor
    let mut sys = System::new();

    // Daha kaliteli CPU bilgisi
    sys.refresh_all();

    println!("===================== SİSTEM RAPORU =====================");

    // OS bilgileri (bazıları platforma göre boş dönebilir)
    let name = System::name().unwrap_or_else(|| "(bilinmiyor)".into());
    let os_ver = System::os_version().unwrap_or_else(|| "(bilinmiyor)".into());
    let kernel = System::kernel_version().unwrap_or_else(|| "(bilinmiyor)".into());
    let host = System::host_name().unwrap_or_else(|| "(bilinmiyor)".into());

    println!("Host adı          : {}", host);
    println!("OS                : {}", name);
    println!("OS sürümü         : {}", os_ver);
    println!("Kernel            : {}", kernel);

    // CPU bilgileri
    let logical = num_cpus::get();
    let physical = num_cpus::get_physical();
    println!("\n---------------------- CPU ----------------------");
    println!("Logical cores     : {}", logical);
    println!("Physical cores    : {}", physical);

    let cpus = sys.cpus();
    if !cpus.is_empty() {
        // sysinfo her logical CPU için entry döndürebilir; model adı genelde aynı
        println!("CPU model         : {}", cpus[0].brand());
        println!("CPU vendor        : {}", cpus[0].vendor_id());
        // frekans: sysinfo MHz döndürür (platforma göre değişebilir)
        println!("CPU freq (MHz)    : {}", cpus[0].frequency());
    }

    // Opsiyonel CPUID (sadece x86/x86_64 + feature)
    #[cfg(all(feature = "cpuid", any(target_arch = "x86", target_arch = "x86_64")))]
    {
        use raw_cpuid::CpuId;
        let cpuid = CpuId::new();
        println!("\n[cpuid] Ek CPU ayrıntısı:");
        if let Some(fi) = cpuid.get_feature_info() {
            println!("  SSE2            : {}", fi.has_sse2());
            println!("  SSE4.2          : {}", fi.has_sse42());
            println!("  AVX             : {}", fi.has_avx());
            println!("  AVX2            : {}", fi.has_avx2());
            println!("  FMA             : {}", fi.has_fma());
        }
        if let Some(brand) = cpuid.get_processor_brand_string() {
            println!("  Brand string     : {}", brand.as_str());
        }
    }

    // RAM bilgileri
    println!("\n---------------------- RAM ----------------------");
    // sysinfo KiB verir
    let total_mem_kib = sys.total_memory();
    let avail_mem_kib = sys.available_memory();
    let used_mem_kib = total_mem_kib.saturating_sub(avail_mem_kib);
    println!(
        "Toplam RAM        : {:.2} GB",
        total_mem_kib as f64 / 1024.0 / 1024.0
    );
    println!(
        "Kullanılan RAM    : {:.2} GB",
        used_mem_kib as f64 / 1024.0 / 1024.0
    );
    println!(
        "Boş RAM           : {:.2} GB",
        avail_mem_kib as f64 / 1024.0 / 1024.0
    );

    // Diskler
    println!("\n--------------------- DİSK ----------------------");
    let disks = Disks::new_with_refreshed_list();
    if disks.list().is_empty() {
        println!("Disk bilgisi alınamadı.");
    } else {
        for d in disks.list() {
            let name = d.name().to_string_lossy();
            let mount = d.mount_point().to_string_lossy();
            let fs = d.file_system().to_string_lossy();

            let total = d.total_space() as f64;
            let avail = d.available_space() as f64;
            let used = total - avail;

            println!("- {}  mount:{}  fs:{}", name, mount, fs);

            println!(
                "  toplam: {:.2} GB | kullanılan: {:.2} GB | boş: {:.2} GB",
                total / 1e9,
                used / 1e9,
                avail / 1e9
            );
        }
    }

    // Ağ arayüzleri
    println!("\n---------------------- AĞ -----------------------");
    let networks = Networks::new_with_refreshed_list();
    if networks.list().is_empty() {
        println!("Ağ arayüzü bilgisi alınamadı.");
    } else {
        for (name, data) in networks.list() {
            println!(
                "- {}  rx:{} bytes  tx:{} bytes",
                name,
                data.total_received(),
                data.total_transmitted()
            );
        }
    }

    println!("============================================================\n");
}

// =============== BENCHMARK 1: CPU (prime count) ===============

fn simple_sieve(limit: usize) -> Vec<usize> {
    let mut is_prime = vec![true; limit + 1];

    is_prime[0] = false;
    if limit >= 1 {
        is_prime[1] = false;
    }

    let sqrt = (limit as f64).sqrt() as usize;

    for i in 2..=sqrt {
        if is_prime[i] {
            for j in (i * i..=limit).step_by(i) {
                is_prime[j] = false;
            }
        }
    }

    is_prime
        .iter()
        .enumerate()
        .filter_map(|(i, &p)| if p { Some(i) } else { None })
        .collect()
}

fn prime_count_segmented_parallel(max: usize) -> (usize, Vec<usize>) {
    // İlk 10 + son 10 için listelerin tamamını tutmak yerine küçük örnek alacağız.
    // Ama basitlik için: segment sonuçlarından prime’ları toplayıp sıralıyoruz.
    // 10 milyon için bu hala makul (664k prime civarı).

    let limit = (max as f64).sqrt() as usize + 1;
    let base_primes = simple_sieve(limit);

    let segment_size = 1_000_000;
    let segments: Vec<(usize, usize)> = (0..=max)
        .step_by(segment_size)
        .map(|start| {
            let end = (start + segment_size).min(max + 1);
            (start, end)
        })
        .collect();

    let mut primes: Vec<usize> = segments
        .par_iter()
        .flat_map(|&(start, end)| {
            let mut is_prime = vec![true; end - start];

            for &p in &base_primes {
                // p*p taşması riskine karşı (burada max küçük ama yine de güvenli)
                let p2 = p.saturating_mul(p);
                let mut first = if start <= p {
                    p2
                } else {
                    ((start + p - 1) / p) * p
                };

                if first < start {
                    first = start;
                }

                // p2 < start ise ilk çoklama zaten start hizasına çekildi
                for j in (first..end).step_by(p) {
                    if j >= 2 {
                        is_prime[j - start] = false;
                    }
                }

                // p’nin kendisini yanlışlıkla eleme ihtimali: start <= p < end ise is_prime[p-start] false olabilir
                // Bu durum yalnızca first == p olduğunda olur; ama biz first’i p2’den başlatıyoruz (start<=p ise).
                // start>p ise first >= start olduğu için p burada yok.
            }

            (start..end)
                .zip(is_prime)
                .filter_map(|(i, prime)| if prime && i >= 2 { Some(i) } else { None })
                .collect::<Vec<_>>()
        })
        .collect();

    primes.sort_unstable();
    let count = primes.len();
    (count, primes)
}

fn bench_cpu_primes(max: usize) -> (BenchResult, Vec<usize>) {
    let t0 = Instant::now();
    let (count, primes) = prime_count_segmented_parallel(max);
    let dt = t0.elapsed();

    let primes_per_sec = count as f64 / dt.as_secs_f64();

    (
        BenchResult {
            name: "CPU prime sieve",
            duration: dt,
            score: primes_per_sec,
            unit: "prime/s",
        },
        primes,
    )
}

// =============== BENCHMARK 2: Memory bandwidth (copy) ===============

fn bench_memory_copy(megabytes: usize, rounds: usize) -> BenchResult {
    let bytes = megabytes * 1024 * 1024;
    let mut a = vec![0u8; bytes];
    let mut b = vec![0u8; bytes];

    // a’yı doldur (optimizasyonun boş kopyayı elmesin diye)
    for (i, v) in a.iter_mut().enumerate() {
        *v = (i as u8).wrapping_mul(31).wrapping_add(7);
    }

    let t0 = Instant::now();
    for _ in 0..rounds {
        b.copy_from_slice(&a);
        // ufak bir kullanım: compiler “ölü kopya” sanmasın
        a[0] = a[0].wrapping_add(b[bytes - 1]);
    }
    let dt = t0.elapsed();

    let total_bytes = (bytes as f64) * (rounds as f64);
    let gb_per_sec = (total_bytes / 1e9) / dt.as_secs_f64();

    BenchResult {
        name: "Memory copy",
        duration: dt,
        score: gb_per_sec,
        unit: "GB/s",
    }
}

// =============== BENCHMARK 3: Multi-core scalar FLOP-ish (busy loop) ===============

fn bench_fp_busy(seconds: f64) -> BenchResult {
    let logical = num_cpus::get();
    let t0 = Instant::now();

    // Her core’da aynı süreye yakın döngü çalıştır
    let ops: u64 = (0..logical)
        .into_par_iter()
        .map(|core_id| {
            let mut x = 1.000_001_f64 + core_id as f64 * 1e-9;
            let mut iters: u64 = 0;
            loop {
                // Biraz karışık hesap: pipeline’ı doldurur
                x = (x * 1.000_000_3 + 0.000_000_7).sin().abs() + 0.000_001;
                x = (x + 0.000_000_9).sqrt();
                iters += 1;
                if t0.elapsed().as_secs_f64() >= seconds {
                    break;
                }
            }
            // x’i kullan
            if x.is_nan() { 0 } else { iters }
        })
        .sum();

    let dt = t0.elapsed();
    let iters_per_sec = ops as f64 / dt.as_secs_f64();

    BenchResult {
        name: "FP busy loop",
        duration: dt,
        score: iters_per_sec,
        unit: "iter/s",
    }
}

fn main() {
    ust_cizgi();
    // 1) Donanım/Sistem raporu
    print_system_report();

    // 2) Benchmarklar
    println!("===================== BENCHMARK =====================");

    // CPU asal tarama (10 milyon)
    let max = 10_000_000;
    let (cpu_res, primes) = bench_cpu_primes(max);
    println!("{}", cpu_res);

    // İlk 10 / Son 10
    println!("\nPrime örnekleri (1..={}):", max);
    let first10: Vec<_> = primes.iter().take(10).cloned().collect();
    let last10: Vec<_> = primes
        .iter()
        .rev()
        .take(10)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    println!("  İlk 10 : {:?}", first10);
    println!("  Son 10 : {:?}", last10);
    println!("  Toplam : {}", primes.len());

    // Memory copy
    // 512 MB x 8 tur = 4 GB kopya (RAM’e göre artır/azalt)
    let mem_res = bench_memory_copy(512, 8);
    println!("\n{}", mem_res);

    // FP busy loop (3 saniye)
    let fp_res = bench_fp_busy(3.0);
    println!("{}", fp_res);

    // Özet
    println!("\n---------------------- ÖZET ----------------------");
    println!(
        "CPU cores (logical/physical): {}/{}",
        num_cpus::get(),
        num_cpus::get_physical()
    );
    println!("CPU prime sieve score       : {:.0} prime/s", cpu_res.score);
    println!("Memory copy bandwidth       : {:.3} GB/s", mem_res.score);
    println!("FP busy loop throughput     : {:.0} iter/s", fp_res.score);
    println!("===================================================");
    alt_cizgi();
}
