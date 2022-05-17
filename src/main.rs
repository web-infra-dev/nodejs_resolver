use std::time::Instant;

use nodejs_resolver::Resolver;
fn main() {
    let start = Instant::now();
    for _ in 0..100 {
        Resolver::get_target_kind("react");
    }

    // for _ in 0..100 {
    //     Resolver::get_target_kind("testtest");
    // }
    println!("{:?}", start.elapsed());
}
