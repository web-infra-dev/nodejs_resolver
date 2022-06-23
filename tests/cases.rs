use nodejs_resolver::{AliasMap, Resolver, ResolverOptions, ResolverResult};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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

fn should_equal_remove_cache(resolver: &Resolver, path: &Path, request: &str, expected: PathBuf) {
    let resolver = Resolver::new(ResolverOptions {
        enable_unsafe_cache: false,
        ..resolver.options.clone()
    });
    match resolver.resolve(path, request) {
        Ok(ResolverResult::Info(info)) => {
            assert_eq!(info.join(), expected);
        }
        _ => unreachable!(),
    }
}

fn should_equal(resolver: &Resolver, path: &Path, request: &str, expected: PathBuf) {
    match resolver.resolve(path, request) {
        Ok(ResolverResult::Info(info)) => {
            assert_eq!(info.join(), expected);
        }
        _ => unreachable!(),
    }
    should_equal_remove_cache(resolver, path, request, expected)
}

fn should_ignored(resolver: &Resolver, path: &Path, request: &str) {
    match resolver.resolve(path, request) {
        Ok(ResolverResult::Ignored) => {}
        _ => unreachable!(),
    }
}

fn should_error(resolver: &Resolver, path: &Path, request: &str, expected_err_msg: String) {
    match resolver.resolve(path, request) {
        Err(err) => {
            assert!(err.contains(&expected_err_msg))
        }
        _ => unreachable!(),
    }
}

macro_rules! vec_to_set {
    ($vec:expr) => {
        HashSet::from_iter($vec.into_iter().map(String::from))
    };
}

#[test]
fn extensions_test() {
    let extensions_cases_path = p(vec!["extensions"]);
    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![String::from("ts"), String::from(".js")], // `extensions` can start with `.` or not.
        ..Default::default()
    });

    should_equal(
        &resolver,
        &extensions_cases_path.join("./a"),
        "",
        p(vec!["extensions", "a.ts"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a",
        p(vec!["extensions", "a.ts"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a.js",
        p(vec!["extensions", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./dir",
        p(vec!["extensions", "dir", "index.ts"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        ".",
        p(vec!["extensions", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "m",
        p(vec!["extensions", "node_modules", "m.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "m/",
        p(vec!["extensions", "node_modules", "m", "index.ts"]),
    );
    should_error(
        &resolver,
        &extensions_cases_path,
        "./b.js",
        format!(
            "Resolve './b.js' failed in '{}'",
            extensions_cases_path.display()
        ),
    );
    should_error(
        &resolver,
        &extensions_cases_path,
        "fs",
        format!(
            "Resolve 'fs' failed in '{}'",
            extensions_cases_path.display()
        ),
    );
    should_error(
        &resolver,
        &extensions_cases_path,
        "./a.js/",
        format!(
            "Resolve './a.js/' failed in '{}'",
            extensions_cases_path.display()
        ),
    );
    should_error(
        &resolver,
        &extensions_cases_path,
        "m.js/",
        format!(
            "Resolve 'm.js/' failed in '{}'",
            extensions_cases_path.display()
        ),
    );

    let extensions_cases_path = p(vec!["extensions2"]);
    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![String::from(".js"), String::from(""), String::from(".ts")], // `extensions` can start with `.` or not.
        ..Default::default()
    });
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a",
        p(vec!["extensions2", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a.js",
        p(vec!["extensions2", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        ".",
        p(vec!["extensions2", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./index",
        p(vec!["extensions2", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./b",
        p(vec!["extensions2", "b"]),
    );

    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![String::from(".js"), String::from(""), String::from(".ts")], // `extensions` can start with `.` or not.
        enforce_extension: Some(false),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a",
        p(vec!["extensions2", "a"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a.js",
        p(vec!["extensions2", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        ".",
        p(vec!["extensions2", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./index",
        p(vec!["extensions2", "index"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./b",
        p(vec!["extensions2", "b"]),
    );
}

#[test]
fn alias_test() {
    let alias_cases_path = p(vec!["alias"]);
    let resolver = Resolver::new(ResolverOptions {
        alias: vec![
            (
                String::from("aliasA"),
                AliasMap::Target(String::from("./a")),
            ),
            (
                String::from("./b$"),
                AliasMap::Target(String::from("./a/index")),
            ), // TODO: should we use trailing?
            (
                String::from("recursive"),
                AliasMap::Target(String::from("./recursive/dir")),
            ),
            (String::from("#"), AliasMap::Target(String::from("./c/dir"))),
            (String::from("@"), AliasMap::Target(String::from("./c/dir"))),
            (
                String::from("@start"),
                AliasMap::Target(p(vec!["alias"]).display().to_string()),
            ),
            (String::from("ignore"), AliasMap::Ignored),
        ],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &p(vec!["in_exist_path"]),
        "@start/a",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./a",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./a/index",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./a/dir",
        p(vec!["alias", "a", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./a/dir/index",
        p(vec!["alias", "a", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "aliasA",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "aliasA/index",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "aliasA/dir",
        p(vec!["alias", "a", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "aliasA/dir/index",
        p(vec!["alias", "a", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "#",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "#/index",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "@",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "@/index",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_error(
        &resolver,
        &alias_cases_path,
        "@/a.js",
        format!(
            "Resolve '@/a.js' failed in '{}'",
            alias_cases_path.display()
        ),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "recursive",
        p(vec!["alias", "recursive", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "recursive/index",
        p(vec!["alias", "recursive", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &p(vec!["in_exist_path"]),
        "@start/a",
        p(vec!["alias", "a", "index"]),
    );
    should_error(
        &resolver,
        &Path::new("@start/a"),
        "",
        "Resolve '' failed in '@start/a'".to_string(),
    );

    // TODO: exact alias
    // should_equal(resolver, &alias_cases_path, "./b?aa#bb?cc", fixture!("alias/a/index?aa#bb?cc"));
    // should_equal(resolver, &alias_cases_path, "./b/?aa#bb?cc", fixture!("alias/a/index?aa#bb?cc"));
    // should_equal(resolver, &alias_cases_path, "./b", fixture!("alias/a/index"));
    // should_equal(resolver, &alias_cases_path, "./b/", fixture!("alias/a/index"));
    should_equal(
        &resolver,
        &alias_cases_path,
        "./b/index",
        p(vec!["alias", "b", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./b/dir",
        p(vec!["alias", "b", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./b/dir/index",
        p(vec!["alias", "b", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./c/index",
        p(vec!["alias", "c", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./c/dir",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./c/dir/index",
        p(vec!["alias", "c", "dir", "index"]),
    );
    should_ignored(&resolver, &alias_cases_path, "ignore");

    // test alias ordered
    let resolver = Resolver::new(ResolverOptions {
        alias: vec![
            (
                String::from("@A/index"),
                AliasMap::Target(String::from("./a")),
            ),
            (String::from("@A"), AliasMap::Target(String::from("./b"))),
        ],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &alias_cases_path,
        "@A/index",
        p(vec!["alias", "a", "index"]),
    );
    let resolver = Resolver::new(ResolverOptions {
        alias: vec![
            (String::from("@A"), AliasMap::Target(String::from("./b"))),
            (
                String::from("@A/index"),
                AliasMap::Target(String::from("./a")),
            ),
        ],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &alias_cases_path,
        "@A/index",
        p(vec!["alias", "b", "index"]),
    );
}

#[test]
fn symlink_test() {
    let symlink_cases_path = p(vec!["symlink"]);
    let resolver = Resolver::new(ResolverOptions::default());

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./node.relative.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./node.relative.sym.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./this/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./this/this/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./outer/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./outer/linked/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./outer/linked/this/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./outer/linked/this/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./that/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./that/outer/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./that/outer/linked/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./that/outer/linked/that/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path,
        "./lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/this"),
        "./lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/this"),
        "./outer/linked/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/this/lib"),
        "./index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/this/outer/linked"),
        "./index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/this/outer/linked"),
        "./lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/that"),
        "./lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/that"),
        "./outer/linked/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/that/lib"),
        "./index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/that/outer/linked"),
        "./index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &symlink_cases_path.join("linked/that/outer/linked"),
        "./lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );

    let linked_path = symlink_cases_path.join("linked");
    let resolver = Resolver::new(ResolverOptions {
        symlinks: false,
        ..Default::default()
    });

    should_equal(
        &resolver,
        &linked_path,
        "./index.js",
        p(vec!["symlink", "linked", "index.js"]),
    );
    should_equal(
        &resolver,
        &linked_path,
        "./this/this/index.js",
        p(vec!["symlink", "linked", "this", "this", "index.js"]),
    );
}

#[test]
fn simple_test() {
    let simple_case_path = p(vec!["simple"]);
    let resolver = Resolver::new(ResolverOptions::default());
    should_equal(
        &resolver,
        &p(vec!["in-exist-path"]),
        &p(vec!["simple", "lib", "index"]).display().to_string(),
        p(vec!["simple", "lib", "index.js"]),
    );
    // directly
    should_equal(
        &resolver,
        &simple_case_path,
        "./lib/index",
        p(vec!["simple", "lib", "index.js"]),
    );
    // as directory
    should_equal(
        &resolver,
        &simple_case_path,
        ".",
        p(vec!["simple", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &simple_case_path,
        "",
        p(vec!["simple", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &simple_case_path.join(".."),
        "./simple",
        p(vec!["simple", "lib", "index.js"]),
    );
    should_equal(
        &resolver,
        &simple_case_path.join(".."),
        "./simple/lib/index",
        p(vec!["simple", "lib", "index.js"]),
    );

    should_equal(
        &resolver,
        &p(vec!["in-exist-path"]),
        &p(vec!["simple", "lib", "index"]).display().to_string(),
        p(vec!["simple", "lib", "index.js"]),
    );
}

#[test]
fn resolve_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(ResolverOptions::default());
    should_equal(
        &resolver,
        &fixture_path,
        p(vec!["main1.js"]).to_str().unwrap(),
        p(vec!["main1.js"]),
    );
    should_equal(&resolver, &fixture_path, "./main1.js", p(vec!["main1.js"]));
    should_equal(&resolver, &fixture_path, "./main1", p(vec!["main1.js"]));
    should_equal(
        &resolver,
        &fixture_path,
        "./main1.js?query",
        p(vec!["main1.js?query"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main1.js#fragment",
        p(vec!["main1.js#fragment"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main1.js#fragment?query",
        p(vec!["main1.js#fragment?query"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main1.js?#fragment",
        p(vec!["main1.js?#fragment"]),
    );
    should_equal(&resolver, &fixture_path, "./a.js", p(vec!["a.js"]));
    should_equal(&resolver, &fixture_path, "./a", p(vec!["a.js"]));
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a.js",
        p(vec!["node_modules", "m1", "a.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a",
        p(vec!["node_modules", "m1", "a.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a?query",
        p(vec!["node_modules", "m1", "a.js?query"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a#fragment",
        p(vec!["node_modules", "m1", "a.js#fragment"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a#fragment?query",
        p(vec!["node_modules", "m1", "a.js#fragment?query"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a?#fragment",
        p(vec!["node_modules", "m1", "a.js?#fragment"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./dirOrFile",
        p(vec!["dirOrFile.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./dirOrFile/",
        p(vec!["dirOrFile", "index.js"]),
    );

    should_equal(
        &resolver,
        &fixture_path,
        "complexm/step1",
        p(vec!["node_modules", "complexm", "step1.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m2/b.js",
        p(vec!["node_modules", "m2", "b.js"]),
    );
    // edge case
    // should_equal(&resolver, "./no#fragment/#/#", fixture!("no\0#fragment/\0#.\0#.js"));
    should_equal(
        &resolver,
        &fixture_path,
        "./no#fragment/#/",
        p(vec!["no.js#fragment", "#"]),
    );

    let web_modules_path = fixture_path.join("node_modules/complexm/web_modules/m1");
    should_equal(
        &resolver,
        &web_modules_path,
        "m2/b.js",
        p(vec!["node_modules", "m2", "b.js"]),
    );

    let multiple_modules_path = fixture_path.join("multiple_modules");
    should_equal(
        &resolver,
        &multiple_modules_path,
        "m1/a.js",
        p(vec!["multiple_modules", "node_modules", "m1", "a.js"]),
    );
    should_equal(
        &resolver,
        &multiple_modules_path,
        "m1/b.js",
        p(vec!["node_modules", "m1", "b.js"]),
    );

    should_equal(
        &resolver,
        &fixture_path.join("browser-module/node_modules"),
        "m1/a",
        p(vec!["node_modules", "m1", "a.js"]),
    );

    // TODO: preferRelativeResolve
}

#[test]
fn browser_filed_test() {
    let browser_module_case_path = p(vec!["browser-module"]);
    let resolver = Resolver::new(ResolverOptions {
        alias_fields: vec![String::from("browser")],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/redirect2",
        p(vec!["browser-module", "lib", "sub", "dir", "index.js"]),
    );

    should_ignored(&resolver, &browser_module_case_path, ".");
    should_ignored(&resolver, &browser_module_case_path, "./lib/ignore");
    should_ignored(&resolver, &browser_module_case_path, "./lib/ignore.js");
    should_ignored(&resolver, &browser_module_case_path, "./util.inspect");
    should_ignored(&resolver, &browser_module_case_path, "./util.inspect.js");
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/replaced",
        p(vec!["browser-module", "lib", "browser.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/replaced.js",
        p(vec!["browser-module", "lib", "browser.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "module-a",
        p(vec!["browser-module", "browser", "module-a.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "module-b",
        p(vec!["browser-module", "node_modules", "module-c.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "module-d",
        p(vec!["browser-module", "node_modules", "module-c.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./toString",
        p(vec!["browser-module", "lib", "toString.js"]),
    );
    should_error(
        &resolver,
        &browser_module_case_path,
        "./toString.js",
        format!(
            "Resolve \'./toString.js\' failed in '{}'",
            browser_module_case_path.display()
        ),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/redirect",
        p(vec!["browser-module", "lib", "sub.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/redirect2",
        p(vec!["browser-module", "lib", "sub", "dir", "index.js"]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/redirect3",
        p(vec![
            "browser-module",
            "lib",
            "redirect3-target",
            "dir",
            "index.js",
        ]),
    );

    let lib_path = browser_module_case_path.join("lib");
    should_ignored(&resolver, &lib_path, "./ignore");
    should_ignored(&resolver, &lib_path, "./ignore.js");
    should_equal(
        &resolver,
        &lib_path,
        "./toString",
        p(vec!["browser-module", "lib", "toString.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "./toString.js",
        p(vec!["browser-module", "lib", "toString.js"]),
    );

    should_equal(
        &resolver,
        &lib_path,
        "./replaced",
        p(vec!["browser-module", "lib", "browser.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "./replaced.js",
        p(vec!["browser-module", "lib", "browser.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "module-a",
        p(vec!["browser-module", "browser", "module-a.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "module-b",
        p(vec!["browser-module", "node_modules", "module-c.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "module-d",
        p(vec!["browser-module", "node_modules", "module-c.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "./redirect",
        p(vec!["browser-module", "lib", "sub.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "./redirect2",
        p(vec!["browser-module", "lib", "sub", "dir", "index.js"]),
    );
    should_equal(
        &resolver,
        &lib_path,
        "./redirect3",
        p(vec![
            "browser-module",
            "lib",
            "redirect3-target",
            "dir",
            "index.js",
        ]),
    );

    let browser_after_main_path = p(vec!["browser-after-main"]);
    should_ignored(&resolver, &browser_after_main_path, ".");
    should_ignored(&resolver, &p(vec![]), "./browser-after-main");

    // TODO: alias_fields
}

#[test]
fn dependencies_test() {
    let dep_case_path = p(vec!["dependencies"]);
    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![String::from(".json"), String::from(".js")],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &dep_case_path.join("a/b/c"), // <dep_case>/a/b/c is an in-exist path
        "some-module/index",
        p(vec![
            "dependencies",
            "a",
            "b",
            "node_modules",
            "some-module",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &dep_case_path.join("a/b/c"), // <dep_case>/a/b/c is an in-exist path
        "module/file",
        p(vec![
            "dependencies",
            "a",
            "node_modules",
            "module",
            "file.js",
        ]),
    );
    should_equal(
        &resolver,
        &dep_case_path.join("a/b/c"), // <dep_case>/a/b/c is an in-exist path
        "other-module/file.js",
        p(vec![
            "dependencies",
            "node_modules",
            "other-module",
            "file.js",
        ]),
    );

    should_equal(
        &resolver,
        &dep_case_path.join("a/b"),
        "some-module/index",
        p(vec![
            "dependencies",
            "a",
            "b",
            "node_modules",
            "some-module",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &dep_case_path.join("a/b"),
        "module/file",
        p(vec![
            "dependencies",
            "a",
            "node_modules",
            "module",
            "file.js",
        ]),
    );
    should_equal(
        &resolver,
        &dep_case_path.join("a/b"),
        "other-module/file.js",
        p(vec![
            "dependencies",
            "node_modules",
            "other-module",
            "file.js",
        ]),
    );
    // TODO: how passing on context?
    // TODO: Maybe it should use (`getPath`)[https://github.com/webpack/enhanced-resolve/blob/main/lib/getPaths.js]
}

#[test]
fn full_specified_test() {
    // TODO: should I need add `fullSpecified` flag?
    let full_cases_path = p(vec!["full", "a"]);
    let resolver = Resolver::new(ResolverOptions {
        alias: vec![
            (
                String::from("alias1"),
                AliasMap::Target(String::from("./abc")),
            ),
            (String::from("alias2"), AliasMap::Target(String::from("./"))),
        ],
        alias_fields: vec![String::from("browser")],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &full_cases_path,
        "./abc.js",
        p(vec!["full", "a", "abc.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package1/file.js",
        p(vec!["full", "a", "node_modules", "package1", "file.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package1",
        p(vec!["full", "a", "node_modules", "package1", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package2",
        p(vec!["full", "a", "node_modules", "package2", "a.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "alias1",
        p(vec!["full", "a", "abc.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "alias2",
        p(vec!["full", "a", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package3",
        p(vec![
            "full",
            "a",
            "node_modules",
            "package3",
            "dir",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package3/dir",
        p(vec![
            "full",
            "a",
            "node_modules",
            "package3",
            "dir",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package4/a.js",
        p(vec!["full", "a", "node_modules", "package4", "b.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        ".",
        p(vec!["full", "a", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./",
        p(vec!["full", "a", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./dir",
        p(vec!["full", "a", "dir", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./dir/",
        p(vec!["full", "a", "dir", "index.js"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./dir?123#456",
        p(vec!["full", "a", "dir", "index.js?123#456"]),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./dir/?123#456",
        p(vec!["full", "a", "dir", "index.js?123#456"]),
    );
}

#[test]
fn missing_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(ResolverOptions::default());
    // TODO: optimize error
    // TODO: path
    should_error(
        &resolver,
        &fixture_path,
        "./missing-file",
        format!(
            "Resolve './missing-file' failed in '{}'",
            fixture_path.display()
        ),
    );
    should_error(
        &resolver,
        &fixture_path,
        "./missing-file.js",
        format!(
            "Resolve './missing-file.js' failed in '{}'",
            fixture_path.display()
        ),
    );
    should_error(
        &resolver,
        &fixture_path,
        "missing-module",
        format!(
            "Resolve 'missing-module' failed in '{}'",
            fixture_path.display()
        ),
    );
    should_error(
        &resolver,
        &fixture_path,
        "missing-module/missing-file",
        format!(
            "Resolve 'missing-module/missing-file' failed in '{}'",
            fixture_path.display()
        ),
    );
    should_error(
        &resolver,
        &fixture_path,
        "m1/missing-file",
        format!(
            "Resolve 'm1/missing-file' failed in '{}'",
            fixture_path.display()
        ),
    );
    should_error(
        &resolver,
        &fixture_path,
        "m1/",
        format!("Resolve 'm1/' failed in '{}'", fixture_path.display()),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a",
        p(vec!["node_modules", "m1", "a.js"]),
    );
}

#[test]
fn incorrect_package_test() {
    let incorrect_package_path = p(vec!["incorrect-package"]);
    let resolver = Resolver::new(ResolverOptions::default());
    should_error(
        &resolver,
        &incorrect_package_path.join("pack1"),
        ".",
        "Parse".to_string(),
    );
    should_error(
        &resolver,
        &incorrect_package_path.join("pack2"),
        ".",
        "Parse".to_string(),
    );
}

#[test]
fn scoped_packages_test() {
    let scoped_path = p(vec!["scoped"]);
    let resolver = Resolver::new(ResolverOptions {
        alias_fields: vec![String::from("browser")],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &scoped_path,
        "@scope/pack1",
        p(vec!["scoped", "node_modules", "@scope", "pack1", "main.js"]),
    );
    should_equal(
        &resolver,
        &scoped_path,
        "@scope/pack1/main",
        p(vec!["scoped", "node_modules", "@scope", "pack1", "main.js"]),
    );
    should_equal(
        &resolver,
        &scoped_path,
        "@scope/pack2",
        p(vec!["scoped", "node_modules", "@scope", "pack2", "main.js"]),
    );
    should_equal(
        &resolver,
        &scoped_path,
        "@scope/pack2/main",
        p(vec!["scoped", "node_modules", "@scope", "pack2", "main.js"]),
    );
    should_equal(
        &resolver,
        &scoped_path,
        "@scope/pack2/lib",
        p(vec![
            "scoped",
            "node_modules",
            "@scope",
            "pack2",
            "lib",
            "index.js",
        ]),
    );
}

#[test]
fn exports_fields_test() {
    // TODO: [`exports_fields`](https://github.com/webpack/enhanced-resolve/blob/main/test/exportsField.js#L2280) flag

    let export_cases_path = p(vec!["exports-field"]);

    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![String::from("js")],
        condition_names: vec_to_set!(["import"]),
        ..Default::default()
    });

    should_equal(
        &resolver,
        &export_cases_path,
        "@scope/import-require",
        p(vec![
            "exports-field",
            "node_modules",
            "@scope",
            "import-require",
            "dist",
            "esm",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "@scope/import-require/a",
        p(vec![
            "exports-field",
            "node_modules",
            "@scope",
            "import-require",
            "dist",
            "esm",
            "a",
            "index.js",
        ]),
    );

    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![String::from("js")],
        condition_names: vec_to_set!(["require"]),
        ..Default::default()
    });

    should_equal(
        &resolver,
        &export_cases_path,
        "@scope/import-require",
        p(vec![
            "exports-field",
            "node_modules",
            "@scope",
            "import-require",
            "dist",
            "esm",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "@scope/import-require/a",
        p(vec![
            "exports-field",
            "node_modules",
            "@scope",
            "import-require",
            "dist",
            "cjs",
            "a",
            "index.js",
        ]),
    );

    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![String::from("js")],
        condition_names: vec_to_set!(["webpack"]),
        ..Default::default()
    });
    should_error(
        &resolver,
        &export_cases_path,
        "@exports-field/coreaaaa",
        format!(
            "Resolve '@exports-field/coreaaaa' failed in '{}'",
            export_cases_path.display()
        ),
    );
    should_error(
        &resolver,
        &export_cases_path,
        "exports-field/x.js",
        "Package path exports-field/x.js is not exported".to_string(),
    );
    should_error(
        &resolver,
        &export_cases_path,
        "exports-field/dist/a.js",
        "Package path exports-field/dist/a.js is not exported".to_string(),
    );
    should_error(
        &resolver,
        &export_cases_path,
        "exports-field/dist/../../../a.js",
        "Package path exports-field/dist/../../../a.js is not exported".to_string(),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "exports-field/dist/main.js",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "lib2",
            "main.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "@exports-field/core",
        p(vec!["exports-field", "a.js"]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "./b",
        p(vec!["exports-field", "b.js"]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "./a",
        p(vec!["exports-field", "a.js"]),
    );
    should_error(
        &resolver,
        &export_cases_path,
        "@exports-field/core/a",
        "Package path @exports-field/core/a is not exported".to_string(),
    );
    // `exports` only used in `Normal` target.
    should_equal(
        &resolver,
        &export_cases_path,
        "./node_modules/exports-field/lib/main.js",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "main.js",
        ]),
    );
    should_error(
        &resolver,
        &export_cases_path,
        "./node_modules/exports-field/dist/main",
        format!(
            "Resolve './node_modules/exports-field/dist/main' failed in '{}'",
            export_cases_path.display()
        ),
    );
    should_error(
        &resolver,
        &export_cases_path,
        "exports-field/anything/else",
        "Package path exports-field/anything/else is not exported".to_string(),
    );
    should_error(
        &resolver,
        &export_cases_path,
        "exports-field/",
        "Only requesting file allowed".to_string(),
    );
    should_error(
        &resolver,
        &export_cases_path,
        "exports-field/dist",
        "Package path exports-field/dist is not exported".to_string(),
    );
    should_error(
        &resolver,
        &export_cases_path,
        "exports-field/lib",
        "Package path exports-field/lib is not exported".to_string(),
    );
    should_error(
        &resolver,
        &export_cases_path,
        "invalid-exports-field",
        "Export field key can't mixed relative path and conditional object".to_string(),
    );
    // `exports` filed take precedence over `main`
    should_equal(
        &resolver,
        &export_cases_path,
        "exports-field",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "x.js",
        ]),
    );

    let export_cases_path2 = p(vec!["exports-field2"]);

    // TODO: maybe we need provide `full_specified` flag.
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field/dist/main.js",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "lib",
            "lib2",
            "main.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field/dist/browser.js",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field/dist/browser.js?foo",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js?foo",
        ]),
    );
    should_error(
        &resolver,
        &export_cases_path2,
        "exports-field/dist/main",
        "Package path exports-field/dist/main is not exported".to_string(),
    );
    // TODO: should `exports-field?foo is not exported`.
    should_error(
        &resolver,
        &export_cases_path2,
        "exports-field?foo",
        "Package path exports-field is not exported".to_string(),
    );
    should_error(
        &resolver,
        &export_cases_path2,
        "exports-field#foo",
        "Package path exports-field is not exported".to_string(),
    );
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field/dist/browser.js#foo",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js#foo",
        ]),
    );

    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![String::from("js")],
        alias_fields: vec![String::from("browser")],
        condition_names: vec_to_set!(["webpack"]),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &export_cases_path,
        "./node_modules/exports-field/lib/main.js",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "./node_modules/exports-field/dist/main.js",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js",
        ]),
    );

    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![String::from(".js")],
        alias_fields: vec![String::from("browser")],
        condition_names: vec_to_set!(["node"]),
        ..Default::default()
    });

    should_equal(
        &resolver,
        &export_cases_path,
        "exports-field/dist/main.js",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "browser.js",
        ]),
    );
}

#[test]
fn imports_fields_test() {
    // TODO: ['imports_fields`](https://github.com/webpack/enhanced-resolve/blob/main/test/importsField.js#L1228)
    let import_cases_path = p(vec!["imports-field"]);
    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![String::from(".js")],
        condition_names: vec_to_set!(["webpack"]),
        ..Default::default()
    });

    should_equal(
        &resolver,
        &import_cases_path,
        "#c-redirect/index",
        p(vec!["imports-field", "node_modules", "c", "index.js"]),
    );

    should_equal(
        &resolver,
        &import_cases_path,
        "c",
        p(vec!["imports-field", "node_modules", "c", "index.js"]),
    );

    should_equal(
        &resolver,
        &import_cases_path,
        "#c",
        p(vec!["imports-field", "node_modules", "c", "index.js"]),
    );
    should_equal(
        &resolver,
        &import_cases_path,
        "#imports-field",
        p(vec!["imports-field", "b.js"]),
    );
    // [ERROR] should fix: https://github.com/webpack/enhanced-resolve/issues/346
    should_equal(&resolver, &import_cases_path, "#b", p(vec!["b.js"]));
    should_equal(
        &resolver,
        &import_cases_path,
        "#a/dist/main.js",
        p(vec![
            "imports-field",
            "node_modules",
            "a",
            "lib",
            "lib2",
            "main.js",
        ]),
    );
    should_equal(
        &resolver,
        &import_cases_path,
        "#ccc/index.js",
        p(vec!["imports-field", "node_modules", "c", "index.js"]),
    );
    should_error(
        &resolver,
        &import_cases_path,
        "#a",
        "Package path #a is not exported".to_string(),
    );
    should_equal(
        &resolver,
        &import_cases_path,
        "#c",
        p(vec!["imports-field", "node_modules", "c", "index.js"]),
    );
    should_equal(
        &resolver,
        &import_cases_path.join("dir"),
        "#imports-field",
        p(vec!["imports-field", "b.js"]),
    );
}

#[test]
fn without_description_file_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![String::from(".js")],
        description_file: None,
        ..Default::default()
    });
    should_equal(&resolver, &fixture_path, "./a", p(vec!["a.js"]));
    let export_cases_path = p(vec!["exports-field"]);
    should_equal(
        &resolver,
        &export_cases_path,
        "exports-field/lib",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "lib",
            "index.js",
        ]),
    );
}

#[test]
fn prefer_relative_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(ResolverOptions {
        prefer_relative: true,
        ..Default::default()
    });
    should_equal(&resolver, &fixture_path, "main1.js", p(vec!["main1.js"]));
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a.js",
        p(vec!["node_modules", "m1", "a.js"]),
    );
}

#[test]
fn pkg_info_cache_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(Default::default());
    let _ = resolver.resolve(&fixture_path, "./browser-module/lib/browser");

    assert!(resolver.unsafe_cache.is_some());
    let pkg_info_cache = &resolver.unsafe_cache.as_ref().unwrap().pkg_info;
    assert_eq!(pkg_info_cache.len(), 3);
    assert_eq!(
        pkg_info_cache
            .get(&p(vec!["browser-module", "lib", "browser"]))
            .unwrap()
            .as_ref()
            .unwrap()
            .abs_dir_path,
        p(vec!["browser-module"])
    );
    assert_eq!(
        pkg_info_cache
            .get(&p(vec!["browser-module", "lib"]))
            .unwrap()
            .as_ref()
            .unwrap()
            .abs_dir_path,
        p(vec!["browser-module"])
    );
    assert_eq!(
        pkg_info_cache
            .get(&p(vec!["browser-module"]))
            .unwrap()
            .as_ref()
            .unwrap()
            .abs_dir_path,
        p(vec!["browser-module"])
    );

    // without cache
    let resolver = Resolver::new(ResolverOptions {
        enable_unsafe_cache: false,
        ..Default::default()
    });
    assert!(resolver.unsafe_cache.is_none());
    let _ = resolver.resolve(&fixture_path, "./browser-module/lib/browser");
    assert!(resolver.unsafe_cache.is_none());

    // TODO: cache after resolve node_modules
    // TODO: more cases.
}

#[test]

fn main_fields_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(ResolverOptions::default());

    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-self",
        p(vec!["main-field-self", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-self2",
        p(vec!["main-field-self2", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main-field",
        p(vec!["main-field", "src", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-inexist",
        p(vec!["main-field-inexist", "index.js"]),
    );

    let resolver = Resolver::new(ResolverOptions {
        main_fields: vec![String::from("module")],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-inexist",
        p(vec!["main-field-inexist", "module.js"]),
    );

    let resolver = Resolver::new(ResolverOptions {
        main_fields: vec![String::from("main"), String::from("module")],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-inexist",
        p(vec!["main-field-inexist", "module.js"]),
    );

    let resolver = Resolver::new(ResolverOptions {
        main_fields: vec![String::from("module"), String::from("main")],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-inexist",
        p(vec!["main-field-inexist", "module.js"]),
    );
}

#[test]
fn tsconfig_paths_test() {
    let tsconfig_paths = p(vec!["tsconfig-paths"]);
    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(tsconfig_paths.join("tsconfig.json")),
        ..Default::default()
    });

    should_equal(
        &resolver,
        &tsconfig_paths,
        "test0",
        p(vec!["tsconfig-paths", "test0-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test1/foo",
        p(vec!["tsconfig-paths", "test1-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test2/foo",
        p(vec!["tsconfig-paths", "test2-success", "foo.ts"]),
    );
    should_error(
        &resolver,
        &tsconfig_paths,
        "te*t3/foo",
        format!(
            "Resolve 'te*t3/foo' failed in '{}'",
            tsconfig_paths.display()
        ),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test3/foo",
        p(vec!["tsconfig-paths", "test3-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test4/foo",
        p(vec!["tsconfig-paths", "test4-first", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test5/foo",
        p(vec!["tsconfig-paths", "test5-second", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "/virtual-in/test",
        p(vec!["tsconfig-paths", "actual", "test.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "/virtual-in-star/test",
        p(vec!["tsconfig-paths", "actual", "test.ts"]),
    );

    // normal
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test0-success",
        p(vec!["tsconfig-paths", "test0-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test1-success",
        p(vec!["tsconfig-paths", "test1-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test2-success/foo",
        p(vec!["tsconfig-paths", "test2-success", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test3-success",
        p(vec!["tsconfig-paths", "test3-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test4-first/foo",
        p(vec!["tsconfig-paths", "test4-first", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./test5-second/foo",
        p(vec!["tsconfig-paths", "test5-second", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "./actual/test",
        p(vec!["tsconfig-paths", "actual", "test.ts"]),
    );
}

#[test]
fn tsconfig_paths_nested() {
    let tsconfig_paths = p(vec!["tsconfig-paths-nested"]);
    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(tsconfig_paths.join("tsconfig.json")),
        ..Default::default()
    });

    should_equal(
        &resolver,
        &tsconfig_paths,
        "test0",
        p(vec!["tsconfig-paths-nested", "nested", "test0-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test1/foo",
        p(vec!["tsconfig-paths-nested", "nested", "test1-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test2/foo",
        p(vec![
            "tsconfig-paths-nested",
            "nested",
            "test2-success",
            "foo.ts",
        ]),
    );
    should_error(
        &resolver,
        &tsconfig_paths,
        "te*t3/foo",
        format!(
            "Resolve 'te*t3/foo' failed in '{}'",
            tsconfig_paths.display()
        ),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test3/foo",
        p(vec!["tsconfig-paths-nested", "nested", "test3-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test4/foo",
        p(vec![
            "tsconfig-paths-nested",
            "nested",
            "test4-first",
            "foo.ts",
        ]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "test5/foo",
        p(vec![
            "tsconfig-paths-nested",
            "nested",
            "test5-second",
            "foo.ts",
        ]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "/virtual-in/test",
        p(vec!["tsconfig-paths-nested", "nested", "actual", "test.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_paths,
        "/virtual-in-star/test",
        p(vec!["tsconfig-paths-nested", "nested", "actual", "test.ts"]),
    );
}

#[test]
fn tsconfig_paths_without_base_url_test() {
    let case_path = p(vec!["tsconfig-paths-without-baseURL"]);
    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_error(
        &resolver,
        &case_path,
        "should-not-be-imported",
        format!(
            "Resolve 'should-not-be-imported' failed in '{}'",
            case_path.display()
        ),
    )
}

#[test]
fn tsconfig_paths_overridden_base_url() {
    let case_path = p(vec!["tsconfig-paths-override-baseURL"]);
    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "#/test",
        p(vec!["tsconfig-paths-override-baseURL", "src", "test.ts"]),
    );
}

#[test]
fn tsconfig_paths_missing_base_url() {
    let case_path = p(vec!["tsconfig-paths-missing-baseURL"]);
    let resolver = Resolver::new(ResolverOptions {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_error(
        &resolver,
        &case_path,
        "#/test",
        format!("Resolve '#/test' failed in '{}'", case_path.display()),
    );
}
