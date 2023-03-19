use std::{env, path::PathBuf};

use nodejs_resolver::{ResolveResult, Resolver};

// cargo run --example simple -- `pwd`/tests/fixtures/simple .
// cargo watch -x 'run --example simple -- `pwd`/tests/fixtures/simple .'

fn main() {
    let path = env::args().nth(1).expect("path");
    let request = env::args().nth(2).expect("request");
    let resolver = Resolver::new(Default::default());
    let path_to_resolve = PathBuf::from(&path);
    match resolver.resolve(&path_to_resolve, &request) {
        Ok(ResolveResult::Info(info)) => println!("{:?}", info.normalized_path()),
        Ok(ResolveResult::Ignored) => println!("Ignored"),
        Err(err) => println!("{err:?}"),
    }
}
