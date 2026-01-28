// Trait Object Dynamic Dispatch in Rust
// A trait method is called on a dyn Trait reference. PTA must resolve
// the vtable to determine the actual implementation called.
//
// Expected finding: indirect call via shape.area() resolves to Circle::area()

trait Shape {
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {                  // resolved target
        std::f64::consts::PI * self.radius * self.radius
    }
}

fn print_area(shape: &dyn Shape) {
    println!("area = {}", shape.area());     // SINK: indirect trait call
}

fn main() {
    let c = Circle { radius: 3.0 };         // SOURCE: Circle vtable assigned
    print_area(&c);
}
