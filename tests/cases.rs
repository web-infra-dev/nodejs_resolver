use nodejs_resolver::Resolver;
use std::collections::HashSet;
use std::path::PathBuf;

macro_rules! get_cases_path {
    ($path: expr) => {
        std::env::current_dir().unwrap().join($path)
    };
}

fn p(paths: Vec<&str>) -> PathBuf {
    paths
        .iter()
        .fold(get_cases_path!("tests").join("fixtures"), |acc, path| {
            acc.join(path)
        })
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

    should_equal!(resolver, "./a"; p(vec!["extensions", "a.ts"]));
    should_equal!(resolver, "./a.js"; p(vec!["extensions", "a.js"]));
    should_equal!(resolver, "./dir"; p(vec!["extensions", "dir", "index.ts"]));
    should_equal!(resolver, "."; p(vec!["extensions", "index.js"]));
    should_equal!(resolver, "m"; p(vec!["extensions", "node_modules", "m.js"]));
    should_equal!(resolver, "m/"; p(vec!["extensions", "node_modules", "m", "index.ts"]));
    should_equal!(resolver, "module"; PathBuf::from("module"));
    should_error!(resolver, "./a.js/"; "Not found directory");
    should_error!(resolver, "m.js/"; "Not found in modules");
    should_error!(resolver, ""; format!("Can't resolve '' in {}", extensions_cases_path.display()));
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

    should_equal!(resolver, "./a"; p(vec!["alias", "a", "index"]));
    should_equal!(resolver, "./a/index"; p(vec!["alias", "a", "index"]));
    should_equal!(resolver, "./a/dir"; p(vec!["alias", "a", "dir", "index"]));
    should_equal!(resolver, "./a/dir/index"; p(vec!["alias", "a", "dir", "index"]));
    should_equal!(resolver, "aliasA"; p(vec!["alias", "a", "index"]));
    should_equal!(resolver, "aliasA/index"; p(vec!["alias", "a", "index"]));
    should_equal!(resolver, "aliasA/dir"; p(vec!["alias", "a", "dir", "index"]));
    should_equal!(resolver, "aliasA/dir/index"; p(vec!["alias", "a", "dir", "index"]));
    should_equal!(resolver, "#"; p(vec!["alias", "c", "dir", "index"]));
    should_equal!(resolver, "#/index"; p(vec!["alias", "c", "dir", "index"]));
    should_equal!(resolver, "@"; p(vec!["alias", "c", "dir", "index"]));
    should_equal!(resolver, "@/index"; p(vec!["alias", "c", "dir" ,"index"]));
    should_equal!(resolver, "recursive"; p(vec!["alias", "recursive" , "dir" ,"index"]));
    should_equal!(resolver, "recursive/index"; p(vec!["alias", "recursive", "dir", "index"]));
    // TODO: exact alias
    // should_equal!(resolver, "./b?aa#bb?cc"; fixture!("alias/a/index?aa#bb?cc"));
    // should_equal!(resolver, "./b/?aa#bb?cc"; fixture!("alias/a/index?aa#bb?cc"));
    // should_equal!(resolver, "./b"; fixture!("alias/a/index"));
    // should_equal!(resolver, "./b/"; fixture!("alias/a/index"));
    should_equal!(resolver, "./b/index"; p(vec!["alias", "b" ,"index"]));
    should_equal!(resolver, "./b/dir"; p(vec!["alias", "b", "dir", "index"]));
    should_equal!(resolver, "./b/dir/index"; p(vec!["alias", "b", "dir", "index"]));
    should_equal!(resolver, "./c/index"; p(vec!["alias", "c", "index"]));
    should_equal!(resolver, "./c/dir"; p(vec!["alias", "c", "dir", "index"]));
    should_equal!(resolver, "./c/dir/index"; p(vec!["alias", "c", "dir", "index"]));
    should_ignore!(resolver, "ignore");
}

#[test]
fn symlink_test() {
    let symlink_cases_path = get_cases_path!("tests/fixtures/symlink");
    let mut resolver = Resolver::default().with_base_dir(&symlink_cases_path.join("linked"));

    should_equal!(resolver, "./index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./node.relative.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./node.relative.sym.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./this/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./this/this/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./outer/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./outer/linked/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./outer/linked/this/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./outer/linked/this/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./that/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./that/outer/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./that/outer/linked/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./that/outer/linked/that/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));

    let mut resolver = resolver.with_base_dir(&symlink_cases_path);
    should_equal!(resolver, "./lib/index.js";  p(vec!["symlink", "lib", "index.js"]));

    let mut resolver = resolver.with_base_dir(&symlink_cases_path.join("linked/this"));
    should_equal!(resolver, "./lib/index.js";  p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./outer/linked/lib/index.js";  p(vec!["symlink", "lib", "index.js"]));

    let mut resolver = resolver.with_base_dir(&symlink_cases_path.join("linked/this/lib"));
    should_equal!(resolver, "./index.js";  p(vec!["symlink", "lib", "index.js"]));

    let mut resolver = resolver.with_base_dir(&symlink_cases_path.join("linked/this/outer/linked"));
    should_equal!(resolver, "./index.js";  p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./lib/index.js";  p(vec!["symlink", "lib", "index.js"]));

    let mut resolver = resolver.with_base_dir(&symlink_cases_path.join("linked/that"));
    should_equal!(resolver, "./lib/index.js";  p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./outer/linked/lib/index.js";  p(vec!["symlink", "lib", "index.js"]));

    let mut resolver = resolver.with_base_dir(&symlink_cases_path.join("linked/that/lib"));
    should_equal!(resolver, "./index.js";  p(vec!["symlink", "lib", "index.js"]));

    let mut resolver = resolver.with_base_dir(&symlink_cases_path.join("linked/that/outer/linked"));
    should_equal!(resolver, "./index.js";  p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, "./lib/index.js";  p(vec!["symlink", "lib", "index.js"]));

    let linked_path = symlink_cases_path.join("linked");
    let mut resolver = Resolver::default()
        .with_symlinks(false)
        .with_base_dir(&linked_path);

    should_equal!(resolver, "./index.js"; p(vec!["symlink", "linked", "index.js"]));
    should_equal!(resolver, "./this/this/index.js"; p(vec!["symlink", "linked", "this", "this", "index.js"]));
}

#[test]
fn simple_test() {
    let simple_case_path = get_cases_path!("tests/fixtures/simple");
    let mut resolver = Resolver::default().with_base_dir(&simple_case_path);
    // directly
    should_equal!(resolver, "./lib/index"; p(vec!["simple", "lib", "index.js"]));
    // as directory
    should_equal!(resolver, "."; p(vec!["simple", "lib", "index.js"]));

    let mut resolver = resolver.with_base_dir(&simple_case_path.join(".."));
    should_equal!(resolver, "./simple"; p(vec!["simple", "lib", "index.js"]));
    should_equal!(resolver, "./simple/lib/index"; p(vec!["simple", "lib", "index.js"]));
}

#[test]
fn resolve_test() {
    let fixture_path = p(vec![]);
    let mut resolver = Resolver::default().with_base_dir(&fixture_path);

    should_equal!(resolver, p(vec!["main1.js"]).to_str().unwrap(); p(vec!["main1.js"]));
    should_equal!(resolver, "./main1.js"; p(vec!["main1.js"]));
    should_equal!(resolver, "./main1"; p(vec!["main1.js"]));
    should_equal!(resolver, "./main1.js?query"; p(vec!["main1.js?query"]));
    should_equal!(resolver, "./main1.js#fragment"; p(vec!["main1.js#fragment"]));
    should_equal!(resolver, "./main1.js#fragment?query"; p(vec!["main1.js#fragment?query"]));
    should_equal!(resolver, "./main1.js?#fragment"; p(vec!["main1.js?#fragment"]));
    should_equal!(resolver, "./a.js"; p(vec!["a.js"]));
    should_equal!(resolver, "./a"; p(vec!["a.js"]));
    should_equal!(resolver, "m1/a.js"; p(vec!["node_modules", "m1", "a.js"]));
    should_equal!(resolver, "m1/a"; p(vec!["node_modules", "m1", "a.js"]));
    should_equal!(resolver, "m1/a?query"; p(vec!["node_modules", "m1", "a.js?query"]));
    should_equal!(resolver, "m1/a#fragment"; p(vec!["node_modules", "m1", "a.js#fragment"]));
    should_equal!(resolver, "m1/a#fragment?query"; p(vec!["node_modules", "m1", "a.js#fragment?query"]));
    should_equal!(resolver, "m1/a?#fragment"; p(vec!["node_modules", "m1", "a.js?#fragment"]));
    should_equal!(resolver, "./dirOrFile"; p(vec!["dirOrFile.js"]));
    should_equal!(resolver, "./dirOrFile/"; p(vec!["dirOrFile", "index.js"]));
    should_equal!(resolver, "./main-field-self"; p(vec!["main-field-self", "index.js"]));
    should_equal!(resolver, "./main-field-self2"; p(vec!["main-field-self2", "index.js"]));
    should_equal!(resolver, "complexm/step1"; p(vec!["node_modules", "complexm", "step1.js"]));
    should_equal!(resolver, "m2/b.js"; p(vec!["node_modules", "m2", "b.js"]));
    // edge case
    // should_equal!(resolver, "./no#fragment/#/#"; fixture!("no\0#fragment/\0#.\0#.js"));
    should_equal!(resolver, "./no#fragment/#/"; p(vec!["no.js#fragment", "#",]));

    let mut resolver =
        resolver.with_base_dir(&fixture_path.join("node_modules/complexm/web_modules/m1"));
    should_equal!(resolver, "m2/b.js"; p(vec!["node_modules", "m2", "b.js"]));

    let mut resolver = resolver.with_base_dir(&fixture_path.join("multiple_modules"));
    should_equal!(resolver, "m1/a.js"; p(vec!["multiple_modules", "node_modules", "m1", "a.js"]));
    should_equal!(resolver, "m1/b.js"; p(vec!["node_modules", "m1", "b.js"]));

    let mut resolver = resolver.with_base_dir(&fixture_path.join("browser-module/node_modules"));
    should_equal!(resolver, "m1/a"; p(vec!["node_modules", "m1", "a.js"]));

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
    should_equal!(resolver, "./lib/replaced"; p(vec!["browser-module", "lib", "browser.js"]));
    should_equal!(resolver, "./lib/replaced.js"; p(vec!["browser-module", "lib", "browser.js"]));
    should_equal!(resolver, "module-a"; p(vec!["browser-module", "browser", "module-a.js"]));
    should_equal!(resolver, "module-b"; p(vec!["browser-module", "node_modules", "module-c.js"]));
    should_equal!(resolver, "module-d"; p(vec!["browser-module", "node_modules", "module-c.js"]));
    should_equal!(resolver, "./toString"; p(vec!["browser-module","lib", "toString.js"]));
    should_equal!(resolver, "./lib/redirect"; p(vec!["browser-module", "lib", "sub.js"]));
    should_equal!(resolver, "./lib/redirect2"; p(vec!["browser-module", "lib", "sub", "dir", "index.js"]));
    should_equal!(resolver, "./lib/redirect3"; p(vec!["browser-module", "lib", "redirect3-target", "dir", "index.js"]));

    let mut resolver = resolver.with_base_dir(&browser_module_case_path.join("lib"));
    should_ignore!(resolver, "./ignore");
    should_ignore!(resolver, "./ignore.js");
    should_equal!(resolver, "./replaced"; p(vec!["browser-module", "lib", "browser.js"]));
    should_equal!(resolver, "./replaced.js"; p(vec!["browser-module", "lib", "browser.js"]));
    should_equal!(resolver, "module-a"; p(vec!["browser-module", "browser", "module-a.js"]));
    should_equal!(resolver, "module-b"; p(vec!["browser-module", "node_modules", "module-c.js"]));
    should_equal!(resolver, "module-d"; p(vec!["browser-module", "node_modules", "module-c.js"]));
    should_equal!(resolver, "./redirect"; p(vec!["browser-module", "lib", "sub.js"]));
    should_equal!(resolver, "./redirect2"; p(vec!["browser-module", "lib", "sub", "dir", "index.js"]));
    should_equal!(resolver, "./redirect3"; p(vec!["browser-module", "lib", "redirect3-target", "dir", "index.js"]));

    // TODO: alias_fields
}

#[test]
fn dependencies_test() {
    let dep_case_path = get_cases_path!("tests/fixtures/dependencies");
    let mut resolver = Resolver::default()
        .with_modules(vec!["modules", "node_modules"])
        .with_extensions(vec![".json", ".js"])
        .with_base_dir(&dep_case_path.join("a/b/c"));

    should_equal!(resolver, "module/file"; p(vec!["dependencies", "a", "node_modules", "module", "file.js"]));
    should_equal!(resolver, "other-module/file.js"; p(vec!["dependencies", "modules", "other-module", "file.js"]));

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

    should_equal!(resolver, "./abc.js"; p(vec!["full", "a", "abc.js"]));
    should_equal!(resolver, "package1/file.js"; p(vec!["full", "a", "node_modules", "package1", "file.js"]));
    should_equal!(resolver, "package1"; p(vec!["full", "a", "node_modules", "package1", "index.js"]));
    should_equal!(resolver, "package2"; p(vec!["full", "a", "node_modules", "package2", "a.js"]));
    should_equal!(resolver, "alias1"; p(vec!["full", "a", "abc.js"]));
    should_equal!(resolver, "alias2"; p(vec!["full", "a", "index.js"]));
    should_equal!(resolver, "package3"; p(vec!["full", "a", "node_modules", "package3", "dir", "index.js"]));
    should_equal!(resolver, "package3/dir"; p(vec!["full", "a", "node_modules", "package3", "dir", "index.js"]));
    should_equal!(resolver, "package4/a.js"; p(vec!["full", "a", "node_modules", "package4", "b.js"]));
    should_equal!(resolver, "."; p(vec!["full", "a", "index.js"]));
    should_equal!(resolver, "./"; p(vec!["full", "a", "index.js"]));
    should_equal!(resolver, "./dir"; p(vec!["full", "a", "dir", "index.js"]));
    should_equal!(resolver, "./dir/"; p(vec!["full", "a", "dir", "index.js"]));
    should_equal!(resolver, "./dir?123#456"; p(vec!["full", "a", "dir", "index.js?123#456"]));
    should_equal!(resolver, "./dir/?123#456"; p(vec!["full", "a", "dir", "index.js?123#456"]));
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

    should_equal!(resolver, "m1/a"; p(vec!["node_modules", "m1", "a.js"]));
}

#[test]
fn incorrect_package_test() {
    let incorrect_package_path = get_cases_path!("tests/fixtures/incorrect-package");
    let resolver = Resolver::default();

    let mut resolver = resolver.with_base_dir(&incorrect_package_path.join("pack1"));
    should_error!(resolver, "."; "Read description file failed");

    let mut resolver = resolver.with_base_dir(&incorrect_package_path.join("pack2"));
    should_error!(resolver, "."; "Read description file failed");
}

#[test]
fn scoped_packages_test() {
    let scoped_path = get_cases_path!("tests/fixtures/scoped");
    let mut resolver = Resolver::default()
        .with_alias_fields(vec!["browser"])
        .with_base_dir(&scoped_path);

    should_equal!(resolver, "@scope/pack1"; p(vec!["scoped", "node_modules", "@scope", "pack1", "main.js"]));
    should_equal!(resolver, "@scope/pack1/main"; p(vec!["scoped", "node_modules", "@scope", "pack1", "main.js"]));
    should_equal!(resolver, "@scope/pack2"; p(vec!["scoped", "node_modules", "@scope", "pack2", "main.js"]));
    should_equal!(resolver, "@scope/pack2/main"; p(vec!["scoped", "node_modules", "@scope", "pack2", "main.js"]));
    should_equal!(resolver, "@scope/pack2/lib"; p(vec!["scoped", "node_modules", "@scope", "pack2", "lib", "index.js"]));
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
    // should_error!(resolver, "exports-field/dist/../../../a.js"; "Package path exports-field/dist/../../../a.js is not exported");

    // should_equal!(resolver, "@exports-field/core"; p(vec!["exports-field", "a.js"]));
    should_equal!(resolver, "exports-field/dist/main.js"; p(vec!["exports-field", "node_modules", "exports-field", "lib", "lib2", "main.js"]));
    should_error!(resolver, "exports-field/dist/../../../a.js"; "Package path exports-field/dist/../../../a.js is not exported");
    should_error!(resolver, "exports-field/dist/a.js"; "Package path exports-field/dist/a.js is not exported");
    should_equal!(resolver, "./node_modules/exports-field/lib/main.js"; p(vec!["exports-field", "node_modules", "exports-field", "lib", "main.js"]));
    should_error!(resolver, "./node_modules/exports-field/dist/main"; "Not found directory");
    should_error!(resolver, "exports-field/anything/else"; "Package path exports-field/anything/else is not exported");
    should_error!(resolver, "exports-field/"; "Only requesting file allowed");
    should_error!(resolver, "exports-field/dist"; "Package path exports-field/dist is not exported");
    should_error!(resolver, "exports-field/lib"; "Package path exports-field/lib is not exported");
    should_error!(resolver, "invalid-exports-field"; "Export field key can't mixed relative path and conditional object");

    let mut resolver = resolver.with_base_dir(&export_cases_path2);
    // TODO: maybe we need provide `full_specified` flag.
    should_equal!(resolver, "exports-field"; p(vec!["exports-field2", "node_modules", "exports-field", "index.js"]));
    should_equal!(resolver, "exports-field/dist/main.js"; p(vec!["exports-field2", "node_modules", "exports-field", "lib", "lib2", "main.js"]));
    should_equal!(resolver, "exports-field/dist/browser.js"; p(vec!["exports-field2", "node_modules", "exports-field", "lib", "browser.js"]));
    should_equal!(resolver, "exports-field/dist/browser.js?foo"; p(vec!["exports-field2", "node_modules", "exports-field", "lib", "browser.js?foo"]));
    should_error!(resolver, "exports-field/dist/main"; "Package path exports-field/dist/main is not exported");
    // TODO: should `exports-field?foo is not exported`.
    should_error!(resolver, "exports-field?foo"; "Package path exports-field is not exported");
    should_error!(resolver, "exports-field#foo"; "Package path exports-field is not exported");
    should_equal!(resolver, "exports-field/dist/browser.js#foo"; p(vec!["exports-field2", "node_modules", "exports-field", "lib", "browser.js#foo"]));

    let mut resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_alias_fields(vec!["browser"])
        .with_base_dir(&export_cases_path)
        .with_condition_names(vec_to_set!(["webpack"]));
    should_equal!(resolver, "./node_modules/exports-field/lib/main.js"; p(vec!["exports-field", "node_modules", "exports-field", "lib", "browser.js"]));
    should_equal!(resolver, "./node_modules/exports-field/dist/main.js"; p(vec!["exports-field", "node_modules", "exports-field", "lib", "browser.js"]));

    let mut resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_base_dir(&export_cases_path)
        .with_condition_names(vec_to_set!(["webpack"]));

    should_equal!(resolver, "exports-field"; p(vec!["exports-field", "node_modules", "exports-field", "x.js"]));

    let mut resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_base_dir(&export_cases_path)
        .with_alias_fields(vec!["browser"])
        .with_condition_names(vec_to_set!(["node"]));

    should_equal!(resolver, "exports-field/dist/main.js"; p(vec!["exports-field", "node_modules", "exports-field", "lib", "browser.js"]));
}

#[test]
fn imports_fields_test() {
    // TODO: ['imports_fields`](https://github.com/webpack/enhanced-resolve/blob/main/test/importsField.js#L1228)
    let import_cases_path = get_cases_path!("tests/fixtures/imports-field");
    let mut resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_base_dir(&import_cases_path)
        .with_condition_names(vec_to_set!(["webpack"]));
    should_equal!(resolver, "#c"; p(vec!["imports-field", "node_modules", "c", "index.js"]));

    should_equal!(resolver, "#imports-field"; p(vec!["imports-field", "b.js"]));
    should_equal!(resolver, "#b"; p(vec!["b.js"]));
    should_equal!(resolver, "#a/dist/main.js"; p(vec!["imports-field", "node_modules", "a", "lib", "lib2", "main.js"]));
    should_equal!(resolver, "#ccc/index.js"; p(vec!["imports-field", "node_modules", "c", "index.js"]));
    should_error!(resolver, "#a"; "Package path #a is not exported");
    // should_equal!(resolver, "#c"; p(vec!["imports-field/node_modules/c/index.js"]));

    let mut resolver = resolver.with_base_dir(&import_cases_path.join("dir"));
    should_equal!(resolver, "#imports-field"; p(vec!["imports-field", "b.js"]));
}
