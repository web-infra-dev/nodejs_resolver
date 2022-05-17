use std::time::Instant;

use daachorse::{DoubleArrayAhoCorasickBuilder, MatchKind};
use nodejs_resolver::Resolver;
fn main() {
    let start = Instant::now();
    for _ in 0..100 {
        Resolver::get_target_kind("react");
    }

    for _ in 0..100 {
        Resolver::get_target_kind("testtest");
    }
    println!("{:?}", start.elapsed());

    // target.starts_with(&format!("{c}:\\")) || target.starts_with(&format!("{c}:/"))
    // for item in ('a'..='z').chain(('A'..='Z')) {
    //     println!("\"{}:/\",", item);
    // }

    // let patterns = vec!["tabc"];
    // let pma = DoubleArrayAhoCorasickBuilder::new()
    //     .match_kind(MatchKind::LeftmostLongest)
    //     .build(&patterns)
    //     .unwrap();

    // let mut it = pma.leftmost_find_iter("abc");

    // let m = it.next().unwrap();
    // println!("{:?}", m);
    // assert_eq!((0, 2, 0), (m.start(), m.end(), m.value()));

    // assert_eq!(None, it.next());
}
