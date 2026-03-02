use lib::{alt_cizgi, ust_cizgi};
use std::io::{self, Write};

fn main() {
    ust_cizgi();

    println!("bir cümle girin:\n");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("girdi okunamadı!");

    let input =input.trim();
    println!("{}", input);

    let kar_say = input.chars().count();
    println!("{}", kar_say);


    alt_cizgi();
}
