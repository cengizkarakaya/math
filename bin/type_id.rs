use std::any::{Any, TypeId};
fn main() {
    println!("\n\n\n");

    fn print_type<T: Any>(value: &T) {
        match value.type_id() {
            id if id == TypeId::of::<i32>() => println!("Bu bir i32!"),
            id if id == TypeId::of::<f32>() => println!("Bu bir f32!"),
            _ => println!("Bilinmeyen tür!"),
        }
    }

    let x = 10u16;
    let y = 10.5f32;

    print_type(&x); // Bu bir i32!
    print_type(&y); // Bu bir f32!


    println!("\n\n\n")
}
