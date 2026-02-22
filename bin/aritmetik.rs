
fn main() {
let x:u8  = 5; let y: u32 = 3;
    let sonuc = x.pow(y); // taban integer, üs u32
    println!("{}", sonuc);
    
    let x: f32  = 5.; let y: f32 = 3.;
    let sonuc = x.powf(y); // taban ve üs float
    println!("{}", sonuc);

    let x: f32  = 5.; let y: i32 = 3;
    let sonuc = x.powi(y); // taban float, üs i32
    println!("{}", sonuc);
}