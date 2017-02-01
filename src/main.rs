extern crate morton;
use morton::Morton;

fn main() {
    let morton32 = Morton::new(4, 8, vec![0; 32]);
}