use std::time::Instant;

use nodejs_resolver::Resolver;
fn main() {
    let start = Instant::now();
    for i in 0..100 {
        Resolver::get_target_kind("react");
    }

    for i in 0..100 {
        Resolver::get_target_kind("fs");
    }
    println!("{:?}", start.elapsed());
}
