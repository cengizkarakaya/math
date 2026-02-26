use lib::{alt_cizgi, ust_cizgi};

fn main() {
    ust_cizgi();

    trait UcgenAlan {
        fn ucgen_alani(&self) -> f64;
    }

    //#[derive(Debug)]
    struct Ucgen {
        kenar: f64,
    }

    impl UcgenAlan for Ucgen {
        fn ucgen_alani(&self) -> f64 {
            (3.0f64).sqrt() / 4.0 * self.kenar * self.kenar
        }
    }

    let x = Ucgen { kenar: 5.0 };

    let sonuc = x.ucgen_alani();

    println!(
        "Eşkenar üçgen kenarı {:?} için:\nÜçgen alanı = {:.2}",
        x.kenar, sonuc
    );

    alt_cizgi();
}
