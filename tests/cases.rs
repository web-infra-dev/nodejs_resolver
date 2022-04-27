use nodejs_resolver::Resolver;
use std::collections::HashSet;
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

macro_rules! vec_to_set {
    ($vec:expr) => {
        HashSet::from_iter($vec.into_iter().map(String::from))
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
fn resolve_test() {
    let fixture_path = fixture!("");
    let mut resolver = Resolver::default().with_base_dir(&fixture_path);

    should_equal!(resolver, fixture!("main1.js").to_str().unwrap(); fixture!("main1.js"));
    should_equal!(resolver, "./main1.js"; fixture!("main1.js"));
    should_equal!(resolver, "./main1"; fixture!("main1.js"));
    should_equal!(resolver, "./main1.js?query"; fixture!("main1.js?query"));
    should_equal!(resolver, "./main1.js#fragment"; fixture!("main1.js#fragment"));
    should_equal!(resolver, "./main1.js#fragment?query"; fixture!("main1.js#fragment?query"));
    should_equal!(resolver, "./main1.js?#fragment"; fixture!("main1.js?#fragment"));
    should_equal!(resolver, "./a.js"; fixture!("a.js"));
    should_equal!(resolver, "./a"; fixture!("a.js"));
    should_equal!(resolver, "m1/a.js"; fixture!("node_modules/m1/a.js"));
    should_equal!(resolver, "m1/a"; fixture!("node_modules/m1/a.js"));
    should_equal!(resolver, "m1/a?query"; fixture!("node_modules/m1/a.js?query"));
    should_equal!(resolver, "m1/a#fragment"; fixture!("node_modules/m1/a.js#fragment"));
    should_equal!(resolver, "m1/a#fragment?query"; fixture!("node_modules/m1/a.js#fragment?query"));
    should_equal!(resolver, "m1/a?#fragment"; fixture!("node_modules/m1/a.js?#fragment"));
    should_equal!(resolver, "./dirOrFile"; fixture!("dirOrFile.js"));
    should_equal!(resolver, "./dirOrFile/"; fixture!("dirOrFile/index.js"));
    should_equal!(resolver, "./main-field-self"; fixture!("main-field-self/index.js"));
    should_equal!(resolver, "./main-field-self2"; fixture!("main-field-self2/index.js"));
    should_equal!(resolver, "complexm/step1"; fixture!("node_modules/complexm/step1.js"));
    should_equal!(resolver, "m2/b.js"; fixture!("node_modules/m2/b.js"));
    // edge case
    // should_equal!(resolver, "./no#fragment/#/#"; fixture!("no\0#fragment/\0#.\0#.js"));
    should_equal!(resolver, "./no#fragment/#/"; fixture!("no.js#fragment/#/"));

    resolver.use_base_dir(&fixture_path.join("node_modules/complexm/web_modules/m1"));
    should_equal!(resolver, "m2/b.js"; fixture!("node_modules/m2/b.js"));

    resolver.use_base_dir(&fixture_path.join("multiple_modules"));
    should_equal!(resolver, "m1/a.js"; fixture!("multiple_modules/node_modules/m1/a.js"));
    should_equal!(resolver, "m1/b.js"; fixture!("node_modules/m1/b.js"));

    resolver.use_base_dir(&fixture_path.join("browser-module/node_modules"));
    should_equal!(resolver, "m1/a"; fixture!("node_modules/m1/a.js"));

    // TODO: preferRelativeResolve
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
fn exports_fields_test() {
    // TODO: [`exports_fields`](https://github.com/webpack/enhanced-resolve/blob/main/test/exportsField.js#L2280) flag

    let export_cases_path = get_cases_path!("tests/fixtures/exports-field");
    let export_cases_path2 = get_cases_path!("tests/fixtures/exports-field2");

    let mut resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_base_dir(&export_cases_path)
        .with_condition_names(vec_to_set!(["webpack"]));
    should_error!(resolver, "exports-field/dist/../../../a.js"; "Package path exports-field/dist/../../../a.js is not exported");

    should_equal!(resolver, "@exports-field/core"; fixture!("exports-field/a.js"));
    should_equal!(resolver, "exports-field/dist/main.js"; fixture!("exports-field/node_modules/exports-field/lib/lib2/main.js"));
    should_error!(resolver, "exports-field/dist/../../../a.js"; "Package path exports-field/dist/../../../a.js is not exported");
    should_error!(resolver, "exports-field/dist/a.js"; "Package path exports-field/dist/a.js is not exported");
    should_equal!(resolver, "./node_modules/exports-field/lib/main.js"; fixture!("exports-field/node_modules/exports-field/lib/main.js"));
    should_error!(resolver, "./node_modules/exports-field/dist/main"; "Not found directory");
    should_error!(resolver, "exports-field/anything/else"; "Package path exports-field/anything/else is not exported");
    should_error!(resolver, "exports-field/"; "Only requesting file allowed");
    should_error!(resolver, "exports-field/dist"; "Package path exports-field/dist is not exported");
    should_error!(resolver, "exports-field/lib"; "Package path exports-field/lib is not exported");
    should_error!(resolver, "invalid-exports-field"; "Export field key can't mixed relative path and conditional object");

    resolver.use_base_dir(&export_cases_path2);
    // TODO: maybe we need provide `full_specified` flag.
    should_equal!(resolver, "exports-field"; fixture!("exports-field2/node_modules/exports-field/index.js"));
    should_equal!(resolver, "exports-field/dist/main.js"; fixture!("exports-field2/node_modules/exports-field/lib/lib2/main.js"));
    should_equal!(resolver, "exports-field/dist/browser.js"; fixture!("exports-field2/node_modules/exports-field/lib/browser.js"));
    should_equal!(resolver, "exports-field/dist/browser.js?foo"; fixture!("exports-field2/node_modules/exports-field/lib/browser.js?foo"));
    should_error!(resolver, "exports-field/dist/main"; "Package path exports-field/dist/main is not exported");
    // TODO: should `exports-field?foo is not exported`.
    should_error!(resolver, "exports-field?foo"; "Package path exports-field is not exported");
    should_error!(resolver, "exports-field#foo"; "Package path exports-field is not exported");
    should_equal!(resolver, "exports-field/dist/browser.js#foo"; fixture!("exports-field2/node_modules/exports-field/lib/browser.js#foo"));

    let mut resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_alias_fields(vec!["browser"])
        .with_base_dir(&export_cases_path)
        .with_condition_names(vec_to_set!(["webpack"]));
    should_equal!(resolver, "./node_modules/exports-field/lib/main.js"; fixture!("exports-field/node_modules/exports-field/lib/browser.js"));
    should_equal!(resolver, "./node_modules/exports-field/dist/main.js"; fixture!("exports-field/node_modules/exports-field/lib/browser.js"));

    let mut resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_base_dir(&export_cases_path)
        .with_condition_names(vec_to_set!(["webpack"]));

    should_equal!(resolver, "exports-field"; fixture!("exports-field/node_modules/exports-field/x.js"));

    let mut resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_base_dir(&export_cases_path)
        .with_alias_fields(vec!["browser"])
        .with_condition_names(vec_to_set!(["node"]));

    should_equal!(resolver, "exports-field/dist/main.js"; fixture!("exports-field/node_modules/exports-field/lib/browser.js"));
}

#[test]
fn imports_fields_test() {
    // TODO: ['imports_fields`](https://github.com/webpack/enhanced-resolve/blob/main/test/importsField.js#L1228)
    let import_cases_path = get_cases_path!("tests/fixtures/imports-field");
    let mut resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_base_dir(&import_cases_path)
        .with_condition_names(vec_to_set!(["webpack"]));
    should_equal!(resolver, "#c"; fixture!("imports-field/node_modules/c/index.js"));

    should_equal!(resolver, "#imports-field"; fixture!("imports-field/b.js"));
    should_equal!(resolver, "#b"; fixture!("b.js"));
    should_equal!(resolver, "#a/dist/main.js"; fixture!("imports-field/node_modules/a/lib/lib2/main.js"));
    should_equal!(resolver, "#ccc/index.js"; fixture!("imports-field/node_modules/c/index.js"));
    should_error!(resolver, "#a"; "Package path #a is not exported");
    // should_equal!(resolver, "#c"; fixture!("imports-field/node_modules/c/index.js"));

    resolver.use_base_dir(&import_cases_path.join("dir"));
    should_equal!(resolver, "#imports-field"; fixture!("imports-field/b.js"));
}
