use lib::{alt_cizgi, ust_cizgi};
use std::io::{self, Write};
fn main() {
    ust_cizgi();

    println!("veri : ");
    io::stdout().flush().unwrap();

    let mut input =String::new();
    io::stdin().read_line(&mut input).unwrap();

    let input = input.trim();
    println!("\ninput : {}", input);

    alt_cizgi();
}
