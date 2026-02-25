use rand::RngExt;
use std::io::{self, Write};
use std::{thread, time::Duration};

// -------- median --------
fn median(mut buf: [f32; 10]) -> f32 {
    buf.sort_by(|a, b| a.partial_cmp(b).unwrap());
    buf[buf.len() / 2]
}

// -------- moving average --------
fn average(buf: &[f32]) -> f32 {
    buf.iter().sum::<f32>() / buf.len() as f32
}

fn main() {
    let mut ema: f32 = 25.0;
    let alpha: f32 = 0.1;

    let mut temp_buffer: [f32; 10] = [25.0; 10];
    let mut index = 0;

    // rand 0.10 RNG
    let mut rng = rand::rng();

    loop {
        // ---- sahte sensör verisi ----
        // temel sıcaklık + küçük jitter + nadir spike
        let mut raw: f32 = rng.random_range(24.5..=25.5);
        if rng.random_range(0..100) == 0 {
            raw += rng.random_range(2.0..=4.0);
        }

        // ring buffer yazma
        temp_buffer[index] = raw;
        index = (index + 1) % temp_buffer.len();

        // 1️⃣ median filtre
        let med = median(temp_buffer);

        // 2️⃣ EMA filtre
        ema = alpha * med + (1.0 - alpha) * ema;

        // 3️⃣ rapor ortalaması
        let avg = average(&temp_buffer);

        print!(
            "raw:{:>6.2} | median:{:>6.2} | ema:{:>6.2} | avg:{:>6.2}\r",
            raw, med, ema, avg
        );
        io::stdout().flush().unwrap();

        thread::sleep(Duration::from_millis(500));
    }
}
