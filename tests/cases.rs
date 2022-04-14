use node_resolve::Resolver;
use std::path::PathBuf;

macro_rules! get_cases_path {
    ($path: expr) => {
        std::env::current_dir().unwrap().join($path)
    };
}

macro_rules! fixture {
    ($path: expr) => {
        get_cases_path!("tests/fixtures").join($path)
    };
}

macro_rules! should_equal {
    ($resolver: expr, $resolve_target: expr; $path: expr) => {
        assert_eq!($resolver.resolve($resolve_target), Ok($path));
    };
}
macro_rules! should_error {
    ($resolver: expr, $resolve_target: expr; $expected_err_msg: expr) => {
        assert_eq!(
            $resolver.resolve($resolve_target),
            Err(String::from($expected_err_msg))
        );
    };
}

#[test]
fn extensions_test() {
    let extensions_cases_path = get_cases_path!("tests/fixtures/extensions");
    let mut resolver = Resolver::default()
        .with_extensions(vec!["ts", "js"])
        .with_base_dir(&extensions_cases_path);

    should_equal!(resolver, "./a"; fixture!("extensions/a.ts"));
    should_equal!(resolver, "./a.js"; fixture!("extensions/a.js"));
    should_equal!(resolver, "./dir"; fixture!("extensions/dir/index.ts"));
    should_equal!(resolver, "."; fixture!("extensions/index.js"));
    should_equal!(resolver, "m"; fixture!("extensions/node_modules/m.js"));
    should_equal!(resolver, "m/"; fixture!("extensions/node_modules/m/index.ts"));
    should_equal!(resolver, "module"; PathBuf::from("module"));
    should_error!(resolver, "./a.js/"; "Not found directory");
    should_error!(resolver, "m.js/"; "Not found");
}

#[test]
fn alias_test() {
    let alias_cases_path = get_cases_path!("tests/fixtures/alias");
    let mut resolver = Resolver::default()
        .with_alias(vec![
            ("aliasA", "./a"),
            ("recursive", "./recursive/dir"),
            ("#", "./c/dir"),
            ("@", "./c/dir"),
        ])
        .with_base_dir(&alias_cases_path);

    should_equal!(resolver, "./a"; fixture!("alias/a/index"));
    should_equal!(resolver, "./a/index"; fixture!("alias/a/index"));
    should_equal!(resolver, "./a/dir"; fixture!("alias/a/dir/index"));
    should_equal!(resolver, "./a/dir/index"; fixture!("alias/a/dir/index"));
    should_equal!(resolver, "aliasA"; fixture!("alias/a/index"));
    should_equal!(resolver, "aliasA/index"; fixture!("alias/a/index"));
    should_equal!(resolver, "aliasA/dir"; fixture!("alias/a/dir/index"));
    should_equal!(resolver, "aliasA/dir/index"; fixture!("alias/a/dir/index"));
    should_equal!(resolver, "#"; fixture!("alias/c/dir/index"));
    should_equal!(resolver, "#/index"; fixture!("alias/c/dir/index"));
    should_equal!(resolver, "@"; fixture!("alias/c/dir/index"));
    should_equal!(resolver, "@/index"; fixture!("alias/c/dir/index"));
    should_equal!(resolver, "recursive"; fixture!("alias/recursive/dir/index"));
    should_equal!(resolver, "recursive/index"; fixture!("alias/recursive/dir/index"));
}

#[test]
fn symlink_test() {
    let symlink_cases_path = get_cases_path!("tests/fixtures/symlink");
    let mut resolver = Resolver::default().with_base_dir(&symlink_cases_path.join("linked"));

    should_equal!(resolver, "./index.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./node.relative.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./node.relative.sym.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./this/lib/index.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./this/this/index.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./outer/lib/index.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./outer/linked/index.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./outer/linked/this/index.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./outer/linked/this/lib/index.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./that/lib/index.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./that/outer/lib/index.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./that/outer/linked/lib/index.js"; fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./that/outer/linked/that/lib/index.js"; fixture!("symlink/lib/index.js"));

    resolver.use_base_dir(&symlink_cases_path);
    should_equal!(resolver, "./lib/index.js";  fixture!("symlink/lib/index.js"));

    resolver.use_base_dir(&symlink_cases_path.join("linked/this"));
    should_equal!(resolver, "./lib/index.js";  fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./outer/linked/lib/index.js";  fixture!("symlink/lib/index.js"));

    resolver.use_base_dir(&symlink_cases_path.join("linked/this/lib"));
    should_equal!(resolver, "./index.js";  fixture!("symlink/lib/index.js"));

    resolver.use_base_dir(&symlink_cases_path.join("linked/this/outer/linked"));
    should_equal!(resolver, "./index.js";  fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./lib/index.js";  fixture!("symlink/lib/index.js"));

    resolver.use_base_dir(&symlink_cases_path.join("linked/that"));
    should_equal!(resolver, "./lib/index.js";  fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./outer/linked/lib/index.js";  fixture!("symlink/lib/index.js"));

    resolver.use_base_dir(&symlink_cases_path.join("linked/that/lib"));
    should_equal!(resolver, "./index.js";  fixture!("symlink/lib/index.js"));

    resolver.use_base_dir(&symlink_cases_path.join("linked/that/outer/linked"));
    should_equal!(resolver, "./index.js";  fixture!("symlink/lib/index.js"));
    should_equal!(resolver, "./lib/index.js";  fixture!("symlink/lib/index.js"));

    let linked_path = symlink_cases_path.join("linked");
    let mut resolver = Resolver::default()
        .with_symlinks(false)
        .with_base_dir(&linked_path);

    should_equal!(resolver, "./index.js"; fixture!("symlink/linked/index.js"));
}
