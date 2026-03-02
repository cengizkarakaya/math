use lib::{alt_cizgi, ust_cizgi};
use std::io::{self, Write};

fn main() {
    ust_cizgi();

    println!("bir cümle girin:\n");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("girdi okunamadı!");

    let kar_say = input.chars().count();
    println!("\nkarakter sayısı: {}", kar_say);

    let input = input.trim_end_matches(&['\r', '\n'][..]);
    let bosluk_sayisi = input.chars().filter(|c| c.is_whitespace()).count();
    println!("Toplam boşluk sayısı: {}", bosluk_sayisi);
    println!("boşluksuz karakter sayısı: {}", kar_say - bosluk_sayisi);

    let kelime_sayisi = input.split_whitespace().count();
    println!("kelime sayısı: {}", kelime_sayisi);

    let en_uzun = input.split_whitespace().max_by_key(|w| w.chars().count());

    match en_uzun {
        Some(kelime) => println!("En uzun kelime: {}", kelime),
        None => println!("Kelime yok"),
    }

    alt_cizgi();
}
