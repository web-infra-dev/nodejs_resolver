use nodejs_resolver::{ResolveResult, Resolver};
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
    ($resolver: expr, $base_dir: expr, $target: expr; $result: expr) => {
        assert_eq!(
            $resolver.resolve($base_dir, $target),
            Ok(ResolveResult::Path($result))
        );
    };
}

macro_rules! should_ignore {
    ($resolver: expr, $base_dir: expr, $target: expr) => {
        assert_eq!(
            $resolver.resolve($base_dir, $target),
            Ok(ResolveResult::Ignored)
        );
    };
}

macro_rules! should_error {
    ($resolver: expr, $base_dir: expr, $target: expr; $expected_err_msg: expr) => {
        assert_eq!(
            $resolver.resolve($base_dir, $target),
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
    let resolver = Resolver::default().with_extensions(vec!["ts", "js"]);

    should_equal!(resolver, &extensions_cases_path, "./a"; p(vec!["extensions", "a.ts"]));
    should_equal!(resolver, &extensions_cases_path, "./a.js"; p(vec!["extensions", "a.js"]));
    should_equal!(resolver, &extensions_cases_path, "./dir"; p(vec!["extensions", "dir", "index.ts"]));
    should_equal!(resolver, &extensions_cases_path, "."; p(vec!["extensions", "index.js"]));
    should_equal!(resolver, &extensions_cases_path, "m"; p(vec!["extensions", "node_modules", "m.js"]));
    should_equal!(resolver, &extensions_cases_path, "m/"; p(vec!["extensions", "node_modules", "m", "index.ts"]));
    should_equal!(resolver, &extensions_cases_path, "module"; PathBuf::from("module"));
    should_error!(resolver, &extensions_cases_path, "./a.js/"; "Not found directory");
    should_error!(resolver, &extensions_cases_path, "m.js/"; "Not found in modules");
    should_error!(resolver, &extensions_cases_path, ""; format!("Can't resolve '' in {}", extensions_cases_path.display()));
}

#[test]
fn alias_test() {
    let alias_cases_path = get_cases_path!("tests/fixtures/alias");
    let resolver = Resolver::default().with_alias(vec![
        ("aliasA", Some("./a")),
        ("./b$", Some("./a/index")), // TODO: should we use trailing?
        ("recursive", Some("./recursive/dir")),
        ("#", Some("./c/dir")),
        ("@", Some("./c/dir")),
        ("@", Some("./c/dir")),
        ("ignore", None),
    ]);

    should_equal!(resolver, &alias_cases_path, "./a"; p(vec!["alias", "a", "index"]));
    should_equal!(resolver, &alias_cases_path, "./a/index"; p(vec!["alias", "a", "index"]));
    should_equal!(resolver, &alias_cases_path, "./a/dir"; p(vec!["alias", "a", "dir", "index"]));
    should_equal!(resolver, &alias_cases_path, "./a/dir/index"; p(vec!["alias", "a", "dir", "index"]));
    should_equal!(resolver, &alias_cases_path, "aliasA"; p(vec!["alias", "a", "index"]));
    should_equal!(resolver, &alias_cases_path, "aliasA/index"; p(vec!["alias", "a", "index"]));
    should_equal!(resolver, &alias_cases_path, "aliasA/dir"; p(vec!["alias", "a", "dir", "index"]));
    should_equal!(resolver, &alias_cases_path, "aliasA/dir/index"; p(vec!["alias", "a", "dir", "index"]));
    should_equal!(resolver, &alias_cases_path, "#"; p(vec!["alias", "c", "dir", "index"]));
    should_equal!(resolver, &alias_cases_path, "#/index"; p(vec!["alias", "c", "dir", "index"]));
    should_equal!(resolver, &alias_cases_path, "@"; p(vec!["alias", "c", "dir", "index"]));
    should_equal!(resolver, &alias_cases_path, "@/index"; p(vec!["alias", "c", "dir" ,"index"]));
    should_equal!(resolver, &alias_cases_path, "recursive"; p(vec!["alias", "recursive" , "dir" ,"index"]));
    should_equal!(resolver, &alias_cases_path, "recursive/index"; p(vec!["alias", "recursive", "dir", "index"]));
    // TODO: exact alias
    // should_equal!(resolver, &alias_cases_path, "./b?aa#bb?cc"; fixture!("alias/a/index?aa#bb?cc"));
    // should_equal!(resolver, &alias_cases_path, "./b/?aa#bb?cc"; fixture!("alias/a/index?aa#bb?cc"));
    // should_equal!(resolver, &alias_cases_path, "./b"; fixture!("alias/a/index"));
    // should_equal!(resolver, &alias_cases_path, "./b/"; fixture!("alias/a/index"));
    should_equal!(resolver, &alias_cases_path, "./b/index"; p(vec!["alias", "b" ,"index"]));
    should_equal!(resolver, &alias_cases_path, "./b/dir"; p(vec!["alias", "b", "dir", "index"]));
    should_equal!(resolver, &alias_cases_path, "./b/dir/index"; p(vec!["alias", "b", "dir", "index"]));
    should_equal!(resolver, &alias_cases_path, "./c/index"; p(vec!["alias", "c", "index"]));
    should_equal!(resolver, &alias_cases_path, "./c/dir"; p(vec!["alias", "c", "dir", "index"]));
    should_equal!(resolver, &alias_cases_path, "./c/dir/index"; p(vec!["alias", "c", "dir", "index"]));
    should_ignore!(resolver, &alias_cases_path, "ignore");
}

#[test]
fn symlink_test() {
    let symlink_cases_path = get_cases_path!("tests/fixtures/symlink");
    let resolver = Resolver::default();

    should_equal!(resolver, &symlink_cases_path.join("linked"), "./index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./node.relative.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./node.relative.sym.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./this/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./this/this/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./outer/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./outer/linked/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./outer/linked/this/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./outer/linked/this/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./that/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./that/outer/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./that/outer/linked/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked"), "./that/outer/linked/that/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));

    should_equal!(resolver, &symlink_cases_path, "./lib/index.js"; p(vec!["symlink", "lib", "index.js"]));

    should_equal!(resolver, &symlink_cases_path.join("linked/this"), "./lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked/this"), "./outer/linked/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));

    should_equal!(resolver, &symlink_cases_path.join("linked/this/lib"), "./index.js"; p(vec!["symlink", "lib", "index.js"]));

    should_equal!(resolver, &symlink_cases_path.join("linked/this/outer/linked"), "./index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked/this/outer/linked"), "./lib/index.js"; p(vec!["symlink", "lib", "index.js"]));

    should_equal!(resolver, &symlink_cases_path.join("linked/that"), "./lib/index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked/that"), "./outer/linked/lib/index.js"; p(vec!["symlink", "lib", "index.js"]));

    should_equal!(resolver, &symlink_cases_path.join("linked/that/lib"), "./index.js"; p(vec!["symlink", "lib", "index.js"]));

    should_equal!(resolver, &symlink_cases_path.join("linked/that/outer/linked"), "./index.js"; p(vec!["symlink", "lib", "index.js"]));
    should_equal!(resolver, &symlink_cases_path.join("linked/that/outer/linked"), "./lib/index.js"; p(vec!["symlink", "lib", "index.js"]));

    let linked_path = symlink_cases_path.join("linked");
    let resolver = Resolver::default().with_symlinks(false);

    should_equal!(resolver, &linked_path, "./index.js"; p(vec!["symlink", "linked", "index.js"]));
    should_equal!(resolver, &linked_path, "./this/this/index.js"; p(vec!["symlink", "linked", "this", "this", "index.js"]));
}

#[test]
fn simple_test() {
    let simple_case_path = get_cases_path!("tests/fixtures/simple");
    let resolver = Resolver::default();
    // directly
    should_equal!(resolver, &simple_case_path, "./lib/index"; p(vec!["simple", "lib", "index.js"]));
    // as directory
    should_equal!(resolver, &simple_case_path, "."; p(vec!["simple", "lib", "index.js"]));

    should_equal!(resolver, &simple_case_path.join(".."), "./simple"; p(vec!["simple", "lib", "index.js"]));
    should_equal!(resolver, &simple_case_path.join(".."), "./simple/lib/index"; p(vec!["simple", "lib", "index.js"]));
}

#[test]
fn resolve_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::default();

    should_equal!(resolver, &fixture_path, p(vec!["main1.js"]).to_str().unwrap(); p(vec!["main1.js"]));
    should_equal!(resolver, &fixture_path, "./main1.js"; p(vec!["main1.js"]));
    should_equal!(resolver, &fixture_path, "./main1"; p(vec!["main1.js"]));
    should_equal!(resolver, &fixture_path, "./main1.js?query"; p(vec!["main1.js?query"]));
    should_equal!(resolver, &fixture_path, "./main1.js#fragment"; p(vec!["main1.js#fragment"]));
    should_equal!(resolver, &fixture_path, "./main1.js#fragment?query"; p(vec!["main1.js#fragment?query"]));
    should_equal!(resolver, &fixture_path, "./main1.js?#fragment"; p(vec!["main1.js?#fragment"]));
    should_equal!(resolver, &fixture_path, "./a.js"; p(vec!["a.js"]));
    should_equal!(resolver, &fixture_path, "./a"; p(vec!["a.js"]));
    should_equal!(resolver, &fixture_path, "m1/a.js"; p(vec!["node_modules", "m1", "a.js"]));
    should_equal!(resolver, &fixture_path, "m1/a"; p(vec!["node_modules", "m1", "a.js"]));
    should_equal!(resolver, &fixture_path, "m1/a?query"; p(vec!["node_modules", "m1", "a.js?query"]));
    should_equal!(resolver, &fixture_path, "m1/a#fragment"; p(vec!["node_modules", "m1", "a.js#fragment"]));
    should_equal!(resolver, &fixture_path, "m1/a#fragment?query"; p(vec!["node_modules", "m1", "a.js#fragment?query"]));
    should_equal!(resolver, &fixture_path, "m1/a?#fragment"; p(vec!["node_modules", "m1", "a.js?#fragment"]));
    should_equal!(resolver, &fixture_path, "./dirOrFile"; p(vec!["dirOrFile.js"]));
    should_equal!(resolver, &fixture_path, "./dirOrFile/"; p(vec!["dirOrFile", "index.js"]));
    should_equal!(resolver, &fixture_path, "./main-field-self"; p(vec!["main-field-self", "index.js"]));
    should_equal!(resolver, &fixture_path, "./main-field-self2"; p(vec!["main-field-self2", "index.js"]));
    should_equal!(resolver, &fixture_path, "complexm/step1"; p(vec!["node_modules", "complexm", "step1.js"]));
    should_equal!(resolver, &fixture_path, "m2/b.js"; p(vec!["node_modules", "m2", "b.js"]));
    // edge case
    // should_equal!(resolver, "./no#fragment/#/#"; fixture!("no\0#fragment/\0#.\0#.js"));
    should_equal!(resolver, &fixture_path, "./no#fragment/#/"; p(vec!["no.js#fragment", "#",]));

    let web_modules_path = fixture_path.join("node_modules/complexm/web_modules/m1");
    should_equal!(resolver, &web_modules_path, "m2/b.js"; p(vec!["node_modules", "m2", "b.js"]));

    let multiple_modules_path = fixture_path.join("multiple_modules");
    should_equal!(resolver, &multiple_modules_path, "m1/a.js"; p(vec!["multiple_modules", "node_modules", "m1", "a.js"]));
    should_equal!(resolver, &multiple_modules_path, "m1/b.js"; p(vec!["node_modules", "m1", "b.js"]));

    should_equal!(resolver, &fixture_path.join("browser-module/node_modules"), "m1/a"; p(vec!["node_modules", "m1", "a.js"]));

    // TODO: preferRelativeResolve
}

#[test]
fn browser_filed_test() {
    let browser_module_case_path = get_cases_path!("tests/fixtures/browser-module");
    let resolver = Resolver::default().with_alias_fields(vec!["browser"]);

    should_ignore!(resolver, &browser_module_case_path, "./lib/ignore");
    should_ignore!(resolver, &browser_module_case_path, "./lib/ignore.js");
    should_equal!(resolver, &browser_module_case_path, "./lib/replaced"; p(vec!["browser-module", "lib", "browser.js"]));
    should_equal!(resolver, &browser_module_case_path, "./lib/replaced.js"; p(vec!["browser-module", "lib", "browser.js"]));
    should_equal!(resolver, &browser_module_case_path, "module-a"; p(vec!["browser-module", "browser", "module-a.js"]));
    should_equal!(resolver, &browser_module_case_path, "module-b"; p(vec!["browser-module", "node_modules", "module-c.js"]));
    should_equal!(resolver, &browser_module_case_path, "module-d"; p(vec!["browser-module", "node_modules", "module-c.js"]));
    should_equal!(resolver, &browser_module_case_path, "./toString"; p(vec!["browser-module","lib", "toString.js"]));
    should_equal!(resolver, &browser_module_case_path, "./lib/redirect"; p(vec!["browser-module", "lib", "sub.js"]));
    should_equal!(resolver, &browser_module_case_path, "./lib/redirect2"; p(vec!["browser-module", "lib", "sub", "dir", "index.js"]));
    should_equal!(resolver, &browser_module_case_path, "./lib/redirect3"; p(vec!["browser-module", "lib", "redirect3-target", "dir", "index.js"]));

    let lib_path = browser_module_case_path.join("lib");
    should_ignore!(resolver, &lib_path, "./ignore");
    should_ignore!(resolver, &lib_path, "./ignore.js");
    should_equal!(resolver, &lib_path, "./replaced"; p(vec!["browser-module", "lib", "browser.js"]));
    should_equal!(resolver, &lib_path, "./replaced.js"; p(vec!["browser-module", "lib", "browser.js"]));
    should_equal!(resolver, &lib_path, "module-a"; p(vec!["browser-module", "browser", "module-a.js"]));
    should_equal!(resolver, &lib_path, "module-b"; p(vec!["browser-module", "node_modules", "module-c.js"]));
    should_equal!(resolver, &lib_path, "module-d"; p(vec!["browser-module", "node_modules", "module-c.js"]));
    should_equal!(resolver, &lib_path, "./redirect"; p(vec!["browser-module", "lib", "sub.js"]));
    should_equal!(resolver, &lib_path, "./redirect2"; p(vec!["browser-module", "lib", "sub", "dir", "index.js"]));
    should_equal!(resolver, &lib_path, "./redirect3"; p(vec!["browser-module", "lib", "redirect3-target", "dir", "index.js"]));

    // TODO: alias_fields
}

#[test]
fn dependencies_test() {
    let dep_case_path = get_cases_path!("tests/fixtures/dependencies");
    let resolver = Resolver::default()
        .with_modules(vec!["modules", "node_modules"])
        .with_extensions(vec![".json", ".js"]);

    should_equal!(resolver, &dep_case_path.join("a/b/c"), "module/file"; p(vec!["dependencies", "a", "node_modules", "module", "file.js"]));
    should_equal!(resolver, &dep_case_path.join("a/b/c"), "other-module/file.js"; p(vec!["dependencies", "modules", "other-module", "file.js"]));

    // TODO: how passing on context?
    // TODO: Maybe it should use (`getPath`)[https://github.com/webpack/enhanced-resolve/blob/main/lib/getPaths.js]
}

#[test]
fn full_specified_test() {
    // TODO: should I need add `fullSpecified` flag?
    let full_cases_path = get_cases_path!("tests/fixtures/full/a");
    let resolver = Resolver::default()
        .with_alias(vec![("alias1", Some("./abc")), ("alias2", Some("./"))])
        .with_alias_fields(vec!["browser"]);

    should_equal!(resolver, &full_cases_path, "./abc.js"; p(vec!["full", "a", "abc.js"]));
    should_equal!(resolver, &full_cases_path, "package1/file.js"; p(vec!["full", "a", "node_modules", "package1", "file.js"]));
    should_equal!(resolver, &full_cases_path, "package1"; p(vec!["full", "a", "node_modules", "package1", "index.js"]));
    should_equal!(resolver, &full_cases_path, "package2"; p(vec!["full", "a", "node_modules", "package2", "a.js"]));
    should_equal!(resolver, &full_cases_path, "alias1"; p(vec!["full", "a", "abc.js"]));
    should_equal!(resolver, &full_cases_path, "alias2"; p(vec!["full", "a", "index.js"]));
    should_equal!(resolver, &full_cases_path, "package3"; p(vec!["full", "a", "node_modules", "package3", "dir", "index.js"]));
    should_equal!(resolver, &full_cases_path, "package3/dir"; p(vec!["full", "a", "node_modules", "package3", "dir", "index.js"]));
    should_equal!(resolver, &full_cases_path, "package4/a.js"; p(vec!["full", "a", "node_modules", "package4", "b.js"]));
    should_equal!(resolver, &full_cases_path, "."; p(vec!["full", "a", "index.js"]));
    should_equal!(resolver, &full_cases_path, "./"; p(vec!["full", "a", "index.js"]));
    should_equal!(resolver, &full_cases_path, "./dir"; p(vec!["full", "a", "dir", "index.js"]));
    should_equal!(resolver, &full_cases_path, "./dir/"; p(vec!["full", "a", "dir", "index.js"]));
    should_equal!(resolver, &full_cases_path, "./dir?123#456"; p(vec!["full", "a", "dir", "index.js?123#456"]));
    should_equal!(resolver, &full_cases_path, "./dir/?123#456"; p(vec!["full", "a", "dir", "index.js?123#456"]));
}

#[test]
fn missing_test() {
    let fixture_path = get_cases_path!("tests/fixtures/");
    let resolver = Resolver::default();
    // TODO: optimize error
    // TODO: path
    should_error!(resolver, &fixture_path, "./missing-file"; "Not found directory");
    should_error!(resolver, &fixture_path, "./missing-file.js"; "Not found directory");
    should_error!(resolver, &fixture_path, "missing-module"; "Not found in modules");
    should_error!(resolver, &fixture_path, "missing-module/missing-file"; "Not found in modules");
    should_error!(resolver, &fixture_path, "m1/missing-file"; "Not found in modules"); // TODO
    should_error!(resolver, &fixture_path, "m1/"; "Not found in modules");
    should_equal!(resolver, &fixture_path, "m1/a"; p(vec!["node_modules", "m1", "a.js"]));
}

#[test]
fn incorrect_package_test() {
    let incorrect_package_path = get_cases_path!("tests/fixtures/incorrect-package");
    let resolver = Resolver::default();

    should_error!(resolver, &incorrect_package_path.join("pack1"), "."; "Read description file failed");
    should_error!(resolver, &incorrect_package_path.join("pack2"), "."; "Read description file failed");
}

#[test]
fn scoped_packages_test() {
    let scoped_path = get_cases_path!("tests/fixtures/scoped");
    let resolver = Resolver::default().with_alias_fields(vec!["browser"]);

    should_equal!(resolver, &scoped_path, "@scope/pack1"; p(vec!["scoped", "node_modules", "@scope", "pack1", "main.js"]));
    should_equal!(resolver, &scoped_path, "@scope/pack1/main"; p(vec!["scoped", "node_modules", "@scope", "pack1", "main.js"]));
    should_equal!(resolver, &scoped_path, "@scope/pack2"; p(vec!["scoped", "node_modules", "@scope", "pack2", "main.js"]));
    should_equal!(resolver, &scoped_path, "@scope/pack2/main"; p(vec!["scoped", "node_modules", "@scope", "pack2", "main.js"]));
    should_equal!(resolver, &scoped_path, "@scope/pack2/lib"; p(vec!["scoped", "node_modules", "@scope", "pack2", "lib", "index.js"]));
}

#[test]
fn exports_fields_test() {
    // TODO: [`exports_fields`](https://github.com/webpack/enhanced-resolve/blob/main/test/exportsField.js#L2280) flag

    let export_cases_path = get_cases_path!("tests/fixtures/exports-field");
    let resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_condition_names(vec_to_set!(["webpack"]));

    should_error!(resolver, &export_cases_path, "exports-field/dist/../../../a.js"; "Package path exports-field/dist/../../../a.js is not exported");
    should_error!(resolver, &export_cases_path, "exports-field/dist/a.js"; "Package path exports-field/dist/a.js is not exported");
    should_equal!(resolver, &export_cases_path, "exports-field/dist/main.js"; p(vec!["exports-field", "node_modules", "exports-field", "lib", "lib2", "main.js"]));
    should_equal!(resolver, &export_cases_path, "@exports-field/core"; p(vec!["exports-field", "a.js"]));
    should_equal!(resolver, &export_cases_path, "./node_modules/exports-field/lib/main.js"; p(vec!["exports-field", "node_modules", "exports-field", "lib", "main.js"]));
    should_error!(resolver, &export_cases_path, "./node_modules/exports-field/dist/main"; "Not found directory");
    should_error!(resolver, &export_cases_path, "exports-field/anything/else"; "Package path exports-field/anything/else is not exported");
    should_error!(resolver, &export_cases_path, "exports-field/"; "Only requesting file allowed");
    should_error!(resolver, &export_cases_path, "exports-field/dist"; "Package path exports-field/dist is not exported");
    should_error!(resolver, &export_cases_path, "exports-field/lib"; "Package path exports-field/lib is not exported");
    should_error!(resolver, &export_cases_path, "invalid-exports-field"; "Export field key can't mixed relative path and conditional object");
    let export_cases_path2 = get_cases_path!("tests/fixtures/exports-field2");

    // TODO: maybe we need provide `full_specified` flag.
    should_equal!(resolver, &export_cases_path2, "exports-field"; p(vec!["exports-field2", "node_modules", "exports-field", "index.js"]));
    should_equal!(resolver, &export_cases_path2, "exports-field/dist/main.js"; p(vec!["exports-field2", "node_modules", "exports-field", "lib", "lib2", "main.js"]));
    should_equal!(resolver, &export_cases_path2, "exports-field/dist/browser.js"; p(vec!["exports-field2", "node_modules", "exports-field", "lib", "browser.js"]));
    should_equal!(resolver, &export_cases_path2, "exports-field/dist/browser.js?foo"; p(vec!["exports-field2", "node_modules", "exports-field", "lib", "browser.js?foo"]));
    should_error!(resolver, &export_cases_path2, "exports-field/dist/main"; "Package path exports-field/dist/main is not exported");
    // TODO: should `exports-field?foo is not exported`.
    should_error!(resolver, &export_cases_path2, "exports-field?foo"; "Package path exports-field is not exported");
    should_error!(resolver, &export_cases_path2, "exports-field#foo"; "Package path exports-field is not exported");
    should_equal!(resolver, &export_cases_path2, "exports-field/dist/browser.js#foo"; p(vec!["exports-field2", "node_modules", "exports-field", "lib", "browser.js#foo"]));

    let resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_alias_fields(vec!["browser"])
        .with_condition_names(vec_to_set!(["webpack"]));

    should_equal!(resolver, &export_cases_path, "./node_modules/exports-field/lib/main.js"; p(vec!["exports-field", "node_modules", "exports-field", "lib", "browser.js"]));
    should_equal!(resolver, &export_cases_path, "./node_modules/exports-field/dist/main.js"; p(vec!["exports-field", "node_modules", "exports-field", "lib", "browser.js"]));

    let resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_condition_names(vec_to_set!(["webpack"]));

    should_equal!(resolver, &export_cases_path, "exports-field"; p(vec!["exports-field", "node_modules", "exports-field", "x.js"]));

    let resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_alias_fields(vec!["browser"])
        .with_condition_names(vec_to_set!(["node"]));

    should_equal!(resolver, &export_cases_path, "exports-field/dist/main.js"; p(vec!["exports-field", "node_modules", "exports-field", "lib", "browser.js"]));
}

#[test]
fn imports_fields_test() {
    // TODO: ['imports_fields`](https://github.com/webpack/enhanced-resolve/blob/main/test/importsField.js#L1228)
    let import_cases_path = get_cases_path!("tests/fixtures/imports-field");
    let resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_condition_names(vec_to_set!(["webpack"]));

    should_equal!(resolver, &import_cases_path, "#c"; p(vec!["imports-field", "node_modules", "c", "index.js"]));
    should_equal!(resolver, &import_cases_path, "#imports-field"; p(vec!["imports-field", "b.js"]));
    should_equal!(resolver, &import_cases_path, "#b"; p(vec!["b.js"]));
    should_equal!(resolver, &import_cases_path, "#a/dist/main.js"; p(vec!["imports-field", "node_modules", "a", "lib", "lib2", "main.js"]));
    should_equal!(resolver, &import_cases_path, "#ccc/index.js"; p(vec!["imports-field", "node_modules", "c", "index.js"]));
    should_error!(resolver, &import_cases_path, "#a"; "Package path #a is not exported");
    should_equal!(resolver, &import_cases_path, "#c"; p(vec!["imports-field", "node_modules", "c", "index.js"]));
    should_equal!(resolver, &import_cases_path.join("dir"), "#imports-field"; p(vec!["imports-field", "b.js"]));
}

#[test]
fn without_description_file() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::default()
        .with_extensions(vec![".js"])
        .with_description_file(None);
    should_equal!(resolver, &fixture_path, "./a"; p(vec!["a.js"]));
    let export_cases_path = get_cases_path!("tests/fixtures/exports-field");
    should_equal!(resolver, &export_cases_path, "exports-field/lib"; p(vec!["exports-field", "node_modules","exports-field", "lib", "index.js"]));
}
