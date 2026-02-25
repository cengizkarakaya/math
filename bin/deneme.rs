use lib::*;
use num::traits::{Float, FloatConst};

// v = π.r^2.h
fn silindir_hacmi<T>(r: T, h: T) -> T
where
    T: Float + FloatConst,
{
    let pi = T::PI();
    pi * r * r * h
}

fn main() {
    ust_cizgi();

    println!("{:.2}", silindir_hacmi(10.2, 5.0));

    alt_cizgi();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silindir_hacmi_birim_deger_f64() {
        let sonuc = silindir_hacmi(1.0_f64, 1.0_f64);
        let fark = (sonuc - std::f64::consts::PI).abs();
        assert!(fark < 1e-12, "beklenen π, bulunan: {sonuc}");
    }

    #[test]
    fn silindir_hacmi_birim_deger_f32() {
        let sonuc = silindir_hacmi(1.0_f32, 1.0_f32);
        let fark = (sonuc - std::f32::consts::PI).abs();
        assert!(fark < 1e-6, "beklenen π, bulunan: {sonuc}");
    }
}
