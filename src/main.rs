extern crate morton;
use morton::Morton;

fn main() {
    let mut vector = Vec::with_capacity(8 * 4);
    for y in 0..8 {
        for x in 0..4 {
            vector.push((x,y));
        }
    }
    let morton4_1x2 = Morton::new(4, 8, vector);
    println!("{:?}", morton4_1x2);
}