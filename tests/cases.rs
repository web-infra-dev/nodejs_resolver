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
        assert_eq!($resolver.resolve($resolve_target), Ok(Some($path)));
    };
}

macro_rules! should_ignore {
    ($resolver: expr, $resolve_target: expr) => {
        assert_eq!($resolver.resolve($resolve_target), Ok(None));
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
    should_error!(resolver, "m.js/"; "Not found in modules");
}

#[test]
fn alias_test() {
    let alias_cases_path = get_cases_path!("tests/fixtures/alias");
    let mut resolver = Resolver::default()
        .with_alias(vec![
            ("aliasA", Some("./a")),
            ("./b$", Some("./a/index")), // TODO: should we use trailing?
            ("recursive", Some("./recursive/dir")),
            ("#", Some("./c/dir")),
            ("@", Some("./c/dir")),
            ("@", Some("./c/dir")),
            ("ignore", None),
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
    // TODO: exact alias
    // should_equal!(resolver, "./b?aa#bb?cc"; fixture!("alias/a/index?aa#bb?cc"));
    // should_equal!(resolver, "./b/?aa#bb?cc"; fixture!("alias/a/index?aa#bb?cc"));
    // should_equal!(resolver, "./b"; fixture!("alias/a/index"));
    // should_equal!(resolver, "./b/"; fixture!("alias/a/index"));
    should_equal!(resolver, "./b/index"; fixture!("alias/b/index"));
    should_equal!(resolver, "./b/dir"; fixture!("alias/b/dir/index"));
    should_equal!(resolver, "./b/dir/index"; fixture!("alias/b/dir/index"));
    should_equal!(resolver, "./c/index"; fixture!("alias/c/index"));
    should_equal!(resolver, "./c/dir"; fixture!("alias/c/dir/index"));
    should_equal!(resolver, "./c/dir/index"; fixture!("alias/c/dir/index"));

    should_ignore!(resolver, "ignore");
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
    should_equal!(resolver, "./this/this/index.js"; fixture!("symlink/linked/this/this/index.js"));
}

#[test]
fn simple_test() {
    let simple_case_path = get_cases_path!("tests/fixtures/simple");
    let mut resolver = Resolver::default().with_base_dir(&simple_case_path);
    // directly
    should_equal!(resolver, "./lib/index"; fixture!("simple/lib/index.js"));
    // as directory
    should_equal!(resolver, "."; fixture!("simple/lib/index.js"));

    resolver.use_base_dir(&simple_case_path.join(".."));
    should_equal!(resolver, "./simple"; fixture!("simple/lib/index.js"));
    should_equal!(resolver, "./simple/lib/index"; fixture!("simple/lib/index.js"));
}

#[test]
fn browser_filed_test() {
    let browser_module_case_path = get_cases_path!("tests/fixtures/browser-module");
    let mut resolver = Resolver::default()
        .with_base_dir(&browser_module_case_path)
        .with_alias_fields(vec!["browser"]);

    should_ignore!(resolver, "./lib/ignore");
    should_ignore!(resolver, "./lib/ignore.js");
    should_equal!(resolver, "./lib/replaced"; fixture!("browser-module/lib/browser.js"));
    should_equal!(resolver, "./lib/replaced.js"; fixture!("browser-module/lib/browser.js"));
    should_equal!(resolver, "module-a"; fixture!("browser-module/browser/module-a.js"));
    should_equal!(resolver, "module-b"; fixture!("browser-module/node_modules/module-c.js"));
    should_equal!(resolver, "module-d"; fixture!("browser-module/node_modules/module-c.js"));
    should_equal!(resolver, "./toString"; fixture!("browser-module/lib/toString.js"));
    should_equal!(resolver, "./lib/redirect"; fixture!("browser-module/lib/sub.js"));
    should_equal!(resolver, "./lib/redirect2"; fixture!("browser-module/lib/sub/dir/index.js"));
    should_equal!(resolver, "./lib/redirect3"; fixture!("browser-module/lib/redirect3-target/dir/index.js"));

    resolver.use_base_dir(&browser_module_case_path.join("lib"));
    should_ignore!(resolver, "./ignore");
    should_ignore!(resolver, "./ignore.js");
    should_equal!(resolver, "./replaced"; fixture!("browser-module/lib/browser.js"));
    should_equal!(resolver, "./replaced.js"; fixture!("browser-module/lib/browser.js"));
    should_equal!(resolver, "module-a"; fixture!("browser-module/browser/module-a.js"));
    should_equal!(resolver, "module-b"; fixture!("browser-module/node_modules/module-c.js"));
    should_equal!(resolver, "module-d"; fixture!("browser-module/node_modules/module-c.js"));
    should_equal!(resolver, "./redirect"; fixture!("browser-module/lib/sub.js"));
    should_equal!(resolver, "./redirect2"; fixture!("browser-module/lib/sub/dir/index.js"));
    should_equal!(resolver, "./redirect3"; fixture!("browser-module/lib/redirect3-target/dir/index.js"));

    // TODO: alias_fields
}

#[test]
fn dependencies_test() {
    let dep_case_path = get_cases_path!("tests/fixtures/dependencies");
    let mut resolver = Resolver::default()
        .with_modules(vec!["modules", "node_modules"])
        .with_extensions(vec![".json", ".js"]);

    resolver.use_base_dir(&dep_case_path.join("a/b/c"));
    should_equal!(resolver, "module/file"; fixture!("dependencies/a/node_modules/module/file.js"));
    should_equal!(resolver, "other-module/file.js"; fixture!("dependencies/modules/other-module/file.js"));

    // TODO: how passing on context?
    // TODO: Maybe it should use (`getPath`)[https://github.com/webpack/enhanced-resolve/blob/main/lib/getPaths.js]
}

#[test]
fn full_specified_test() {
    // TODO: should I need add `fullSpecified` flag?
    let full_cases_path = get_cases_path!("tests/fixtures/full/a");
    let mut resolver = Resolver::default()
        .with_alias(vec![("alias1", Some("./abc")), ("alias2", Some("./"))])
        .with_alias_fields(vec!["browser"])
        .with_base_dir(&full_cases_path);

    should_equal!(resolver, "./abc.js"; fixture!("full/a/abc.js"));
    should_equal!(resolver, "package1/file.js"; fixture!("full/a/node_modules/package1/file.js"));
    should_equal!(resolver, "package1"; fixture!("full/a/node_modules/package1/index.js"));
    should_equal!(resolver, "package2"; fixture!("full/a/node_modules/package2/a.js"));
    should_equal!(resolver, "alias1"; fixture!("full/a/abc.js"));
    should_equal!(resolver, "alias2"; fixture!("full/a/index.js"));
    should_equal!(resolver, "package3"; fixture!("full/a/node_modules/package3/dir/index.js"));
    should_equal!(resolver, "package3/dir"; fixture!("full/a/node_modules/package3/dir/index.js"));
    should_equal!(resolver, "package4/a.js"; fixture!("full/a/node_modules/package4/b.js"));
    should_equal!(resolver, "."; fixture!("full/a/index.js"));
    should_equal!(resolver, "./"; fixture!("full/a/index.js"));
    should_equal!(resolver, "./dir"; fixture!("full/a/dir/index.js"));
    should_equal!(resolver, "./dir/"; fixture!("full/a/dir/index.js"));
    should_equal!(resolver, "./dir?123#456"; fixture!("full/a/dir/index.js?123#456"));
    should_equal!(resolver, "./dir/?123#456"; fixture!("full/a/dir/index.js?123#456"));
}

#[test]
fn missing_test() {
    let fixture_path = get_cases_path!("tests/fixtures/");
    let mut resolver = Resolver::default().with_base_dir(&fixture_path);
    // TODO: optimize error
    // TODO: path
    should_error!(resolver, "./missing-file"; "Not found directory");
    should_error!(resolver, "./missing-file.js"; "Not found directory");
    should_error!(resolver, "missing-module"; "Not found in modules");
    should_error!(resolver, "missing-module/missing-file"; "Not found in modules");
    should_error!(resolver, "m1/missing-file"; "Not found in modules"); // TODO
    should_error!(resolver, "m1/"; "Not found in modules");

    should_equal!(resolver, "m1/a"; fixture!("node_modules/m1/a.js"));
}

#[test]
fn incorrect_package_test() {
    let incorrect_package_path = get_cases_path!("tests/fixtures/incorrect-package");
    let mut resolver = Resolver::default();

    resolver.use_base_dir(&incorrect_package_path.join("pack1"));
    should_error!(resolver, "."; "Read description file failed");

    resolver.use_base_dir(&incorrect_package_path.join("pack2"));
    should_error!(resolver, "."; "Read description file failed");
}

#[test]
fn scoped_packages_test() {
    let scoped_path = get_cases_path!("tests/fixtures/scoped");
    let mut resolver = Resolver::default()
        .with_alias_fields(vec!["browser"])
        .with_base_dir(&scoped_path);

    should_equal!(resolver, "@scope/pack1"; fixture!("scoped/node_modules/@scope/pack1/main.js"));
    should_equal!(resolver, "@scope/pack1/main"; fixture!("scoped/node_modules/@scope/pack1/main.js"));
    should_equal!(resolver, "@scope/pack2"; fixture!("scoped/node_modules/@scope/pack2/main.js"));
    should_equal!(resolver, "@scope/pack2/main"; fixture!("scoped/node_modules/@scope/pack2/main.js"));
    should_equal!(resolver, "@scope/pack2/lib"; fixture!("scoped/node_modules/@scope/pack2/lib/index.js"));
}

#[test]
fn exports_fields_test() {}

#[test]
fn imports_fields_test() {}
