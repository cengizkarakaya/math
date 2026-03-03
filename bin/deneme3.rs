use std::io::{self, Write};

fn main() {

    println!("bir cümle girin:\n");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("girdi okunamadı!");

    let karakter_sayisi = input.chars().count();
    println!("\nkarakter sayısı: {}", karakter_sayisi);

    let input = input.trim_end_matches(&['\r', '\n'][..]);
    let bosluk_sayisi = input.chars().filter(|c| c.is_whitespace()).count();
    println!("Toplam boşluk sayısı: {}", bosluk_sayisi);
    println!("boşluksuz karakter sayısı: {}", karakter_sayisi - bosluk_sayisi);

    let kelime_sayisi = input.split_whitespace().count();
    println!("kelime sayısı: {}", kelime_sayisi);

    let en_uzun = input.split_whitespace().max_by_key(|w| w.chars().count());

    match en_uzun {
        Some(kelime) => println!("En uzun kelime: {}", kelime),
        None => println!("Kelime yok"),
    }

}
