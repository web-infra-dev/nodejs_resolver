use nodejs_resolver::{
    test_helper::{p, vec_to_set},
    AliasMap, Cache, EnforceExtension, Error, Options, ResolveResult, Resolver,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn should_equal(resolver: &Resolver, path: &Path, request: &str, expected: PathBuf) {
    match resolver.resolve(path, request) {
        Ok(ResolveResult::Resource(resource)) => {
            assert_eq!(resource.join(), expected);
        }
        Ok(ResolveResult::Ignored) => panic!("should not ignored"),
        Err(error) => panic!("{error:?}"),
    }
}

fn should_ignored(resolver: &Resolver, path: &Path, request: &str) {
    match resolver.resolve(path, request) {
        Ok(ResolveResult::Ignored) => {}
        _ => unreachable!(),
    }
}

fn should_failed(resolver: &Resolver, path: &Path, request: &str) {
    let result = resolver.resolve(path, request);
    if !matches!(result, Err(Error::ResolveFailedTag)) {
        println!("{result:?}");
        panic!("should failed");
    }
}

fn should_overflow(resolver: &Resolver, path: &Path, request: &str) {
    let result = resolver.resolve(path, request);
    if !matches!(result, Err(Error::Overflow)) {
        println!("{result:?}");
        unreachable!();
    }
}

fn should_unexpected_json_error(
    resolver: &Resolver,
    path: &Path,
    request: &str,
    error_file_path: PathBuf,
) {
    match resolver.resolve(path, request) {
        Err(err) => match err {
            Error::UnexpectedJson((actual_error_file_path, _)) => {
                assert_eq!(error_file_path, *actual_error_file_path)
            }
            _ => {
                println!("{err:?}");
                unreachable!();
            }
        },
        Ok(result) => {
            println!("{result:?}");
            unreachable!();
        }
    }
}

fn should_unexpected_value_error(
    resolver: &Resolver,
    path: &Path,
    request: &str,
    expected_err_msg: String,
) {
    match resolver.resolve(path, request) {
        Err(err) => match err {
            Error::UnexpectedValue(err) => {
                if err.contains(&expected_err_msg) {
                } else {
                    assert_eq!(err, expected_err_msg);
                }
            }
            _ => {
                println!("{err:?}");
                unreachable!();
            }
        },
        Ok(result) => {
            println!("{result:?}");
            unreachable!();
        }
    }
}

#[test]
fn extensions_test() {
    let extensions_cases_path = p(vec!["extensions"]);
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".ts"), String::from(".js")],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &extensions_cases_path,
        "m/",
        p(vec!["extensions", "node_modules", "m", "index.ts"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a.js",
        p(vec!["extensions", "a.js"]),
    );
    should_failed(&resolver, &extensions_cases_path, "./a.js/");
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
        &extensions_cases_path.join("index"),
        ".",
        p(vec!["extensions", "index.ts"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path.join("index.js"),
        ".",
        p(vec!["extensions", "index.js"]),
    );
    should_failed(&resolver, &extensions_cases_path.join("index."), ".");
    should_failed(&resolver, &extensions_cases_path.join("inde"), ".");
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
    should_failed(&resolver, &extensions_cases_path, "./b.js");
    should_failed(&resolver, &extensions_cases_path, "fs");
    should_failed(&resolver, &extensions_cases_path, "./a.js/");
    should_failed(&resolver, &extensions_cases_path, "m.js/");
    let resolver = Resolver::new(Options {
        extensions: vec![String::from("ts"), String::from(".js")],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &extensions_cases_path.join("./a"),
        "",
        p(vec!["extensions", "a.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        "./a",
        p(vec!["extensions", "a.js"]),
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
        p(vec!["extensions", "dir", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path,
        ".",
        p(vec!["extensions", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path.join("index"),
        ".",
        p(vec!["extensions", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path.join("index.js"),
        ".",
        p(vec!["extensions", "index.js"]),
    );
    should_equal(
        &resolver,
        &extensions_cases_path.join("index."),
        ".",
        p(vec!["extensions", "index.ts"]),
    );
    should_failed(&resolver, &extensions_cases_path.join("inde"), ".");

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
        p(vec!["extensions", "node_modules", "m", "index.js"]),
    );
    should_failed(&resolver, &extensions_cases_path, "./b.js");
    should_failed(&resolver, &extensions_cases_path, "fs");
    should_failed(&resolver, &extensions_cases_path, "./a.js/");
    should_failed(&resolver, &extensions_cases_path, "m.js/");

    let extensions_cases_path = p(vec!["extensions2"]);
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js"), String::new(), String::from(".ts")], // `extensions` can start with `.` or not.
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

    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js"), String::new(), String::from(".ts")], // `extensions` can start with `.` or not.
        enforce_extension: EnforceExtension::Disabled,
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
    let resolver = Resolver::new(Options {
        alias: vec![
            (
                String::from("aliasA"),
                vec![AliasMap::Target(String::from("./a"))],
            ),
            (
                String::from("b$"),
                vec![AliasMap::Target(String::from("./a/index"))],
            ),
            (
                String::from("./b$"),
                vec![AliasMap::Target(String::from("./a/index"))],
            ),
            (
                String::from("c$"),
                vec![AliasMap::Target(
                    p(vec!["alias", "a", "index"]).display().to_string(),
                )],
            ),
            (
                String::from("fs"),
                vec![AliasMap::Target(
                    alias_cases_path
                        .join("node_modules")
                        .join("browser")
                        .join("index.js")
                        .to_string_lossy()
                        .to_string(),
                )],
            ),
            // ---
            (
                String::from("./e"),
                vec![AliasMap::Target(String::from("./d"))],
            ),
            (
                String::from("./d"),
                vec![AliasMap::Target(String::from("./e"))],
            ),
            // ---
            (
                String::from("./f"),
                vec![AliasMap::Target(String::from("./g"))],
            ),
            (
                String::from("./g"),
                vec![AliasMap::Target(String::from("./h"))],
            ),
            (
                String::from("multiAlias"),
                vec![
                    AliasMap::Target(String::from("./a1")),
                    AliasMap::Target(String::from("./a2")),
                    AliasMap::Target(String::from("./a")),
                ],
            ),
            (
                String::from("recursive"),
                vec![AliasMap::Target(String::from("./recursive/dir"))],
            ),
            (
                String::from("#"),
                vec![AliasMap::Target(String::from("./c/dir"))],
            ),
            (
                String::from("@"),
                vec![AliasMap::Target(String::from("./c/dir"))],
            ),
            (
                String::from("@start"),
                vec![AliasMap::Target(p(vec!["alias"]).display().to_string())],
            ),
            (
                String::from("@recursive/pointed"),
                vec![AliasMap::Target(String::from(
                    "@recursive/general/index.js",
                ))],
            ),
            (
                String::from("@recursive/general"),
                vec![AliasMap::Target(String::from(
                    "@recursive/general/redirect.js",
                ))],
            ),
            (
                String::from("@recursive"),
                vec![AliasMap::Target(String::from("@recursive/general"))],
            ),
            (
                String::from("./c"),
                vec![AliasMap::Target(String::from("./c"))],
            ),
            (
                String::from("alias_with_query"),
                vec![AliasMap::Target(String::from("./a?q2"))],
            ),
            (
                String::from("alias_with_fragment"),
                vec![AliasMap::Target(String::from("./a#f2"))],
            ),
            (
                String::from("alias_with_query_fragment"),
                vec![AliasMap::Target(String::from("./a?q2#f2"))],
            ),
            (String::from("ignore"), vec![AliasMap::Ignored]),
        ],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &alias_cases_path,
        "./b/index",
        p(vec!["alias", "b", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./b?aa#bb?cc",
        p(vec!["alias", "a", "index?aa#bb?cc"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./b/?aa#bb?cc",
        p(vec!["alias", "a", "index?aa#bb?cc"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./b",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./b/",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "c?query",
        p(vec!["alias", "a", "index?query"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "b?query",
        p(vec!["alias", "a", "index?query"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "b",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "c",
        p(vec!["alias", "a", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "multiAlias",
        p(vec!["alias", "a", "index"]),
    );
    should_failed(&resolver, &alias_cases_path, "ignored/a");
    should_ignored(&resolver, &alias_cases_path, "ignore/a");
    should_equal(
        &resolver,
        &alias_cases_path,
        "ignore-a",
        p(vec!["alias", "node_modules", "ignore-a", "index.js"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path.join("node_modules").join("@recursive"),
        "fs",
        p(vec!["alias", "node_modules", "browser", "index.js"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "fs",
        p(vec!["alias", "node_modules", "browser", "index.js"]),
    );
    should_overflow(&resolver, &alias_cases_path, "./e");
    should_equal(
        &resolver,
        &alias_cases_path,
        "./f",
        p(vec!["alias", "h", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "@recursive/index",
        p(vec![
            "alias",
            "node_modules",
            "@recursive",
            "general",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "@recursive/general",
        p(vec![
            "alias",
            "node_modules",
            "@recursive",
            "general",
            "redirect.js",
        ]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "@recursive/pointed",
        p(vec![
            "alias",
            "node_modules",
            "@recursive",
            "general",
            "index.js",
        ]),
    );
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
        "aliasA/",
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
    should_failed(&resolver, &alias_cases_path, "@/a.js");
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
        &alias_cases_path,
        "./c",
        p(vec!["alias", "c", "index"]),
    );
    should_equal(
        &resolver,
        &p(vec!["in_exist_path"]),
        "@start/a",
        p(vec!["alias", "a", "index"]),
    );
    should_failed(&resolver, Path::new("@start/a"), "");
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
    should_equal(
        &resolver,
        &alias_cases_path,
        "alias_with_query",
        p(vec!["alias/a/index?q2"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "alias_with_query/",
        p(vec!["alias/a/index?q2"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "alias_with_query#f1",
        p(vec!["alias/a/index?q2#f1"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "alias_with_query?q1",
        p(vec!["alias/a/index?q2"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "alias_with_fragment?q1",
        p(vec!["alias/a/index?q1#f2"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "alias_with_query_fragment?q1",
        p(vec!["alias/a/index?q2#f2"]),
    );
    should_ignored(&resolver, &alias_cases_path, "ignore");
    // test alias ordered
    let resolver = Resolver::new(Options {
        alias: vec![
            (
                String::from("@A/index"),
                vec![AliasMap::Target(String::from("./a"))],
            ),
            (
                String::from("@A"),
                vec![AliasMap::Target(String::from("./b"))],
            ),
        ],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &alias_cases_path,
        "@A/index",
        p(vec!["alias", "a", "index"]),
    );
    let resolver = Resolver::new(Options {
        alias: vec![
            (
                String::from("@A"),
                vec![AliasMap::Target(String::from("./b"))],
            ),
            (
                String::from("@A/index"),
                vec![AliasMap::Target(String::from("./a"))],
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
fn fallback_test() {
    let alias_cases_path = p(vec!["alias"]);
    let resolver = Resolver::new(Options {
        fallback: vec![
            (
                String::from("aliasA"),
                vec![AliasMap::Target(String::from("./a"))],
            ),
            // -- exists
            (
                String::from("./e"),
                vec![AliasMap::Target(String::from("./d"))],
            ),
            (
                String::from("./d"),
                vec![AliasMap::Target(String::from("./e"))],
            ),
            // --
            // in-exists
            (
                String::from("./ee"),
                vec![AliasMap::Target(String::from("./dd"))],
            ),
            (
                String::from("./dd"),
                vec![AliasMap::Target(String::from("./ee"))],
            ),
            (
                String::from("./ff"),
                vec![AliasMap::Target(String::from("./ccc"))],
            ),
        ],
        ..Default::default()
    });
    // maybe better is `should_overflow(&resolver, &alias_cases_path, "./ee");`
    should_failed(&resolver, &alias_cases_path, "./ee");
    should_failed(&resolver, &alias_cases_path, "./ff");
    should_equal(
        &resolver,
        &alias_cases_path,
        "./d",
        p(vec!["alias", "d", "index.js"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "./e",
        p(vec!["alias", "e", "index"]),
    );
    should_equal(
        &resolver,
        &alias_cases_path,
        "aliasA",
        p(vec!["alias", "a", "index"]),
    );
}

#[test]
fn symlink_test() {
    let symlink_cases_path = p(vec!["symlink"]);
    let resolver = Resolver::new(Options::default());

    should_equal(
        &resolver,
        &symlink_cases_path.join("linked"),
        "./this/lib/index.js",
        p(vec!["symlink", "lib", "index.js"]),
    );
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
    let resolver = Resolver::new(Options {
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
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
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
fn pnpm_structure_test() {
    let case_path = p(vec!["pnpm-structure", "node_modules"]);
    let resolver = Resolver::new(Default::default());
    should_equal(
        &resolver,
        &case_path.join("exports-field-a").join("lib"),
        "exports-field-aa",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-aa",
            "index.js",
        ]),
    );
    should_failed(&resolver, &case_path.join("exports-field-c"), "./b");
    should_equal(
        &resolver,
        &case_path.join("exports-field-c").join("lib"),
        "exports-field-b/b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-b",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("exports-field-c").join("lib"),
        "./b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-c",
            "lib",
            "b.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("exports-field-a").join("lib"),
        "exports-field-a",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-a",
            "lib",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("exports-field-a").join("lib"),
        "exports-field-b/b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-b",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("exports-field-a"),
        "exports-field-b/b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-b",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("exports-field-a"),
        "./lib",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "exports-field-a",
            "lib",
            "index.js",
        ]),
    );
    should_unexpected_value_error(
        &resolver,
        &case_path.join("exports-field-a"),
        "exports-field-b",
        "Package path exports-field-b is not exported".to_string(),
    );
    should_equal(
        &resolver,
        &p(vec!["pnpm-structure", "module-a"]),
        "module-b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "module-b",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &case_path.join("module-a"),
        "module-b",
        p(vec![
            "pnpm-structure",
            "node_modules",
            "module-b",
            "index.js",
        ]),
    )
}

#[test]
fn resolve_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(Options::default());
    should_equal(
        &resolver,
        &fixture_path,
        "m1/a.js",
        p(vec!["node_modules", "m1", "a.js"]),
    );
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
        p(vec!["./中文.js?query#fragment"])
            .display()
            .to_string()
            .as_str(),
        p(vec!["./中文.js?query#fragment"]),
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
    should_failed(&resolver, &p(vec!["no#fragment"]), "#/#");
    should_failed(&resolver, &p(vec!["no#fragment", "#"]), "#");
    should_failed(&resolver, &p(vec!["no#fragment", "#"]), "#.js");
    should_equal(
        &resolver,
        &p(vec!["no#fragment", "#"]),
        "../../a",
        p(vec!["a.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["no#fragment", "#"]),
        "./#",
        p(vec!["no#fragment", "#", "#.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["no#fragment"]),
        "./#/#",
        p(vec!["no#fragment", "#", "#.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["no#fragment"]),
        "./#/#.js",
        p(vec!["no#fragment", "#", "#.js"]),
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
}

#[test]
fn browser_filed_test() {
    let browser_module_case_path = p(vec!["browser-module"]);

    let resolver = Resolver::new(Default::default());
    should_equal(
        &resolver,
        &browser_module_case_path,
        "browser-string",
        p(vec![
            "browser-module",
            "node_modules",
            "browser-string",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path
            .join("node_modules")
            .join("relative")
            .join("default"),
        "../lib/a",
        p(vec![
            "browser-module",
            "node_modules",
            "relative",
            "lib",
            "a.js",
        ]),
    );

    let resolver = Resolver::new(Options {
        browser_field: true,
        ..Default::default()
    });

    should_equal(
        &resolver,
        &browser_module_case_path
            .join("node_modules")
            .join("relative")
            .join("default"),
        "../lib/a",
        p(vec![
            "browser-module",
            "node_modules",
            "relative",
            "lib",
            "b.js",
        ]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path
            .join("node_modules")
            .join("relative")
            .join("default"),
        "../lib/c",
        p(vec![
            "browser-module",
            "node_modules",
            "relative",
            "lib",
            "b.js",
        ]),
    );
    should_ignored(&resolver, &p(vec![]), "./browser-after-main");
    should_ignored(&resolver, &browser_module_case_path, ".");
    should_ignored(&resolver, &browser_module_case_path, "./lib/ignore");
    should_ignored(&resolver, &browser_module_case_path, "./lib/ignore.js");
    should_ignored(&resolver, &browser_module_case_path, "./util.inspect");
    should_ignored(&resolver, &browser_module_case_path, "./util.inspect.js");
    should_equal(
        &resolver,
        &browser_module_case_path,
        "browser-string",
        p(vec![
            "browser-module",
            "node_modules",
            "browser-string",
            "target.js",
        ]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "recursive-module",
        p(vec![
            "browser-module",
            "node_modules",
            "recursive-module",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &browser_module_case_path,
        "./lib/replaced",
        p(vec!["browser-module", "lib", "browser.js"]),
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
    should_failed(&resolver, &browser_module_case_path, "toString");
    should_failed(&resolver, &browser_module_case_path, "./toString.js");
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

    // browser with alias
    let resolver = Resolver::new(Options {
        browser_field: true,
        alias: vec![(
            String::from("./lib/toString.js"),
            vec![AliasMap::Target(String::from("module-d"))],
        )],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &browser_module_case_path,
        "./toString",
        p(vec!["browser-module", "node_modules", "module-c.js"]),
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
    should_ignored(&resolver, &browser_after_main_path, ".");

    // TODO: alias_fields
}

#[test]
fn dependencies_test() {
    let dep_case_path = p(vec!["dependencies"]);
    let resolver = Resolver::new(Options {
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
fn fully_specified_test() {
    let full_cases_path = p(vec!["full", "a"]);
    let resolver = Resolver::new(Options {
        alias: vec![
            (
                String::from("alias1"),
                vec![AliasMap::Target(
                    p(vec!["full", "a", "abc"]).display().to_string(),
                )],
            ),
            (
                String::from("alias2"),
                vec![AliasMap::Target(p(vec!["full", "a"]).display().to_string())],
            ),
        ],
        browser_field: true,
        fully_specified: true,
        ..Default::default()
    });

    should_equal(
        &resolver,
        &full_cases_path,
        "package5",
        full_cases_path.join("node_modules/package5/index.js"),
    );
    should_failed(&resolver, &full_cases_path, "package5/file");
    should_equal(
        &resolver,
        &full_cases_path,
        "package5/file.js",
        full_cases_path.join("node_modules/package5/file.js"),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package1",
        full_cases_path.join("node_modules/package1/index.js"),
    );
    should_failed(&resolver, &full_cases_path, "./abc");
    should_failed(
        &resolver,
        &full_cases_path,
        full_cases_path.join("abc").display().to_string().as_str(),
    );
    should_failed(&resolver, &full_cases_path, "package1/file");
    should_failed(&resolver, &full_cases_path, ".");
    should_failed(&resolver, &full_cases_path, "./");
    should_failed(&resolver, &full_cases_path, "package3/dir");
    should_failed(&resolver, &full_cases_path, "package3/dir/index");
    should_failed(&resolver, &full_cases_path, "package3/a");
    should_equal(
        &resolver,
        &full_cases_path,
        "package3/dir/index.js",
        full_cases_path.join("node_modules/package3/dir/index.js"),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "./abc.js",
        full_cases_path.join("abc.js"),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        full_cases_path
            .join("abc.js")
            .display()
            .to_string()
            .as_str(),
        full_cases_path.join("abc.js"),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package1/file.js",
        full_cases_path.join("node_modules/package1/file.js"),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package1",
        full_cases_path.join("node_modules/package1/index.js"),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package2",
        full_cases_path.join("node_modules/package2/a.js"),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "alias1",
        full_cases_path.join("abc.js"),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "alias2",
        full_cases_path.join("index.js"),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package3",
        full_cases_path.join("node_modules/package3/dir/index.js"),
    );
    should_equal(
        &resolver,
        &full_cases_path,
        "package4/a.js",
        full_cases_path.join("node_modules/package4/b.js"),
    );

    let resolver = Resolver::new(Options {
        alias: vec![
            (
                String::from("alias1"),
                vec![AliasMap::Target(
                    p(vec!["full", "a", "abc"]).display().to_string(),
                )],
            ),
            (
                String::from("alias2"),
                vec![AliasMap::Target(p(vec!["full", "a"]).display().to_string())],
            ),
        ],
        browser_field: true,
        ..Default::default()
    });
    should_failed(&resolver, &full_cases_path.join(".."), ".");
    should_equal(
        &resolver,
        &full_cases_path,
        "package4/a.js",
        p(vec!["full", "a", "node_modules", "package4", "b.js"]),
    );
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
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    // TODO: optimize error
    // TODO: path
    should_failed(&resolver, &fixture_path, "./missing-file");
    should_failed(&resolver, &fixture_path, "./missing-file.js");
    should_failed(&resolver, &fixture_path, "missing-module");
    should_failed(&resolver, &fixture_path, "missing-module/missing-file");
    should_failed(&resolver, &fixture_path, "m1/missing-file");
    should_failed(&resolver, &fixture_path, "m1/");
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
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    should_unexpected_json_error(
        &resolver,
        &incorrect_package_path.join("pack1"),
        ".",
        incorrect_package_path.join("pack1").join("package.json"),
    );
    should_unexpected_json_error(
        &resolver,
        &incorrect_package_path.join("pack2"),
        ".",
        incorrect_package_path.join("pack2").join("package.json"),
    );
}

#[test]
fn scoped_packages_test() {
    let scoped_path = p(vec!["scoped"]);
    let resolver = Resolver::new(Options {
        browser_field: true,
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
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        condition_names: vec_to_set(vec!["webpack"]),
        ..Default::default()
    });
    should_failed(&resolver, &export_cases_path, "@exports-field/coreaaaa");
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/x.js",
        "Package path exports-field/x.js is not exported".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/dist/a.js",
        "Trying to access out of package scope. Requesting ./../../a.js".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/dist/",
        "Resolving to directories is not possible with the exports field (request was exports-field/dist/ in".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/dist",
        "Package path exports-field/dist is not exported".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/dist/../../../a.js",
        "Trying to access out of package scope. Requesting ./lib/lib2/../../../a.js".to_string(),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "exports-field/package.json",
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "package.json",
        ]),
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
        "exports-field/dist/main",
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
    // TODO: error stack
    should_unexpected_value_error(
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
    should_failed(
        &resolver,
        &export_cases_path,
        "./node_modules/exports-field/dist/main",
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/anything/else",
        "Package path exports-field/anything/else is not exported".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/",
        "Resolving to directories is not possible with the exports field (request was exports-field/ in".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/dist",
        "Package path exports-field/dist is not exported".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/lib",
        "Package path exports-field/lib is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "invalid-exports-field",
        "Export field key should be relative path and start with \".\", but got umd".to_string(),
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
    should_unexpected_value_error(
        &resolver,
        &p(vec!["exports-field-error"]),
        "exports-field",
        "Trying to access out of package scope. Requesting ./a/../b/../../pack1/index.js"
            .to_string(),
    );

    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        browser_field: true,
        condition_names: vec_to_set(vec!["webpack"]),
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
    // exports map should works when request relative path
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
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        browser_field: true,
        condition_names: vec_to_set(vec!["node"]),
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
    // exports map should works when request abs path
    should_equal(
        &resolver,
        &export_cases_path,
        &p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "main.js",
        ])
        .display()
        .to_string(),
        p(vec![
            "exports-field",
            "node_modules",
            "exports-field",
            "main.js",
        ]),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path,
        "exports-field/main.js",
        "Package path exports-field/main.js is not exported in".to_string(),
    );

    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        condition_names: vec_to_set(vec!["require"]),
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

    let resolver = Resolver::new(Options {
        condition_names: vec_to_set(vec!["import"]),
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
    should_equal(
        &resolver,
        &export_cases_path,
        "string-side-effects",
        p(vec![
            "exports-field",
            "node_modules",
            "string-side-effects",
            "index.js",
        ]),
    );
    should_equal(
        &resolver,
        &export_cases_path,
        "string-side-effects/",
        p(vec![
            "exports-field",
            "node_modules",
            "string-side-effects",
            "index.js",
        ]),
    );
}

#[test]
fn exports_filed_test_2() {
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        condition_names: vec_to_set(vec!["webpack"]),
        ..Default::default()
    });
    let export_cases_path2 = p(vec!["exports-field2"]);
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
    should_equal(
        &resolver,
        &export_cases_path2,
        "exports-field/dist/main",
        p(vec![
            "exports-field2",
            "node_modules",
            "exports-field",
            "lib",
            "lib2",
            "main.js",
        ]),
    );
    // TODO: error stack
    // TODO: should `exports-field?foo is not exported`.
    should_unexpected_value_error(
        &resolver,
        &export_cases_path2,
        "exports-field?foo",
        "Package path exports-field is not exported".to_string(),
    );
    // TODO: error stack
    should_unexpected_value_error(
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
}

#[test]
fn exports_filed_test_3() {
    let resolver = Resolver::new(Options::default());
    should_equal(
        &resolver,
        &p(vec!["exports-field3"]),
        "outer",
        p(vec!["exports-field3", "main.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["exports-field3", "pkg1"]),
        "outer",
        p(vec!["exports-field3", "main.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["exports-field3", "pkg1", "index.js"]),
        "outer",
        p(vec!["exports-field3", "main.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["exports-field3", "pkg1"]),
        "m1",
        p(vec![
            "exports-field3",
            "pkg1",
            "node_modules",
            "m1",
            "m1.js",
        ]),
    );
}

#[test]
fn exports_filed_test_4() {
    let export_cases_path4 = p(vec!["exports-field4"]);

    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        browser_field: true,
        exports_field: vec![vec![String::from("exportsField"), String::from("exports")]],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &export_cases_path4,
        "exports-field",
        p(vec![
            "exports-field4",
            "node_modules",
            "exports-field",
            "main.js",
        ]),
    );

    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        browser_field: true,
        exports_field: vec![
            vec![String::from("exportsField"), String::from("exports")],
            vec![String::from("exports")],
        ],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &export_cases_path4,
        "exports-field",
        p(vec![
            "exports-field4",
            "node_modules",
            "exports-field",
            "main.js",
        ]),
    );

    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        browser_field: true,
        exports_field: vec![
            vec![String::from("exports")],
            vec![String::from("exportsField"), String::from("exports")],
        ],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &export_cases_path4,
        "exports-field",
        p(vec![
            "exports-field4",
            "node_modules",
            "exports-field",
            "main.js",
        ]),
    );

    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        browser_field: true,
        exports_field: vec![
            vec![String::from("ex")],
            vec![String::from("exportsField"), String::from("exports")],
        ],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &export_cases_path4,
        "exports-field",
        p(vec![
            "exports-field4",
            "node_modules",
            "exports-field",
            "index",
        ]),
    );
}

#[test]
fn exports_filed_test_5() {
    let export_cases_path5 = p(vec!["exports-field5"]);
    let resolver = Resolver::new(Options::default());
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/missing",
        "pkgexports/missing is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/invalid1",
        "pkgexports/invalid1 is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/invalid4",
        "pkgexports/invalid4 is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/sub/internal/test.js",
        "pkgexports/sub/internal/test.js is not exported".to_string(),
    );
    // FIXME:
    // should_unexpected_value_error(
    //     &resolver,
    //     &export_cases_path5,
    //     "pkgexports/sub/internal//test.js",
    //     "pkgexports/sub/internal//test.js is not exported".to_string(),
    // );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/null",
        "pkgexports/null is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/null",
        "pkgexports/null is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports////null",
        "pkgexports////null is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/null/subpath",
        "pkgexports/null/subpath is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/nofallback1",
        "nofallback1 is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/trailer",
        "pkgexports/trailer is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/sub/",
        "Resolving to directories is not possible with the exports field (request was pkgexports/sub/".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/belowdir/pkgexports/asdf.js",
        "Export should be relative path and start w".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/belowdir",
        "pkgexports/belowdir is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/invalid2",
        "pkgexports/invalid2 is not exported".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/invalid3",
        "Export should be relati".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/invalid5",
        "Package path pkgexports/invalid5 is not expor".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/nofallback2",
        "nofallback2 is not exported".to_string(),
    );
    // FIXME:
    // should_unexpected_value_error(
    //     &resolver,
    //     &export_cases_path5,
    //     "pkgexports/nodemodules",
    //     "nodemodules is not exported".to_string(),
    // );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/resolve-self-invalid",
        "Package path pkgexports/resolve-self-invalid is not".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/sub/./../asdf.js",
        "Trying to access out of package scope. Requesting ././../asd".to_string(),
    );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/sub/no-a-file.js",
        "Package path pkgexports/sub/no-a-file.js is not exported".to_string(),
    );
    // FIXME:
    // should_unexpected_value_error(
    //     &resolver,
    //     &export_cases_path5,
    //     "pkgexports/no-ext",
    //     "Trying to access out of package scope. Requesting ././../asd".to_string(),
    // );
    should_unexpected_value_error(
        &resolver,
        &export_cases_path5,
        "pkgexports/dir2/trailer",
        "Package path pkgexports/dir2/trailer is not export".to_string(),
    );
}

#[test]
fn imports_fields_test() {
    let import_cases_path = p(vec!["imports-field"]);
    let resolver = Resolver::new(Options {
        extensions: vec![String::from(".js")],
        condition_names: vec_to_set(vec!["webpack"]),
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
    should_unexpected_value_error(
        &resolver,
        &import_cases_path,
        "#b",
        "Trying to access out of package scope. Requesting ../b.js".to_string(),
    );
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
    should_unexpected_value_error(
        &resolver,
        &import_cases_path,
        "#a",
        "Package path #a can't imported in".to_string(),
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
fn prefer_relative_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(Options {
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
fn cache_fs() {
    use std::fs::write;
    use std::thread::sleep;
    use std::time::Duration;

    let fixture_path = p(vec!["cache-fs"]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    should_equal(
        &resolver,
        &fixture_path,
        ".",
        p(vec!["cache-fs", "src", "index.js"]),
    );

    write(
        fixture_path.join("package.json"),
        "{\"main\": \"./src/module.js\"}",
    )
    .expect("write failed");

    resolver.clear_entries();
    sleep(Duration::from_secs(1));

    should_equal(
        &resolver,
        &fixture_path,
        ".",
        p(vec!["cache-fs", "src", "module.js"]),
    );

    write(
        fixture_path.join("package.json"),
        "{\"main\": \"./src/index.js\"}",
    )
    .expect("write failed");

    resolver.clear_entries();
    sleep(Duration::from_secs(1));

    should_equal(
        &resolver,
        &fixture_path,
        ".",
        p(vec!["cache-fs", "src", "index.js"]),
    );
}

#[test]
fn cache_fs2() {
    use std::fs::rename;
    use std::thread::sleep;
    use std::time::Duration;
    let fixture_path = p(vec!["cache-fs2"]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    should_equal(
        &resolver,
        &fixture_path,
        "./index",
        p(vec!["cache-fs2", "index.js"]),
    );
    rename(fixture_path.join("index.js"), fixture_path.join("temp.js")).expect("rename failed");
    sleep(Duration::from_secs(1));
    should_equal(
        &resolver,
        &fixture_path,
        "./index",
        p(vec!["cache-fs2", "index.js"]),
    );
    resolver.clear_entries();
    should_failed(&resolver, &fixture_path, "./index");
    rename(fixture_path.join("temp.js"), fixture_path.join("index.js")).expect("rename failed");
    sleep(Duration::from_secs(1));
    should_failed(&resolver, &fixture_path, "./index");
    resolver.clear_entries();
    should_equal(
        &resolver,
        &fixture_path,
        "./index",
        p(vec!["cache-fs2", "index.js"]),
    );
}

#[test]
fn main_fields_test() {
    let fixture_path = p(vec![]);
    let resolver = Resolver::new(Options {
        ..Default::default()
    });
    should_equal(
        &resolver,
        &p(vec!["main-field", "src"]),
        "../",
        p(vec!["main-field", "src", "index.js"]),
    );
    should_equal(
        &resolver,
        &p(vec!["main-field", "src"]),
        "..",
        p(vec!["main-field", "src", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-end-slash",
        p(vec!["main-field-end-slash", "src", "index.js"]),
    );
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
    should_equal(
        &resolver,
        &fixture_path,
        "./main-filed-no-relative",
        p(vec!["main-filed-no-relative", "c.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path.join("main-filed-no-relative"),
        ".",
        p(vec!["main-filed-no-relative", "c.js"]),
    );

    let resolver = Resolver::new(Options {
        main_fields: vec![String::from("module")],
        ..Default::default()
    });

    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-inexist",
        p(vec!["main-field-inexist", "module.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path,
        "./main-filed-no-relative",
        p(vec!["main-filed-no-relative", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture_path.join("main-filed-no-relative"),
        ".",
        p(vec!["main-filed-no-relative", "index.js"]),
    );

    let resolver = Resolver::new(Options {
        main_fields: vec![String::from("main"), String::from("module")],

        ..Default::default()
    });

    should_equal(
        &resolver,
        &fixture_path,
        "./main-field-inexist",
        p(vec!["main-field-inexist", "module.js"]),
    );

    let resolver = Resolver::new(Options {
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
    let tsconfig_path = p(vec!["tsconfig-paths"]);
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(tsconfig_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &tsconfig_path,
        "",
        p(vec!["tsconfig-paths", "index.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "?a",
        p(vec!["tsconfig-paths", "index.ts?a"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "actual/test",
        p(vec!["tsconfig-paths", "actual", "test.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test0-success",
        p(vec!["tsconfig-paths", "test0-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test0-success?a",
        p(vec!["tsconfig-paths", "test0-success.ts?a"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test0-success.ts",
        p(vec!["tsconfig-paths", "test0-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test2/foo",
        p(vec!["tsconfig-paths", "test2-success", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test2/foo?a",
        p(vec!["tsconfig-paths", "test2-success", "foo.ts?a"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test0",
        p(vec!["tsconfig-paths", "test0-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test1/foo",
        p(vec!["tsconfig-paths", "test1-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test2/foo",
        p(vec!["tsconfig-paths", "test2-success", "foo.ts"]),
    );
    should_failed(&resolver, &tsconfig_path, "te*t3/foo");
    should_equal(
        &resolver,
        &tsconfig_path,
        "test3/foo",
        p(vec!["tsconfig-paths", "test3-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test4/foo",
        p(vec!["tsconfig-paths", "test4-first", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test5/foo",
        p(vec!["tsconfig-paths", "test5-second", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "/virtual-in/test",
        p(vec!["tsconfig-paths", "actual", "test.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "/virtual-in-star/test",
        p(vec!["tsconfig-paths", "actual", "test.ts"]),
    );
    // normal
    should_equal(
        &resolver,
        &tsconfig_path,
        "./test0-success",
        p(vec!["tsconfig-paths", "test0-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "./test1-success",
        p(vec!["tsconfig-paths", "test1-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "./test2-success/foo",
        p(vec!["tsconfig-paths", "test2-success", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "./test3-success",
        p(vec!["tsconfig-paths", "test3-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "./test4-first/foo",
        p(vec!["tsconfig-paths", "test4-first", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "./test5-second/foo",
        p(vec!["tsconfig-paths", "test5-second", "foo.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "./actual/test",
        p(vec!["tsconfig-paths", "actual", "test.ts"]),
    );
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        prefer_relative: true,
        tsconfig: Some(tsconfig_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &tsconfig_path,
        "actual/test.ts",
        p(vec!["tsconfig-paths", "actual", "test.ts"]),
    );
}

#[test]
fn tsconfig_paths_nested() {
    let tsconfig_path = p(vec!["tsconfig-paths-nested"]);
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(tsconfig_path.join("tsconfig.json")),
        ..Default::default()
    });

    should_equal(
        &resolver,
        &tsconfig_path,
        "test0",
        p(vec!["tsconfig-paths-nested", "nested", "test0-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test1/foo",
        p(vec!["tsconfig-paths-nested", "nested", "test1-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "test2/foo",
        p(vec![
            "tsconfig-paths-nested",
            "nested",
            "test2-success",
            "foo.ts",
        ]),
    );
    should_failed(&resolver, &tsconfig_path, "te*t3/foo");
    should_equal(
        &resolver,
        &tsconfig_path,
        "test3/foo",
        p(vec!["tsconfig-paths-nested", "nested", "test3-success.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
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
        &tsconfig_path,
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
        &tsconfig_path,
        "/virtual-in/test",
        p(vec!["tsconfig-paths-nested", "nested", "actual", "test.ts"]),
    );
    should_equal(
        &resolver,
        &tsconfig_path,
        "/virtual-in-star/test",
        p(vec!["tsconfig-paths-nested", "nested", "actual", "test.ts"]),
    );
}

#[test]
fn tsconfig_paths_without_base_url_test() {
    let case_path = p(vec!["tsconfig-paths-without-baseURL"]);
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_failed(&resolver, &case_path, "should-not-be-imported");
    should_equal(&resolver, &case_path, "alias/a", case_path.join("src/a.ts"));
    should_equal(
        &resolver,
        &case_path,
        "./should-not-be-imported",
        case_path.join("should-not-be-imported.ts"),
    );
}

#[test]
fn tsconfig_paths_overridden_base_url() {
    let case_path = p(vec!["tsconfig-paths-override-baseURL"]);
    let resolver = Resolver::new(Options {
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
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_failed(&resolver, &case_path, "#/test");
}

#[test]
fn tsconfig_paths_extends_from_node_modules() {
    let case_path = p(vec!["tsconfig-paths-extends-from-module"]);
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.json")),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "foo",
        p(vec!["tsconfig-paths-extends-from-module", "src", "test.ts"]),
    );

    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(case_path.join("tsconfig.scope.json")),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "foo",
        p(vec!["tsconfig-paths-extends-from-module", "src", "test.ts"]),
    );
}

#[test]
fn tsconfig_inexist() {
    let resolver = Resolver::new(Options {
        extensions: vec![".ts".to_string()],
        tsconfig: Some(p(vec![])),
        ..Default::default()
    });
    assert!(matches!(
        resolver.resolve(&p(vec![]), "./a.js"),
        Err(Error::CantFindTsConfig(_))
    ))
}

#[test]
fn load_description_data() {
    let case_path = p(vec!["exports-field"]);
    let resolver = Resolver::new(Options::default());
    let resource = if let ResolveResult::Resource(resource) = resolver
        .resolve(&case_path, "@scope/import-require")
        .unwrap()
    {
        resource
    } else {
        panic!("error")
    };

    assert_eq!(
        resource.description.as_ref().unwrap().dir().as_ref(),
        p(vec![
            "exports-field",
            "node_modules",
            "@scope",
            "import-require",
        ])
    );

    assert_eq!(
        *resource
            .description
            .as_ref()
            .unwrap()
            .data()
            .raw()
            .get("sideEffects")
            .unwrap(),
        serde_json::json!(["./index.js", "./a.js"])
    );

    let resource = if let ResolveResult::Resource(resource) =
        resolver.resolve(&case_path, "exports-field").unwrap()
    {
        resource
    } else {
        panic!("error")
    };

    assert_eq!(
        resource.description.as_ref().unwrap().dir().as_ref(),
        p(vec!["exports-field", "node_modules", "exports-field"])
    );

    assert_eq!(
        *resource
            .description
            .as_ref()
            .unwrap()
            .data()
            .raw()
            .get("sideEffects")
            .unwrap(),
        serde_json::Value::Bool(false)
    );

    let resource = if let ResolveResult::Resource(resource) =
        resolver.resolve(&case_path, "string-side-effects").unwrap()
    {
        resource
    } else {
        panic!("error")
    };

    assert_eq!(
        *resource
            .description
            .as_ref()
            .unwrap()
            .data()
            .raw()
            .get("sideEffects")
            .unwrap(),
        serde_json::Value::String("*.js".to_string())
    );

    // match resolver
    //     .load_side_effects(&p(vec!["incorrect-package", "sideeffects-map"]))
    //     .unwrap_err()
    // {
    //     Error::UnexpectedValue(error) => assert_eq!(
    //         error,
    //         format!(
    //             "sideEffects in {} had unexpected value {{}}",
    //             p(vec!["incorrect-package", "sideeffects-map", "package.json"]).display()
    //         )
    //     ),
    //     _ => unreachable!(),
    // }
    //     // match resolver
    //     //     .load_side_effects(&p(vec!["incorrect-package", "sideeffects-other-in-array"]))
    //     //     .unwrap_err()
    //     // {
    //     //     Error::UnexpectedValue(error) => assert_eq!(
    //     //         error,
    //     //         format!(
    //     //             "sideEffects in {} had unexpected value 1",
    //     //             p(vec![
    //     //                 "incorrect-package",
    //     //                 "sideeffects-other-in-array",
    //     //                 "package.json"
    //     //             ])
    //     //             .display()
    //     //         )
    //     //     ),
    //     //     _ => unreachable!(),
    //     // }
}

#[test]
fn shared_cache_test2() {
    let case_path = p(vec!["browser-module"]);
    let cache = Arc::new(Cache::default());
    let resolver1 = Resolver::new(Options {
        browser_field: true,
        external_cache: Some(cache.clone()),
        ..Default::default()
    });
    should_ignored(&resolver1, &case_path, "./lib/ignore.js");

    let resolver2 = Resolver::new(Options {
        external_cache: Some(cache.clone()),
        ..Default::default()
    });
    should_equal(
        &resolver2,
        &case_path,
        "./lib/ignore.js",
        case_path.join("lib").join("ignore.js"),
    );

    let resolver3 = Resolver::new(Options {
        external_cache: Some(cache),
        main_fields: vec!["module".to_string()],
        ..Default::default()
    });
    should_equal(
        &resolver3,
        &p(vec!["main-field-inexist"]),
        ".",
        p(vec!["main-field-inexist", "module.js"]),
    );
}

#[test]
fn empty_test() {
    let case_path = p(vec!["empty"]);
    let resolver = Resolver::new(Options::default());
    should_failed(&resolver, &case_path, ".");
    should_failed(&resolver, &p(vec![]), "./empty");
}

#[test]
fn browser_it_self() {
    let case_path = p(vec!["browser-to-self"]);
    let resolver = Resolver::new(Options {
        browser_field: true,
        condition_names: vec_to_set(vec!["browser"]),
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "a.js",
        p(vec!["browser-to-self", "node_modules", "a.js", "a.js"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "b.js",
        p(vec!["browser-to-self", "node_modules", "b.js", "b.js"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "b.js/b.js",
        p(vec!["browser-to-self", "node_modules", "b.js", "b.js"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "b.js/b.mjs",
        p(vec!["browser-to-self", "node_modules", "b.js", "b.mjs"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "b.js/package.json",
        p(vec![
            "browser-to-self",
            "node_modules",
            "b.js",
            "package.json",
        ]),
    );
    should_overflow(&resolver, &case_path, "c.js");
    let resolver = Resolver::new(Options {
        browser_field: true,
        main_fields: vec![
            "browser".to_string(),
            "module".to_string(),
            "main".to_string(),
        ],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "a.js",
        p(vec!["browser-to-self", "node_modules", "a.js", "a.js"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "c.js",
        p(vec!["browser-to-self", "node_modules", "c.js", "c.js"]),
    );
    let resolver = Resolver::new(Options {
        browser_field: false,
        main_fields: vec![
            "browser".to_string(),
            "module".to_string(),
            "main".to_string(),
        ],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "a.js",
        p(vec!["browser-to-self", "node_modules", "a.js", "a.js"]),
    );
    should_equal(
        &resolver,
        &case_path,
        "c.js",
        p(vec!["browser-to-self", "node_modules", "c.js", "c.js"]),
    );
    let resolver = Resolver::new(Options {
        browser_field: false,
        ..Default::default()
    });
    should_equal(
        &resolver,
        &case_path,
        "a.js",
        p(vec!["browser-to-self", "node_modules", "a.js", "a.js"]),
    );
    should_failed(&resolver, &case_path, "c.js");
}

#[test]
fn self_in_dep_test() {
    let path = p(vec!["self-is-dep", "src", "index.js"]);
    let resolver = Resolver::new(Options::default());
    should_equal(
        &resolver,
        &path,
        "@scope/self-is-dep/src/a",
        p(vec!["self-is-dep", "src", "b.js"]),
    );
}

#[test]
fn resolve_to_context_test() {
    let resolver = Resolver::new(Options {
        resolve_to_context: true,
        ..Default::default()
    });
    should_equal(&resolver, &p(vec![]), "./", p(vec![]));
    should_equal(&resolver, &p(vec![]), "./dirOrFile", p(vec!["dirOrFile"]));
    should_equal(
        &resolver,
        &p(vec![]),
        "./dirOrFile/../../fixtures/./dirOrFile/..",
        p(vec![]),
    );
    should_equal(&resolver, &p(vec![]), "./m.js", p(vec!["m.js"]));
    should_equal(&resolver, &p(vec![]), "./main-field", p(vec!["main-field"]));
    should_equal(
        &resolver,
        &p(vec!["browser-module"]),
        "browser-string",
        p(vec!["browser-module", "node_modules", "browser-string"]),
    );
    should_equal(
        &resolver,
        &p(vec![]),
        "./main-field-inexist",
        p(vec!["main-field-inexist"]),
    );
}

#[test]
fn resolve_modules_test() {
    let resolver = Resolver::new(Options {
        modules: vec![p(vec!["alias"]).display().to_string()],
        ..Default::default()
    });
    should_equal(&resolver, &p(vec![]), "a", p(vec!["alias", "a", "index"]));
    let resolver = Resolver::new(Options {
        modules: vec!["xxxx".to_string(), "alias".to_string()],
        ..Default::default()
    });
    should_equal(&resolver, &p(vec![]), "a", p(vec!["alias", "a", "index"]));
    should_equal(
        &resolver,
        &p(vec![]),
        "node_modules/browser",
        p(vec!["alias", "node_modules", "browser", "index.js"]),
    );
    let fixture = p(vec!["scoped", "node_modules"]);
    let resolver = Resolver::new(Options {
        modules: vec![fixture.display().to_string(), "node_modules".to_string()],
        ..Default::default()
    });
    should_equal(
        &resolver,
        &p(vec![]),
        "recursive-module",
        p(vec!["node_modules", "recursive-module", "index.js"]),
    );
    let resolver = Resolver::new(Options {
        modules: vec![fixture.display().to_string()],
        ..Default::default()
    });
    should_failed(&resolver, &p(vec![]), "recursive-module");
}

#[test]
fn extension_alias() {
    let resolver = Resolver::new(Options {
        extensions: vec![".js".to_string()],
        main_files: vec!["index.js".to_string()],
        extension_alias: vec![
            (
                ".js".to_string(),
                vec![".ts".to_string(), ".js".to_string()],
            ),
            (".mjs".to_string(), vec![".mts".to_string()]),
        ],
        ..Default::default()
    });
    let fixture = p(vec!["extension-alias"]);
    should_equal(
        &resolver,
        &fixture,
        "./index",
        p(vec!["extension-alias", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture,
        "./index.js",
        p(vec!["extension-alias", "index.ts"]),
    );
    should_equal(
        &resolver,
        &fixture,
        "./dir/index.js",
        p(vec!["extension-alias", "dir", "index.ts"]),
    );
    should_equal(
        &resolver,
        &fixture,
        "./dir2/index.js",
        p(vec!["extension-alias", "dir2", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture,
        "./dir2/index.mjs",
        p(vec!["extension-alias", "dir2", "index.mts"]),
    );
    should_failed(&resolver, &fixture, "./index.mjs");

    let fixture = p(vec!["full", "a"]);
    should_equal(
        &resolver,
        &fixture,
        "package1/index.js",
        p(vec!["full", "a", "node_modules", "package1", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture,
        "package1/index",
        p(vec!["full", "a", "node_modules", "package1", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture,
        "package1",
        p(vec!["full", "a", "node_modules", "package1", "index.js"]),
    );
}

#[test]
fn extension_alias2() {
    let resolver = Resolver::new(Options {
        extensions: vec![".js".to_string()],
        main_files: vec!["index.js".to_string()],
        extension_alias: vec![(".js".to_string(), vec![])],
        ..Default::default()
    });
    let fixture = p(vec!["extension-alias"]);
    should_equal(
        &resolver,
        &fixture,
        "./dir2",
        p(vec!["extension-alias", "dir2", "index.js"]),
    );
    should_equal(
        &resolver,
        &fixture,
        "./dir2/index",
        p(vec!["extension-alias", "dir2", "index.js"]),
    );
}
