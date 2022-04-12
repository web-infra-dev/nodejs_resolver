use std::{env::current_dir, path::PathBuf};

use node_resolve::Resolver;

#[test]
fn test() {
    let extensions_cases_path = current_dir().unwrap().join("tests/fixtures/extensions");
    let resolver = Resolver::new(extensions_cases_path.clone());
    let target = PathBuf::from(format!("{}/a.js", extensions_cases_path.to_str().unwrap().to_string())) ;
    assert_eq!(resolver.resolve("./a").unwrap(), target);
}
