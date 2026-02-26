use lib::{alt_cizgi, ust_cizgi};

fn main() {
    ust_cizgi();

    fn ucgen_alani(x: f64) -> f64 {
        (3.0f64).sqrt() / 4.0 * x * x
    }

    let sonuc = ucgen_alani(5.0);

    println!("{:.3}", sonuc);

    alt_cizgi();
}
