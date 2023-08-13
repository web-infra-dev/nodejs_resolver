use std::collections::HashSet;

/// port from https://github.com/webpack/enhanced-resolve/blob/main/lib/util/entrypoints.js
use crate::{Error, RResult};

type MappingValue = serde_json::Value;
type ConditionalMapping = serde_json::Map<String, MappingValue>;

pub struct ImportsField;

pub struct ExportsField;

const DEFAULT_MARK: &str = "default";

fn conditional_mapping<'a>(
    map: &'a ConditionalMapping,
    condition_names: &'a HashSet<String>,
) -> RResult<Option<&'a serde_json::Value>> {
    let mut lookup: Vec<(&ConditionalMapping, Vec<String>, usize)> =
        vec![(map, map.keys().map(String::from).collect(), 0)];
    'outer: while !lookup.is_empty() {
        let (mapping, conditions, j) = lookup.last().unwrap();
        let len = conditions.len();
        for (i, condition) in conditions.iter().enumerate().skip(*j) {
            if condition == DEFAULT_MARK {
                if i != len - 1 {
                    return Err(Error::UnexpectedValue(
                        "Default condition should be last one".to_string(),
                    ));
                } else if let Some(value) = mapping.get(DEFAULT_MARK) {
                    match value {
                        MappingValue::Object(inner) => {
                            let len = lookup.len();
                            lookup[len - 1].2 = i + 1;
                            lookup.push((inner, inner.keys().map(String::from).collect(), 0));
                            continue 'outer;
                        }
                        _ => return Ok(Some(value)),
                    }
                }
            }

            if condition_names.contains(condition) {
                if let Some(value) = mapping.get(condition) {
                    match value {
                        MappingValue::Object(inner) => {
                            let len = lookup.len();
                            lookup[len - 1].2 = i + 1;
                            lookup.push((inner, inner.keys().map(String::from).collect(), 0));
                            continue 'outer;
                        }
                        _ => return Ok(Some(value)),
                    }
                }
            }
        }
        lookup.pop();
    }

    Ok(None)
}

/// TODO: should seal all functions except
///  `build_field` and `field_process`.
pub trait Field {
    fn check_target(relative_path: &str) -> Result<(), String> {
        let relative_path_chars = relative_path.chars().collect::<Vec<char>>();
        let slash_index_list = get_next_list(&relative_path_chars, '/');
        let mut last_non_slash_index = 0;
        let mut cd = 0;
        while let Some(&Some(slash_index)) = slash_index_list.get(last_non_slash_index) {
            if relative_path_chars[last_non_slash_index] == '.'
                && relative_path_chars[last_non_slash_index + 1] == '.'
            {
                cd -= 1;
                if cd < 0 {
                    return Err(format!(
                        "Trying to access out of package scope. Requesting {relative_path}"
                    ));
                }
            } else if relative_path_chars[last_non_slash_index] == '.' {
            } else {
                cd += 1;
            }
            last_non_slash_index = slash_index + 1;
        }
        Ok(())
    }

    fn assert_target(exp: &str, expect_folder: bool) -> RResult<()>;
    fn assert_request(request: &str) -> RResult<String>;
    fn find_match<'a>(
        json_value: &'a serde_json::Value,
        request: &'a str,
    ) -> RResult<Option<(&'a MappingValue, &'a str, bool, bool)>>;

    fn target_mapping(
        remaining_request: &str,
        is_pattern: bool,
        is_subpath_mapping: bool,
        target: &str,
    ) -> RResult<String> {
        if remaining_request.is_empty() {
            Self::assert_target(target, false)?;
            return Ok(target.to_string());
        } else if is_subpath_mapping {
            Self::assert_target(target, true)?;
            return Ok(format!("{target}{remaining_request}"));
        }

        Self::assert_target(target, false)?;
        if is_pattern {
            let to = &remaining_request.replace('$', "$$");
            Ok(target.replace('*', to))
        } else {
            Ok(target.to_string())
        }
    }

    fn mapping(
        remaining_request: &str,
        is_pattern: bool,
        is_subpath_mapping: bool,
        mapping: &MappingValue,
        condition_names: &HashSet<String>,
    ) -> RResult<Vec<String>> {
        Ok(match mapping {
            MappingValue::String(target) => {
                vec![Self::target_mapping(
                    remaining_request,
                    is_pattern,
                    is_subpath_mapping,
                    target,
                )?]
            }
            MappingValue::Array(target) => target
                .iter()
                .filter_map(|item| {
                    Self::mapping(
                        remaining_request,
                        is_pattern,
                        is_subpath_mapping,
                        item,
                        condition_names,
                    )
                    .ok()
                })
                .flatten()
                .collect(),
            MappingValue::Object(map) => match conditional_mapping(map, condition_names)? {
                Some(mapping_value) => Self::mapping(
                    remaining_request,
                    is_pattern,
                    is_subpath_mapping,
                    mapping_value,
                    condition_names,
                )?,
                None => vec![],
            },
            _ => vec![],
        })
    }

    fn field_process<'a>(
        root: &'a serde_json::Value,
        target: &'a str,
        condition_names: &'a HashSet<String>,
    ) -> RResult<Vec<String>> {
        let request = Self::assert_request(target)?;
        let Some((mapping, remaining_request, is_subpath_mapping, is_pattern)) = Self::find_match(root, &request)? else {
            return Ok(vec![])
        };
        Self::mapping(remaining_request, is_pattern, is_subpath_mapping, mapping, condition_names)
    }
}

impl Field for ExportsField {
    fn assert_request(request: &str) -> RResult<String> {
        if !request.starts_with('.') {
            Err(Error::UnexpectedValue(format!(
                "Request should be relative path and start with '.', but got {request}"
            )))
        } else if request.len() == 1 {
            Ok(request.to_string())
        } else if !request.starts_with("./") {
            Err(Error::UnexpectedValue(format!(
                "Request should be relative path and start with '.', but got {request}"
            )))
        } else {
            Ok(request.to_string())
        }
    }

    fn assert_target(exp: &str, expect_folder: bool) -> RResult<()> {
        if exp.len() < 2 || exp.starts_with('/') || (exp.starts_with('.') && !exp.starts_with("./"))
        {
            Err(Error::UnexpectedValue(format!(
                "Export should be relative path and start with \"./\", but got {exp}"
            )))
        } else if exp.ends_with('/') != expect_folder {
            if expect_folder {
                Err(Error::UnexpectedValue(format!("Expected {exp} is folder mapping")))
            } else {
                Err(Error::UnexpectedValue(format!("Expected {exp} is file mapping")))
            }
        } else {
            Ok(())
        }
    }

    /// reference: https://nodejs.org/api/packages.html#exports
    fn find_match<'a>(
        json_value: &'a serde_json::Value,
        request: &'a str,
    ) -> RResult<Option<(&'a MappingValue, &'a str, bool, bool)>> {
        match json_value {
            serde_json::Value::Object(map) => {
                for (i, key) in map.keys().enumerate() {
                    if !key.starts_with('.') {
                        if i == 0 {
                            for key in map.keys() {
                                if key.starts_with('.') || key.starts_with('/') {
                                    return Err(Error::UnexpectedValue(format!(
                                        "Export field key should be relative path and start with \"./\", but got {key}"
                                    )));
                                }
                            }
                            // {"." => Object};
                            if request != "." {
                                return Ok(None);
                            } else {
                                return Ok(Some((json_value, ".", false, false)));
                            }
                        } else {
                            return Err(Error::UnexpectedValue(format!(
                                "Export field key should be relative path and start with \".\", but got {key}"
                            )));
                        }
                    } else if key.len() == 1 {
                        // key == "."
                        continue;
                    } else if key.as_bytes().get(1) != Some(&b'/') {
                        return Err(Error::UnexpectedValue(format!(
                            "Export field key should be relative path and start with \"./\", but got {key}"
                        )));
                    }
                }
                return Ok(find_normalized_match_in_object(map, request));
            }
            serde_json::Value::Array(_) | serde_json::Value::String(_) => {
                if request != "." {
                    Ok(None)
                } else {
                    Ok(Some((json_value, ".", false, false)))
                }
            }
            _ => Ok(None),
        }
    }
}

impl Field for ImportsField {
    fn assert_request(request: &str) -> RResult<String> {
        if !request.starts_with('#') {
            Err(Error::UnexpectedValue(format!("Request should start with #, but got {request}")))
        } else if request.len() == 1 {
            Err(Error::UnexpectedValue("Request should have at least 2 characters".to_string()))
        } else if request.starts_with("#/") {
            Err(Error::UnexpectedValue(format!(
                "Import field key should not start with #/, but got {request}"
            )))
        } else if request.ends_with('/') {
            Err(Error::UnexpectedValue("Only requesting file allowed".to_string()))
        } else {
            Ok(request.to_string())
        }
    }

    fn assert_target(exp: &str, expect_folder: bool) -> RResult<()> {
        let is_folder = exp.ends_with('/');
        if is_folder != expect_folder {
            if expect_folder {
                Err(Error::UnexpectedValue(format!("Expected {exp} is folder mapping")))
            } else {
                Err(Error::UnexpectedValue(format!("Expected {exp} is file mapping")))
            }
        } else {
            Ok(())
        }
    }

    fn find_match<'a>(
        json_value: &'a serde_json::Value,
        request: &'a str,
    ) -> RResult<Option<(&'a MappingValue, &'a str, bool, bool)>> {
        let field = match json_value {
            MappingValue::Object(field) => field,
            _ => return Ok(None),
        };
        for key in field.keys() {
            if !key.starts_with('#') {
                return Err(Error::UnexpectedValue(format!(
                    "Imports field key should start with #, but got {key}"
                )));
            } else if key.len() == 1 {
                // key eq "#"
                return Err(Error::UnexpectedValue(format!(
                    "Imports field key should have at least 2 characters, but got {key}"
                )));
            } else if key.starts_with("#/") {
                return Err(Error::UnexpectedValue(format!(
                    "Import field key should not start with #/, but got {key}"
                )));
            }
        }
        Ok(find_normalized_match_in_object(field, request))
    }
}

fn get_next_list(path: &[char], target: char) -> Vec<Option<usize>> {
    // TODO: rewrite it use fp.
    let len = path.len();
    let mut last_index = None;
    let mut index_list = vec![None; len];
    for i in (0..len).rev() {
        if path[i] == target {
            last_index = Some(i);
        }
        index_list[i] = last_index;
    }
    index_list
}

fn pattern_key_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let a_pattern_index = a.find('*');
    let b_pattern_index = b.find('*');
    let base_len_a = if let Some(i) = a_pattern_index { i + 1 } else { a.len() };
    let base_len_b = if let Some(i) = b_pattern_index { i + 1 } else { b.len() };
    if base_len_a > base_len_b {
        std::cmp::Ordering::Less
    } else if base_len_b > base_len_a || a_pattern_index.is_none() {
        std::cmp::Ordering::Greater
    } else if b_pattern_index.is_none() || a.len() > b.len() {
        std::cmp::Ordering::Less
    } else if b.len() > a.len() {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Equal
    }
}

fn find_normalized_match_in_object<'a>(
    field: &'a ConditionalMapping,
    request: &'a str,
) -> Option<(&'a MappingValue, &'a str, bool, bool)> {
    assert!(!request.is_empty());
    if !request.contains('*') && !request.ends_with('/') {
        if let Some(target) = field.get(request) {
            return Some((target, "", false, false));
        }
    }
    let mut best_match = "";
    let mut best_match_subpath = None;

    for key in field.keys() {
        if let Some(pattern_index) = key.find('*') {
            let sliced = &key[0..pattern_index];
            if request.starts_with(sliced) {
                let pattern_trailer = &key[pattern_index + 1..];
                if request.len() >= key.len()
                    && request.ends_with(pattern_trailer)
                    && pattern_key_compare(best_match, key).is_gt()
                    && key.rfind('*') == Some(pattern_index)
                {
                    best_match = key;
                    best_match_subpath =
                        Some(&request[pattern_index..request.len() - pattern_trailer.len()]);
                }
            }
        } else if key.ends_with('/')
            && request.starts_with(key)
            && pattern_key_compare(best_match, key).is_gt()
        {
            best_match = key;
            best_match_subpath = Some(&request[key.len()..]);
        }
    }

    best_match_subpath.map(|subpath| {
        let target = &field[best_match];
        let is_subpath_mapping = best_match.ends_with('/');
        let is_pattern = best_match.contains('*');
        (target, subpath, is_subpath_mapping, is_pattern)
    })
}

#[cfg(test)]
mod exports_field_map_test {
    use std::vec;

    use serde_json::json;

    use super::*;
    use crate::test_helper;

    fn process_exports_fields(
        value: serde_json::Value,
        request: &str,
        condition_names: Vec<&str>,
    ) -> RResult<Vec<String>> {
        ExportsField::field_process(&value, request, &test_helper::vec_to_set(condition_names))
    }

    fn should_equal(
        value: serde_json::Value,
        request: &str,
        condition_names: Vec<&str>,
        expected: Vec<&str>,
    ) {
        let actual = process_exports_fields(value, request, condition_names);
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual, expected.into_iter().map(|s| s.to_string()).collect::<Vec<String>>())
    }

    fn should_error(
        value: serde_json::Value,
        request: &str,
        condition_names: Vec<&str>,
        expected_error_message: &str,
    ) {
        let actual = process_exports_fields(value, request, condition_names);
        assert!(actual.is_err());
        let error = actual.unwrap_err();
        match error {
            Error::UnexpectedValue(message) => assert_eq!(expected_error_message, message),
            _ => unreachable!(),
        }
    }

    #[test]
    fn exports_field_map_test_1() {
        should_error(
            json!({
              "/utils/": "./a/"
            }),
            "./utils/index.mjs",
            vec![],
            "Export field key should be relative path and start with \"./\", but got /utils/",
        );
        // --- remove above
        should_equal(
            json!({
                "./utils/": {
                    "webpack": "./wpk/",
                    "browser": ["lodash/", "./utils/"],
                    "node": ["./utils/"]
                }
            }),
            "./utils/index.mjs",
            vec!["browser", "webpack"],
            vec!["./wpk/index.mjs"],
        );
        should_equal(json!("./main.js"), ".", vec![], vec!["./main.js"]);
        should_equal(json!("./main.js"), "./main.js", vec![], vec![]);
        should_equal(json!("./main.js"), "./lib.js", vec![], vec![]);
        should_equal(json!(["./a.js", "./b.js"]), ".", vec![], vec!["./a.js", "./b.js"]);
        should_equal(json!(["./a.js", "./b.js"]), "./a.js", vec![], vec![]);
        should_equal(json!(["./a.js", "./b.js"]), "./lib.js", vec![], vec![]);
        should_equal(
            json!({
              "./a/": "./A/",
              "./a/b/c": "./c.js",
            }),
            "./a/b/d.js",
            vec![],
            vec!["./A/b/d.js"],
        );
        should_equal(
            json!({
              "./a/": "./A/",
              "./a/b": "./b.js",
            }),
            "./a/c.js",
            vec![],
            vec!["./A/c.js"],
        );
        should_equal(
            json!({
              "./a/": "./A/",
              "./a/b/c/d": "./c.js",
            }),
            "./a/b/d/c.js",
            vec![],
            vec!["./A/b/d/c.js"],
        );
        should_equal(
            json!({
              "./a/*": "./A/*",
              "./a/b/c": "./c.js",
            }),
            "./a/b/d.js",
            vec![],
            vec!["./A/b/d.js"],
        );
        should_equal(
            json!({
              "./a/*": "./A/*",
              "./a/b": "./b.js",
            }),
            "./a/c.js",
            vec![],
            vec!["./A/c.js"],
        );
        should_equal(
            json!({
               "./a/*": "./A/*",
               "./a/b/c/d": "./b.js",
            }),
            "./a/b/d/c.js",
            vec![],
            vec!["./A/b/d/c.js"],
        );
        should_equal(
            json!({
              "./ab*": "./ab/*",
              "./abc*": "./abc/*",
              "./a*": "./a/*",
            }),
            "./abcd",
            vec!["browser"],
            vec!["./abc/d"],
        );
        should_equal(
            json!({
              "./ab*": "./ab/*",
              "./abc*": "./abc/*",
              "./a*": "./a/*",
            }),
            "./abcd",
            vec![],
            vec!["./abc/d"],
        );
        should_equal(
            json!({
              "./ab*": "./ab/*",
              "./abc*": "./abc/*",
              "./a*": "./a/*",
            }),
            "./abcd/e",
            vec!["browser"],
            vec!["./abc/d/e"],
        );
        should_equal(
            json!({
              "./x/ab*": "./ab/*",
              "./x/abc*": "./abc/*",
              "./x/a*": "./a/*",
            }),
            "./x/abcd",
            vec!["browser"],
            vec!["./abc/d"],
        );
        should_equal(
            json!({
              "./x/ab*": "./ab/*",
              "./x/abc*": "./abc/*",
              "./x/a*": "./a/*",
            }),
            "./x/abcd/e",
            vec!["browser"],
            vec!["./abc/d/e"],
        );
        should_equal(
            json!({
                "browser": {
                    "default": "./index.js"
                }
            }),
            "./lib.js",
            vec!["browser"],
            vec![],
        );
        should_equal(
            json!({
                "browser": {
                    "default": "./index.js"
                }
            }),
            ".",
            vec!["browser"],
            vec!["./index.js"],
        );
        should_equal(
            json!({
                "./foo/": {
                    "import": ["./dist/", "./src/"],
                    "webpack": "./wp/"
                },
                ".": "./main.js"
            }),
            "./foo/test/file.js",
            vec!["import", "webpack"],
            vec!["./dist/test/file.js", "./src/test/file.js"],
        );
        should_equal(
            json!({
                "./foo/*": {
                    "import": ["./dist/*", "./src/*"],
                    "webpack": "./wp/*"
                },
                ".": "./main.js"
            }),
            "./foo/test/file.js",
            vec!["import", "webpack"],
            vec!["./dist/test/file.js", "./src/test/file.js"],
        );
        should_equal(
            json!({
                "./timezones/": "./data/timezones/"
            }),
            "./timezones/pdt.mjs",
            vec![],
            vec!["./data/timezones/pdt.mjs"],
        );
        should_equal(
            json!({
                "./": "./data/timezones/"
            }),
            "./timezones/pdt.mjs",
            vec![],
            vec!["./data/timezones/timezones/pdt.mjs"],
        );
        should_equal(
            json!({
                "./*": "./data/timezones/*.mjs"
            }),
            "./timezones/pdt",
            vec![],
            vec!["./data/timezones/timezones/pdt.mjs"],
        );
        should_equal(
            json!({
                "./lib/": {
                    "browser": ["./browser/"]
                },
                "./dist/index.js": {
                    "node": "./index.js"
                }
            }),
            "./dist/index.js",
            vec!["browser"],
            vec![],
        );
        should_equal(
            json!({
                "./lib/*": {
                    "browser": ["./browser/*"]
                },
                "./dist/index.js": {
                    "node": "./index.js"
                }
            }),
            "./dist/index.js",
            vec!["browser"],
            vec![],
        );
        should_equal(
            json!({
                "./lib/": {
                    "browser": ["./browser/"]
                },
                "./dist/index.js": {
                    "node": "./index.js",
                    "default": "./browser/index.js"
                }
            }),
            "./dist/index.js",
            vec!["browser"],
            vec!["./browser/index.js"],
        );
        should_equal(
            json!({
                "./lib/*": {
                    "browser": ["./browser/*"]
                },
                "./dist/index.js": {
                    "node": "./index.js",
                    "default": "./browser/index.js"
                }
            }),
            "./dist/index.js",
            vec!["browser"],
            vec!["./browser/index.js"],
        );
        should_equal(
            json!({
                "./dist/a": "./dist/index.js"
            }),
            "./dist/aaa",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                "./dist/a/a/": "./dist/index.js"
            }),
            "./dist/a",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                "./dist/a/a/*": "./dist/index.js"
            }),
            "./dist/a",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                ".": "./index.js"
            }),
            "./timezones/pdt.mjs",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                "./index.js": "./main.js"
            }),
            "./index.js",
            vec![],
            vec!["./main.js"],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./#foo",
            vec![],
            vec!["./ok.js"],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./bar#foo",
            vec![],
            vec!["./ok.js"],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./#zapp/ok.js#abc",
            vec![],
            vec!["./ok.js#abc"],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./#zapp/ok.js#abc",
            vec![],
            vec!["./ok.js#abc"],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./#zapp/ok.js?abc",
            vec![],
            vec!["./ok.js?abc"],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./#zapp/🎉.js",
            vec![],
            vec!["./🎉.js"],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./#zapp/%F0%9F%8E%89.js",
            vec![],
            vec!["./%F0%9F%8E%89.js"],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./🎉",
            vec![],
            vec!["./ok.js"],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./%F0%9F%8E%89",
            vec![],
            vec!["./other.js"],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./module",
            vec![],
            vec!["./ok.js"],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./module#foo",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            }),
            "./module?foo",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./z*z*z*"
            }),
            "./#zippi",
            vec![],
            vec!["./zizizi"],
        );
        should_equal(
            json!({
                "./a?b?c/": "./"
            }),
            "./a?b?c/d?e?f",
            vec![],
            vec!["./d?e?f"],
        );
        should_equal(
            json!({
                ".": "./dist/index.js"
            }),
            ".",
            vec![],
            vec!["./dist/index.js"],
        );
        should_equal(
            json!({
                "./": "./",
                "./*": "./*",
                "./dist/index.js": "./dist/index.js",
            }),
            ".",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                "./dist/": "./dist/",
                "./dist/*": "./dist/*",
                "./dist*": "./dist*",
                "./dist/index.js": "./dist/a.js"
            }),
            "./dist/index.js",
            vec![],
            vec!["./dist/a.js"],
        );
        should_equal(
            json!({
                "./": {
                    "browser": ["./browser/"]
                },
                "./*": {
                    "browser": ["./browser/*"]
                },
                "./dist/index.js": {
                    "browser": "./index.js"
                },
            }),
            "./dist/index.js",
            vec!["browser"],
            vec!["./index.js"],
        );
        should_equal(
            json!({
                "./a?b?c/": "./"
            }),
            "./a?b?c/d?e?f",
            vec![],
            vec!["./d?e?f"],
        );
        should_equal(
            json!({
                "./": {
                    "browser": ["./browser/"]
                },
                "./*": {
                    "browser": ["./browser/*"]
                },
                "./dist/index.js": {
                    "node": "./node.js"
                },
            }),
            "./dist/index.js",
            vec!["browser"],
            vec![],
        );
        should_equal(
            json!({
                ".": {
                    "browser": "./index.js",
                    "node": "./src/node/index.js",
                    "default": "./src/index.js"
                }
            }),
            ".",
            vec!["browser"],
            vec!["./index.js"],
        );
        should_equal(
            json!({
                ".": {
                    "browser": "./index.js",
                    "node": "./src/node/index.js",
                    "default": "./src/index.js"
                }
            }),
            ".",
            vec![],
            vec!["./src/index.js"],
        );
        should_equal(
            json!({
                ".": "./index"
            }),
            ".",
            vec![],
            vec!["./index"],
        );
        should_equal(
            json!({
                "./index": "./index.js"
            }),
            "./index",
            vec![],
            vec!["./index.js"],
        );
        should_equal(
            json!({
                ".": [
                    { "browser": "./browser.js" },
                    { "require": "./require.js" },
                    { "import": "./import.mjs" }
                ]
            }),
            ".",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                ".": [
                    { "browser": "./browser.js" },
                    { "require": "./require.js" },
                    { "import": "./import.mjs" }
                ]
            }),
            ".",
            vec!["import"],
            vec!["./import.mjs"],
        );
        should_equal(
            json!({
                ".": [
                    { "browser": "./browser.js" },
                    { "require": "./require.js" },
                    { "import": "./import.mjs" }
                ]
            }),
            ".",
            vec!["import", "require"],
            vec!["./require.js", "./import.mjs"],
        );
        should_equal(
            json!({
                ".": [
                    { "browser": "./browser.js" },
                    { "require": "./require.js" },
                    { "import": ["./import.mjs", "./import.js"] }
                ]
            }),
            ".",
            vec!["import", "require"],
            vec!["./require.js", "./import.mjs", "./import.js"],
        );
        should_equal(
            json!({
                "./timezones": "./data/timezones",
            }),
            "./timezones/pdt.mjs",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                "./timezones/pdt/": "./data/timezones/pdt/",
            }),
            "./timezones/pdt/index.mjs",
            vec![],
            vec!["./data/timezones/pdt/index.mjs"],
        );
        should_equal(
            json!({
                "./timezones/pdt/*": "./data/timezones/pdt/*",
            }),
            "./timezones/pdt/index.mjs",
            vec![],
            vec!["./data/timezones/pdt/index.mjs"],
        );
        should_equal(
            json!({
                "./": "./timezones/",
            }),
            "./pdt.mjs",
            vec![],
            vec!["./timezones/pdt.mjs"],
        );
        should_equal(
            json!({
                "./*": "./timezones/*",
            }),
            "./pdt.mjs",
            vec![],
            vec!["./timezones/pdt.mjs"],
        );
        should_equal(
            json!({
                "./": "./",
            }),
            "./timezones/pdt.mjs",
            vec![],
            vec!["./timezones/pdt.mjs"],
        );
        should_equal(
            json!({
                "./*": "./*",
            }),
            "./timezones/pdt.mjs",
            vec![],
            vec!["./timezones/pdt.mjs"],
        );
        should_equal(
            json!({
                ".": "./",
            }),
            "./timezones/pdt.mjs",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                ".": "./*",
            }),
            "./timezones/pdt.mjs",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                "./": "./",
                "./dist/": "./lib/"
            }),
            "./dist/index.mjs",
            vec![],
            vec!["./lib/index.mjs"],
        );
        should_equal(
            json!({
                "./*": "./*",
                "./dist/*": "./lib/*"
            }),
            "./dist/index.mjs",
            vec![],
            vec!["./lib/index.mjs"],
        );
        should_equal(
            json!({
                "./dist/utils/": "./dist/utils/",
                "./dist/": "./lib/"
            }),
            "./dist/utils/index.js",
            vec![],
            vec!["./dist/utils/index.js"],
        );
        should_equal(
            json!({
                "./dist/utils/*": "./dist/utils/*",
                "./dist/*": "./lib/*"
            }),
            "./dist/utils/index.js",
            vec![],
            vec!["./dist/utils/index.js"],
        );
        should_equal(
            json!({
                "./dist/utils/index.js": "./dist/utils/index.js",
                "./dist/utils/": "./dist/utils/index.mjs",
                "./dist/": "./lib/"
            }),
            "./dist/utils/index.js",
            vec![],
            vec!["./dist/utils/index.js"],
        );
        should_equal(
            json!({
                "./dist/utils/index.js": "./dist/utils/index.js",
                "./dist/utils/*": "./dist/utils/index.mjs",
                "./dist/*": "./lib/*"
            }),
            "./dist/utils/index.js",
            vec![],
            vec!["./dist/utils/index.js"],
        );
        should_equal(
            json!({
                "./": {
                    "browser": "./browser/"
                },
                "./dist/": "./lib/"
            }),
            "./dist/index.mjs",
            vec!["browser"],
            vec!["./lib/index.mjs"],
        );
        should_equal(
            json!({
                "./*": {
                    "browser": "./browser/*"
                },
                "./dist/*": "./lib/*"
            }),
            "./dist/index.mjs",
            vec!["browser"],
            vec!["./lib/index.mjs"],
        );
        should_equal(
            json!({
                "./utils/": {
                    "browser": ["lodash/", "./utils/"],
                    "node": ["./utils-node/"]
                }
            }),
            "./utils/index.js",
            vec!["browser"],
            vec!["lodash/index.js", "./utils/index.js"],
        );
        should_equal(
            json!({
                "./utils/*": {
                    "browser": ["lodash/*", "./utils/*"],
                    "node": ["./utils-node/*"]
                }
            }),
            "./utils/index.js",
            vec!["browser"],
            vec!["lodash/index.js", "./utils/index.js"],
        );
        should_equal(
            json!({
                "./utils/": {
                    "webpack": "./wpk/",
                    "browser": ["lodash/", "./utils/"],
                    "node": ["./node/"]
                }
            }),
            "./utils/index.mjs",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                "./utils/*": {
                    "webpack": "./wpk/*",
                    "browser": ["lodash/*", "./utils/*"],
                    "node": ["./node/*"]
                }
            }),
            "./utils/index.mjs",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                "./utils/": {
                    "webpack": "./wpk/",
                    "browser": ["lodash/", "./utils/"],
                    "node": ["./utils/"]
                }
            }),
            "./utils/index.mjs",
            vec!["browser", "webpack"],
            vec!["./wpk/index.mjs"],
        );
        should_equal(
            json!({
                "./utils/*": {
                    "webpack": "./wpk/*",
                    "browser": ["lodash/*", "./utils/*"],
                    "node": ["./utils/*"]
                }
            }),
            "./utils/index.mjs",
            vec!["browser", "webpack"],
            vec!["./wpk/index.mjs"],
        );
        should_equal(
            json!({
              "./utils/index": "./a/index.js"
            }),
            "./utils/index.mjs",
            vec![],
            vec![],
        );
        should_equal(
            json!({
              "./utils/index.mjs": "./a/index.js"
            }),
            "./utils/index",
            vec![],
            vec![],
        );
        should_equal(
            json!({
              "./utils/index": {
                  "browser": "./a/index.js",
                  "default": "./b/index.js",
              }
            }),
            "./utils/index.mjs",
            vec!["browser"],
            vec![],
        );
        should_equal(
            json!({
              "./utils/index.mjs": {
                  "browser": "./a/index.js",
                  "default": "./b/index.js",
              }
            }),
            "./utils/index",
            vec!["browser"],
            vec![],
        );
        should_equal(
            json!({
              "./../../utils/": "./dist/"
            }),
            "./../../utils/index",
            vec![],
            vec!["./dist/index"],
        );
        should_equal(
            json!({
              "./../../utils/*": "./dist/*"
            }),
            "./../../utils/index",
            vec![],
            vec!["./dist/index"],
        );
        should_equal(
            json!({
              "./utils/": "./../src/"
            }),
            "./utils/index",
            vec![],
            vec!["./../src/index"],
        );
        should_equal(
            json!({
              "./utils/*": "./../src/*"
            }),
            "./utils/index",
            vec![],
            vec!["./../src/index"],
        );
        should_equal(
            json!({
              "./utils/index": "./src/../index.js"
            }),
            "./utils/index",
            vec![],
            vec!["./src/../index.js"],
        );
        should_equal(
            json!({
              "./utils/../utils/index": "./src/../index.js"
            }),
            "./utils/index",
            vec![],
            vec![],
        );
        should_equal(
            json!({
                "./utils/": {
                    "browser": "./utils/../"
                }
            }),
            "./utils/index",
            vec!["browser"],
            vec!["./utils/../index"],
        );
        should_equal(
            json!({
                "./": "./src/../../",
                "./dist/": "./dist/"
            }),
            "./dist/index",
            vec!["browser"],
            vec!["./dist/index"],
        );
        should_equal(
            json!({
                "./*": "./src/../../*",
                "./dist/*": "./dist/*"
            }),
            "./dist/index",
            vec!["browser"],
            vec!["./dist/index"],
        );
        should_equal(
            json!({
                "./utils/": "./dist/"
            }),
            "./utils/timezone/../../index",
            vec![],
            vec!["./dist/timezone/../../index"],
        );
        should_equal(
            json!({
                "./utils/*": "./dist/*"
            }),
            "./utils/timezone/../../index",
            vec![],
            vec!["./dist/timezone/../../index"],
        );
        should_equal(
            json!({
                "./utils/": "./dist/"
            }),
            "./utils/timezone/../index",
            vec![],
            vec!["./dist/timezone/../index"],
        );
        should_equal(
            json!({
                "./utils/*": "./dist/*"
            }),
            "./utils/timezone/../index",
            vec![],
            vec!["./dist/timezone/../index"],
        );
        should_equal(
            json!({
                "./utils/": "./dist/target/"
            }),
            "./utils/../../index",
            vec![],
            vec!["./dist/target/../../index"],
        );
        should_equal(
            json!({
                "./utils/*": "./dist/target/*"
            }),
            "./utils/../../index",
            vec![],
            vec!["./dist/target/../../index"],
        );
        should_equal(
            json!({
                "./utils/": {
                    "browser": "./node_modules/"
                }
            }),
            "./utils/lodash/dist/index.js",
            vec!["browser"],
            vec!["./node_modules/lodash/dist/index.js"],
        );
        should_equal(
            json!({
                "./utils/*": {
                    "browser": "./node_modules/*"
                }
            }),
            "./utils/lodash/dist/index.js",
            vec!["browser"],
            vec!["./node_modules/lodash/dist/index.js"],
        );
        should_equal(
            json!({
                "./utils/": "./utils/../node_modules/"
            }),
            "./utils/lodash/dist/index.js",
            vec!["browser"],
            vec!["./utils/../node_modules/lodash/dist/index.js"],
        );
        should_equal(
            json!({
                "./utils/*": "./utils/../node_modules/*"
            }),
            "./utils/lodash/dist/index.js",
            vec!["browser"],
            vec!["./utils/../node_modules/lodash/dist/index.js"],
        );
        should_equal(
            json!({
                "./utils/": {
                    "browser": {
                        "webpack": "./",
                        "default": {
                            "node": "./node/"
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["browser"],
            vec![],
        );
        should_equal(
            json!({
                "./utils/*": {
                    "browser": {
                        "webpack": "./*",
                        "default": {
                            "node": "./node/*"
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["browser"],
            vec![],
        );
        should_equal(
            json!({
                "./utils/": {
                    "browser": {
                        "webpack": ["./", "./node/"],
                        "default": {
                            "node": "./node/"
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["browser", "webpack"],
            vec!["./index.js", "./node/index.js"],
        );
        should_equal(
            json!({
                "./utils/*": {
                    "browser": {
                        "webpack": ["./*", "./node/*"],
                        "default": {
                            "node": "./node/*"
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["browser", "webpack"],
            vec!["./index.js", "./node/index.js"],
        );
        should_equal(
            json!({
                "./utils/": {
                    "browser": {
                        "webpack": ["./", "./node/"],
                        "default": {
                            "node": "./node/"
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["webpack"],
            vec![],
        );
        should_equal(
            json!({
                "./utils/*": {
                    "browser": {
                        "webpack": ["./*", "./node/*"],
                        "default": {
                            "node": "./node/*"
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["webpack"],
            vec![],
        );
        should_equal(
            json!({
                "./utils/": {
                    "browser": {
                        "webpack": ["./", "./node/"],
                        "default": {
                            "node": "./node/"
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["node", "browser"],
            vec!["./node/index.js"],
        );
        should_equal(
            json!({
                "./utils/*": {
                    "browser": {
                        "webpack": ["./*", "./node/*"],
                        "default": {
                            "node": "./node/*"
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["node", "browser"],
            vec!["./node/index.js"],
        );
        should_equal(
            json!({
                "./utils/": {
                    "browser": {
                        "webpack": ["./", "./node/"],
                        "default": {
                            "node": {
                                "webpack": ["./wpck/"]
                            }
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["browser", "node"],
            vec![],
        );
        should_equal(
            json!({
                "./utils/*": {
                    "browser": {
                        "webpack": ["./*", "./node/*"],
                        "default": {
                            "node": {
                                "webpack": ["./wpck/*"]
                            }
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["browser", "node"],
            vec![],
        );
        should_equal(
            json!({
                "./utils/": {
                    "browser": {
                        "webpack": ["./", "./node/"],
                        "default": {
                            "node": {
                                "webpack": ["./wpck/"]
                            }
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["browser", "node", "webpack"],
            vec!["./index.js", "./node/index.js"],
        );
        should_equal(
            json!({
                "./utils/*": {
                    "browser": {
                        "webpack": ["./*", "./node/*"],
                        "default": {
                            "node": {
                                "webpack": ["./wpck/*"]
                            }
                        }
                    }
                }
            }),
            "./utils/index.js",
            vec!["browser", "node", "webpack"],
            vec!["./index.js", "./node/index.js"],
        );
        should_equal(
            json!({
                "./a.js": {
                    "abc": {
                        "def": "./x.js"
                    },
                    "ghi": "./y.js"
                }
            }),
            "./a.js",
            vec!["abc", "ghi"],
            vec!["./y.js"],
        );
        should_equal(
            json!({
                "./a.js": {
                    "abc": {
                        "def": "./x.js",
                        "default": []
                    },
                    "ghi": "./y.js"
                }
            }),
            "./a.js",
            vec!["abc", "ghi"],
            vec![],
        );

        should_error(
            json!({
                "./utils/": {
                    "browser": "../this/"
                }
            }),
            "./utils/index",
            vec!["browser"],
            "Export should be relative path and start with \"./\", but got ../this/",
        );
        should_error(
            json!({
                "./utils/*": {
                    "browser": "../this/*"
                }
            }),
            "./utils/index",
            vec!["browser"],
            "Export should be relative path and start with \"./\", but got ../this/*",
        );
        should_error(
            json!({
              ".": {
                  "default": "./src/index.js",
                  "browser": "./index.js",
                  "node": "./src/node/index.js"
              }
            }),
            ".",
            vec!["browser"],
            "Default condition should be last one",
        );
        should_error(
            json!({
                "./*": "."
            }),
            "./timezones/pdt.mjs",
            vec![],
            "Export should be relative path and start with \"./\", but got .",
        );
        should_error(
            json!({
                "./": "."
            }),
            "./timezones/pdt.mjs",
            vec![],
            "Export should be relative path and start with \"./\", but got .",
        );
        should_error(
            json!({
                "./timezones/": "./data/timezones"
            }),
            "./timezones/pdt.mjs",
            vec![],
            "Expected ./data/timezones is folder mapping",
        );
        should_error(
            json!({
              "./node": "./node.js",
              "browser": {
                "default": "./index.js"
              },
            }),
            ".",
            vec!["browser"],
            "Export field key should be relative path and start with \".\", but got browser",
        );
        should_error(
            json!({
              "browser": {
                "default": "./index.js"
              },
              "./node": "./node.js"
            }),
            ".",
            vec!["browser"],
            "Export field key should be relative path and start with \"./\", but got ./node",
        );
        should_error(
            json!({
              "/utils/": "./a/"
            }),
            "./utils/index.mjs",
            vec![],
            "Export field key should be relative path and start with \"./\", but got /utils/",
        );
        should_error(
            json!({
              "./utils/": "/a/"
            }),
            "./utils/index.mjs",
            vec![],
            "Export should be relative path and start with \"./\", but got /a/",
        );
        should_error(
            json!({
              "./utils/": "./a/"
            }),
            "/utils/index.mjs",
            vec![],
            "Request should be relative path and start with '.', but got /utils/index.mjs",
        );
        should_error(
            json!({
              "./utils/": {
                  "browser": "./a/",
                  "default": "./b/"
              }
            }),
            "/utils/index.mjs",
            vec!["browser"],
            "Request should be relative path and start with '.', but got /utils/index.mjs",
        );
        should_error(
            json!({
              "./utils/": {
                  "browser": "./a/",
                  "default": "./b/"
              }
            }),
            "/utils/index.mjs/",
            vec!["browser"],
            "Request should be relative path and start with '.', but got /utils/index.mjs/",
        );
        should_error(
            json!({
              "./utils/": {
                  "browser": "./a/",
                  "default": "./b/"
              }
            }),
            "../utils/index.mjs",
            vec!["browser"],
            "Request should be relative path and start with '.', but got ../utils/index.mjs",
        );
        should_error(
            json!({
                "../../utils/": "./dist/"
            }),
            "../../utils/index",
            vec![],
            "Request should be relative path and start with '.', but got ../../utils/index",
        );
        should_error(
            json!({
                "../../utils/*": "./dist/*"
            }),
            "./utils/index",
            vec![],
            "Export field key should be relative path and start with \"./\", but got ../../utils/*",
        );
        should_error(
            json!({
                "./utils/*": "./dist/*"
            }),
            "../../utils/index",
            vec![],
            "Request should be relative path and start with '.', but got ../../utils/index",
        );
        should_error(
            json!({
                "./utils/": "../src/"
            }),
            "./utils/index",
            vec![],
            "Export should be relative path and start with \"./\", but got ../src/",
        );
        should_error(
            json!({
                "./utils/*": "../src/*"
            }),
            "./utils/index",
            vec![],
            "Export should be relative path and start with \"./\", but got ../src/*",
        );
        should_error(
            json!({
                "/utils/": {
                    "browser": "./a/",
                    "default": "./b/"
                }
            }),
            "./utils/index.mjs",
            vec!["browser"],
            "Export field key should be relative path and start with \"./\", but got /utils/",
        );
        should_error(
            json!({
                "./utils/": {
                    "browser": "/a/",
                    "default": "/b/"
                }
            }),
            "./utils/index.mjs",
            vec!["browser"],
            "Export should be relative path and start with \"./\", but got /a/",
        );
        should_error(
            json!({
                "./utils/*": {
                    "browser": "/a/",
                    "default": "/b/"
                }
            }),
            "./utils/index.mjs",
            vec!["browser"],
            "Export should be relative path and start with \"./\", but got /a/",
        );
    }

    #[test]
    fn exports_field_map_test_2() {
        // copy from node
        // https://github.com/nodejs/node/blob/main/test/fixtures/node_modules/pkgexports/package.json
        let value = || {
            json!({
              "./hole": "./lib/hole.js",
              "./space": "./sp%20ce.js",
              "./valid-cjs": "./asdf.js",
              "./sub/*": "./*",
              "./sub/internal/*": null,
              "./belowdir/*": "../belowdir/*",
              "./belowfile": "../belowfile",
              "./null": null,
              "./null/": null,
              "./invalid1": {},
              "./invalid2": 1234,
              "./invalid3": "",
              "./invalid4": {},
              "./invalid5": "invalid5.js",
              "./fallbackdir/*": [[], null, {}, "builtin:x/*", "./*"],
              "./fallbackfile": [[], null, {}, "builtin:x", "./asdf.js"],
              "./nofallback1": [],
              "./nofallback2": [null, {}, "builtin:x"],
              "./nodemodules": "./node_modules/internalpkg/x.js",
              "./doubleslash": ".//asdf.js",
              "./no-addons": {
                "node-addons": "./addons-entry.js",
                "default": "./no-addons-entry.js"
              },
              "./condition": [{
                "custom-condition": {
                  "import": "./custom-condition.mjs",
                  "require": "./custom-condition.js"
                },
                "import": "///overridden",
                "require": {
                  "require": {
                    "nomatch": "./nothing.js"
                  },
                  "default": "./sp ce.js"
                },
                "default": "./asdf.js",
                "node": "./lib/hole.js",
                "import": {
                  "nomatch": "./nothing.js"
                }
              }],
              "./no-ext": "./asdf",
              "./resolve-self": {
                "require": "./resolve-self.js",
                "import": "./resolve-self.mjs"
              },
              "./resolve-self-invalid": {
                "require": "./resolve-self-invalid.js",
                "import": "./resolve-self-invalid.mjs"
              },
              "./*/trailer": "./subpath/*.js",
              "./*/*railer": "never",
              "./*trailer": "never",
              "./*/dir2/trailer": "./subpath/*/index.js",
              "./a/*": "./subpath/*.js",
              "./a/b/": "./nomatch/",
              "./a/b*": "./subpath*.js",
              "./subpath/*": "./subpath/*",
              "./subpath/sub-*": "./subpath/dir1/*.js",
              "./subpath/sub-*.js": "./subpath/dir1/*.js",
              "./features/*": "./subpath/*/*.js",
              "./trailing-pattern-slash*": "./trailing-pattern-slash*index.js"
            })
        };

        should_equal(value(), "./subpath/sub-dir1.js", vec![], vec!["./subpath/dir1/dir1.js"]);
        should_equal(value(), "./valid-cjs", vec![], vec!["./asdf.js"]);
        should_equal(value(), "./space", vec![], vec!["./sp%20ce.js"]);
        should_equal(
            value(),
            "./fallbackdir/asdf.js",
            vec![],
            vec!["builtin:x/asdf.js", "./asdf.js"],
        );
        should_equal(value(), "./fallbackfile", vec![], vec!["builtin:x", "./asdf.js"]);
        should_equal(value(), "./condition", vec!["require"], vec!["./sp ce.js"]);
        should_equal(value(), "./resolve-self", vec!["require"], vec!["./resolve-self.js"]);
        should_equal(value(), "./resolve-self", vec!["import"], vec!["./resolve-self.mjs"]);
        should_equal(value(), "./subpath/sub-dir1", vec![], vec!["./subpath/dir1/dir1.js"]);
        should_equal(value(), "./subpath/sub-dir1.js", vec![], vec!["./subpath/dir1/dir1.js"]);
        should_equal(value(), "./features/dir1", vec![], vec!["./subpath/dir1/dir1.js"]);
        should_equal(value(), "./dir1/dir1/trailer", vec![], vec!["./subpath/dir1/dir1.js"]);
        should_equal(value(), "./dir2/trailer", vec![], vec!["./subpath/dir2.js"]);
        should_equal(value(), "./dir2/dir2/trailer", vec![], vec!["./subpath/dir2/index.js"]);
        should_equal(value(), "./a/dir1/dir1", vec![], vec!["./subpath/dir1/dir1.js"]);
        should_equal(value(), "./a/b/dir1/dir1", vec![], vec!["./subpath/dir1/dir1.js"]);
        // FIXME:
        // should_equal(
        //     value(),
        //     "./a//dir1/dir1",
        //     vec![],
        //     vec!["./subpath/dir1/dir1.js"],
        // );
        //  FIXME:
        // should_equal(value(), "./doubleslash", vec![], vec!["./asdf.js"]);
        should_equal(value(), "./sub/no-a-file.js", vec![], vec!["./no-a-file.js"]);
        should_equal(value(), "./sub/internal/test.js", vec![], vec![]);
        // FIXME:
        // should_equal(
        //     value(),
        //     "./sub//internal/test.js",
        //     vec![],
        //     vec!["./internal/test.js"],
        // );
        should_equal(
            //1
            value(),
            "./trailing-pattern-slash/",
            vec![],
            vec!["./trailing-pattern-slash/index.js"],
        );
    }
}

#[test]
fn imports_field_map_test() {
    use serde_json::json;

    use crate::test_helper;

    fn process_imports_fields(
        value: serde_json::Value,
        request: &str,
        condition_names: Vec<&str>,
    ) -> RResult<Vec<String>> {
        ImportsField::field_process(&value, request, &test_helper::vec_to_set(condition_names))
    }

    fn should_equal(
        value: serde_json::Value,
        request: &str,
        condition_names: Vec<&str>,
        expected: Vec<&str>,
    ) {
        let actual = process_imports_fields(value, request, condition_names);
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual, expected.into_iter().map(|s| s.to_string()).collect::<Vec<String>>())
    }

    fn should_error(
        value: serde_json::Value,
        request: &str,
        condition_names: Vec<&str>,
        expected_error_message: &str,
    ) {
        let actual = process_imports_fields(value, request, condition_names);
        assert!(actual.is_err());
        let error = actual.unwrap_err();
        match error {
            Error::UnexpectedValue(message) => assert_eq!(expected_error_message, message),
            _ => unreachable!(),
        }
    }

    should_equal(
        json!({
            "#abc/": {
                "import": ["./dist/", "./src/"],
                "webpack": "./wp/"
            },
            "#abc": "./main.js"
        }),
        "#abc/test/file.js",
        vec!["import", "webpack"],
        vec!["./dist/test/file.js", "./src/test/file.js"],
    );
    should_equal(
        json!({
            "#1/timezones/": "./data/timezones/"
        }),
        "#1/timezones/pdt.mjs",
        vec![],
        vec!["./data/timezones/pdt.mjs"],
    );
    should_equal(
        json!({
            "#aaa/": "./data/timezones/",
            "#a/": "./data/timezones/"
        }),
        "#a/timezones/pdt.mjs",
        vec![],
        vec!["./data/timezones/timezones/pdt.mjs"],
    );
    should_equal(
        json!({
            "#a/lib/": {
                "browser": ["./browser/"]
            },
            "#a/dist/index.js": {
                "node": "./index.js"
            }
        }),
        "#a/dist/index.js",
        vec!["browser"],
        vec![],
    );
    should_equal(
        json!({
            "#a/lib/": {
                "browser": ["./browser/"]
            },
            "#a/dist/index.js": {
                "node": "./index.js",
                "default": "./browser/index.js"
            }
        }),
        "#a/dist/index.js",
        vec!["browser"],
        vec!["./browser/index.js"],
    );
    should_equal(
        json!({
            "#a/dist/a": "./dist/index.js",
        }),
        "#a/dist/aaa",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a/a/a/": "./dist/index.js",
        }),
        "#a/a/a",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a/a/a/": "./dist/index.js",
        }),
        "#a/a/a",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a": "./index.js",
        }),
        "#a/timezones/pdt.mjs",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a/index.js": "./main.js",
        }),
        "#a/index.js",
        vec![],
        vec!["./main.js"],
    );
    should_equal(
        json!({
            "#a/#foo": "./ok.js",
            "#a/module": "./ok.js",
            "#a/🎉": "./ok.js",
            "#a/%F0%9F%8E%89": "./other.js",
            "#a/bar#foo": "./ok.js",
            "#a/#zapp/": "./"
        }),
        "#a/#foo",
        vec![],
        vec!["./ok.js"],
    );
    should_equal(
        json!({
            "#a/#foo": "./ok.js",
            "#a/module": "./ok.js",
            "#a/🎉": "./ok.js",
            "#a/%F0%9F%8E%89": "./other.js",
            "#a/bar#foo": "./ok.js",
            "#a/#zapp/": "./"
        }),
        "#a/bar#foo",
        vec![],
        vec!["./ok.js"],
    );
    should_equal(
        json!({
            "#a/#foo": "./ok.js",
            "#a/module": "./ok.js",
            "#a/🎉": "./ok.js",
            "#a/%F0%9F%8E%89": "./other.js",
            "#a/bar#foo": "./ok.js",
            "#a/#zapp/": "./"
        }),
        "#a/#zapp/ok.js#abc",
        vec![],
        vec!["./ok.js#abc"],
    );
    should_equal(
        json!({
            "#a/#foo": "./ok.js",
            "#a/module": "./ok.js",
            "#a/🎉": "./ok.js",
            "#a/%F0%9F%8E%89": "./other.js",
            "#a/bar#foo": "./ok.js",
            "#a/#zapp/": "./"
        }),
        "#a/#zapp/ok.js?abc",
        vec![],
        vec!["./ok.js?abc"],
    );
    should_equal(
        json!({
            "#a/#foo": "./ok.js",
            "#a/module": "./ok.js",
            "#a/🎉": "./ok.js",
            "#a/%F0%9F%8E%89": "./other.js",
            "#a/bar#foo": "./ok.js",
            "#a/#zapp/": "./"
        }),
        "#a/#zapp/🎉.js",
        vec![],
        vec!["./🎉.js"],
    );
    should_equal(
        json!({
            "#a/#foo": "./ok.js",
            "#a/module": "./ok.js",
            "#a/🎉": "./ok.js",
            "#a/%F0%9F%8E%89": "./other.js",
            "#a/bar#foo": "./ok.js",
            "#a/#zapp/": "./"
        }),
        "#a/#zapp/%F0%9F%8E%89.js",
        vec![],
        vec!["./%F0%9F%8E%89.js"],
    );
    should_equal(
        json!({
            "#a/#foo": "./ok.js",
            "#a/module": "./ok.js",
            "#a/🎉": "./ok.js",
            "#a/%F0%9F%8E%89": "./other.js",
            "#a/bar#foo": "./ok.js",
            "#a/#zapp/": "./"
        }),
        "#a/🎉",
        vec![],
        vec!["./ok.js"],
    );
    should_equal(
        json!({
            "#a/#foo": "./ok.js",
            "#a/module": "./ok.js",
            "#a/🎉": "./ok.js",
            "#a/%F0%9F%8E%89": "./other.js",
            "#a/bar#foo": "./ok.js",
            "#a/#zapp/": "./"
        }),
        "#a/%F0%9F%8E%89",
        vec![],
        vec!["./other.js"],
    );
    should_equal(
        json!({
            "#a/#foo": "./ok.js",
            "#a/module": "./ok.js",
            "#a/🎉": "./ok.js",
            "#a/%F0%9F%8E%89": "./other.js",
            "#a/bar#foo": "./ok.js",
            "#a/#zapp/": "./"
        }),
        "#a/module",
        vec![],
        vec!["./ok.js"],
    );
    should_equal(
        json!({
            "#a/#foo": "./ok.js",
            "#a/module": "./ok.js",
            "#a/🎉": "./ok.js",
            "#a/%F0%9F%8E%89": "./other.js",
            "#a/bar#foo": "./ok.js",
            "#a/#zapp/": "./"
        }),
        "#a/module#foo",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a/#foo": "./ok.js",
            "#a/module": "./ok.js",
            "#a/🎉": "./ok.js",
            "#a/%F0%9F%8E%89": "./other.js",
            "#a/bar#foo": "./ok.js",
            "#a/#zapp/": "./"
        }),
        "#a/module?foo",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a/a?b?c/": "./"
        }),
        "#a/a?b?c/d?e?f",
        vec![],
        vec!["./d?e?f"],
    );
    should_equal(
        json!({
            "#a/": "/user/a/"
        }),
        "#a/index",
        vec![],
        vec!["/user/a/index"],
    );
    should_equal(
        json!({
            "#a/": "./A/",
            "#a/b/c": "./c.js"
        }),
        "#a/b/d.js",
        vec![],
        vec!["./A/b/d.js"],
    );
    should_equal(
        json!({
            "#a/": "./A/",
            "#a/b": "./b.js"
        }),
        "#a/c.js",
        vec![],
        vec!["./A/c.js"],
    );
    should_equal(
        json!({
            "#a/": "./A/",
            "#a/b/c/d": "./c.js"
        }),
        "#a/b/c/d.js",
        vec![],
        vec!["./A/b/c/d.js"],
    );
    should_equal(
        json!({
            "#a": "./dist/index.js"
        }),
        "#a",
        vec![],
        vec!["./dist/index.js"],
    );
    should_equal(
        json!({
            "#a/": "./"
        }),
        "#a",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a/": "./dist/",
            "#a/index.js": "./dist/a.js"
        }),
        "#a/index.js",
        vec![],
        vec!["./dist/a.js"],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": ["./browser/"]
            },
            "#a/index.js": {
                "browser": "./index.js"
            }
        }),
        "#a/index.js",
        vec!["browser"],
        vec!["./index.js"],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": ["./browser/"]
            },
            "#a/index.js": {
                "node": "./node.js"
            }
        }),
        "#a/index.js",
        vec!["browser"],
        vec![],
    );
    should_equal(
        json!({
            "#a": {
                "browser": "./index.js",
                "node": "./src/node/index.js",
                "default": "./src/index.js"
            },
        }),
        "#a",
        vec!["browser"],
        vec!["./index.js"],
    );
    should_equal(
        json!({
            "#a": {
                "browser": "./index.js",
                "node": "./src/node/index.js",
                "default": "./src/index.js"
            },
        }),
        "#a",
        vec![],
        vec!["./src/index.js"],
    );
    should_equal(
        json!({
            "#a": "./index"
        }),
        "#a",
        vec![],
        vec!["./index"],
    );
    should_equal(
        json!({
            "#a/index": "./index.js"
        }),
        "#a/index",
        vec![],
        vec!["./index.js"],
    );
    should_equal(
        json!({
            "#a": "b"
        }),
        "#a",
        vec![],
        vec!["b"],
    );
    should_equal(
        json!({
            "#a/": "b/"
        }),
        "#a/index",
        vec![],
        vec!["b/index"],
    );
    should_equal(
        json!({
            "#a?q=a#hashishere": "b#anotherhashishere"
        }),
        "#a?q=a#hashishere",
        vec![],
        vec!["b#anotherhashishere"],
    );
    should_equal(
        json!({
            "#a": [
                {"browser": "./browser.js"},
                {"require": "./require.js"},
                {"import": "./import.mjs"}
            ],
        }),
        "#a",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a": [
                {"browser": "./browser.js"},
                {"require": "./require.js"},
                {"import": "./import.mjs"}
            ],
        }),
        "#a",
        vec!["import"],
        vec!["./import.mjs"],
    );
    should_equal(
        json!({
            "#a": [
                {"browser": "./browser.js"},
                {"require": "./require.js"},
                {"import": "./import.mjs"}
            ],
        }),
        "#a",
        vec!["import", "require"],
        vec!["./require.js", "./import.mjs"],
    );
    should_equal(
        json!({
            "#a": [
                {"browser": "./browser.js"},
                {"require": "./require.js"},
                {"import": ["./import.mjs", "#b/import.js"]}
            ],
        }),
        "#a",
        vec!["import", "require"],
        vec!["./require.js", "./import.mjs", "#b/import.js"],
    );
    should_equal(
        json!({
            "#timezones": "./data/timezones/"
        }),
        "#timezones/pdt.mjs",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#timezones/pdt/": "./data/timezones/pdt/"
        }),
        "#timezones/pdt/index.mjs",
        vec![],
        vec!["./data/timezones/pdt/index.mjs"],
    );
    should_equal(
        json!({
            "#a/": "./timezones/"
        }),
        "#a/pdt.mjs",
        vec![],
        vec!["./timezones/pdt.mjs"],
    );
    should_equal(
        json!({
            "#a/": "./"
        }),
        "#a/timezones/pdt.mjs",
        vec![],
        vec!["./timezones/pdt.mjs"],
    );
    should_equal(
        json!({
            "#a": "."
        }),
        "#a/timezones/pdt.mjs",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a": "./"
        }),
        "#a/timezones/pdt.mjs",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a/": "./",
            "#a/dist/": "./lib/"
        }),
        "#a/dist/index.mjs",
        vec![],
        vec!["./lib/index.mjs"],
    );
    should_equal(
        json!({
            "#a/dist/utils/": "./dist/utils/",
            "#a/dist/": "./lib/"
        }),
        "#a/dist/utils/index.js",
        vec![],
        vec!["./dist/utils/index.js"],
    );
    should_equal(
        json!({
            "#a/dist/utils/index.js": "./dist/utils/index.js",
            "#a/dist/utils/": "./dist/utils/index.mjs",
            "#a/dist/": "./lib/"
        }),
        "#a/dist/utils/index.js",
        vec![],
        vec!["./dist/utils/index.js"],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": "./browser/"
            },
            "#a/dist/": "./lib/"
        }),
        "#a/dist/index.js",
        vec![],
        vec!["./lib/index.js"],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": "./browser/"
            },
            "#a/dist/": "./lib/"
        }),
        "#a/dist/index.js",
        vec!["browser"],
        vec!["./lib/index.js"],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": ["lodash/", "./utils/"],
                "node": ["./utils-node/"]
            },
        }),
        "#a/index.js",
        vec!["browser"],
        vec!["lodash/index.js", "./utils/index.js"],
    );
    should_equal(
        json!({
            "#a/": {
                "webpack": "./wpk",
                "browser": ["lodash/", "./utils/"],
                "node": ["./node/"]
            },
        }),
        "#a/index.mjs",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a/": {
                "webpack": "./wpk/",
                "browser": ["lodash/", "./utils/"],
                "node": ["./node/"]
            },
        }),
        "#a/index.mjs",
        vec!["browser", "webpack"],
        vec!["./wpk/index.mjs"],
    );
    should_equal(
        json!({
            "#a/index": "./a/index.js"
        }),
        "#a/index.mjs",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a/index.mjs": "./a/index.js"
        }),
        "#a/index",
        vec![],
        vec![],
    );
    should_equal(
        json!({
            "#a/index": {
                "browser": "./a/index.js",
                "default": "./b/index.js"
            }
        }),
        "#a/index.mjs",
        vec!["browser"],
        vec![],
    );
    should_equal(
        json!({
            "#a/index.mjs": {
                "browser": "./a/index.js",
                "default": "./b/index.js"
            }
        }),
        "#a/index",
        vec!["browser"],
        vec![],
    );
    should_equal(
        json!({
            "#a/../../utils/": "./dist/"
        }),
        "#a/../../utils/index",
        vec![],
        vec!["./dist/index"],
    );
    should_equal(
        json!({
            "#a/": "./dist/"
        }),
        "#a/../../utils/index",
        vec![],
        vec!["./dist/../../utils/index"],
    );
    should_equal(
        json!({
            "#a/": "../src/"
        }),
        "#a/index",
        vec![],
        vec!["../src/index"],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": "./utils/../../../"
            }
        }),
        "#a/index",
        vec!["browser"],
        vec!["./utils/../../../index"],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": "moment/node_modules/"
            }
        }),
        "#a/lodash/dist/index.js",
        vec!["browser"],
        vec!["moment/node_modules/lodash/dist/index.js"],
    );
    should_equal(
        json!({
            "#a/": "../node_modules/"
        }),
        "#a/lodash/dist/index.js",
        vec!["browser"],
        vec!["../node_modules/lodash/dist/index.js"],
    );
    should_equal(
        json!({
            "#a/": "../node_modules/"
        }),
        "#a/lodash/dist/index.js",
        vec![],
        vec!["../node_modules/lodash/dist/index.js"],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": {
                    "webpack": "./",
                    "default": {
                        "node": "./node/"
                    }
                }
            }
        }),
        "#a/index.js",
        vec!["browser"],
        vec![],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": {
                    "webpack": ["./", "./node/"],
                    "default": {
                        "node": "./node/"
                    }
                }
            }
        }),
        "#a/index.js",
        vec!["browser", "webpack"],
        vec!["./index.js", "./node/index.js"],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": {
                    "webpack": ["./", "./node/"],
                    "default": {
                        "node": "./node/"
                    }
                }
            }
        }),
        "#a/index.js",
        vec!["webpack"],
        vec![],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": {
                    "webpack": ["./", "./node/"],
                    "default": {
                        "node": "moment/node/"
                    }
                }
            }
        }),
        "#a/index.js",
        vec!["node", "browser"],
        vec!["moment/node/index.js"],
    );
    should_equal(
        json!({
            "#a/": {
                "browser": {
                    "webpack": ["./", "./node/"],
                    "default": {
                        "node": {
                            "webpack": ["./wpck"]
                        }
                    }
                }
            }
        }),
        "#a/index.js",
        vec!["browser", "node", "webpack"],
        vec!["./index.js", "./node/index.js"],
    );
    should_equal(
        json!({
            "#a": {
                "abc": {
                    "def": "./x.js"
                },
                "ghi": "./y.js"
            }
        }),
        "#a",
        vec!["abc", "ghi"],
        vec!["./y.js"],
    );
    should_equal(
        json!({
            "#a": {
                "abc": {
                    "def": "./x.js",
                    "default": []
                },
                "ghi": "./y.js"
            }
        }),
        "#a",
        vec!["abc", "ghi"],
        vec![],
    );
    should_error(
        json!({
            "/utils/": "./a/",
        }),
        "#a/index.mjs",
        vec![],
        "Imports field key should start with #, but got /utils/",
    );
    should_error(
        json!({
            "/utils/": {
                "browser": "./a/",
                "default": "./b/"
            },
        }),
        "#a/index.mjs",
        vec![],
        "Imports field key should start with #, but got /utils/",
    );
    should_error(
        json!({
            "#a": {
                "default": "./src/index.js",
                "browser": "./index.js",
                "node": "./src/node/index.js"
            },
        }),
        "#a",
        vec!["browser"],
        "Default condition should be last one",
    );
    should_error(
        json!({
            "#timezones/": "./data/timezones"
        }),
        "#timezones/pdt.mjs",
        vec![],
        "Expected ./data/timezones is folder mapping",
    );
    should_error(
        json!({
            "#a/": "./a/"
        }),
        "/utils/index.mjs",
        vec![],
        "Request should start with #, but got /utils/index.mjs",
    );
    should_error(
        json!({
            "#a/": {
                "browser": "./a/",
                "default": "./b/"
            }
        }),
        "/utils/index.mjs",
        vec![],
        "Request should start with #, but got /utils/index.mjs",
    );
    should_error(
        json!({
            "#a/": {
                "browser": "./a/",
                "default": "./b/"
            }
        }),
        "#",
        vec!["browser"],
        "Request should have at least 2 characters",
    );
    should_error(
        json!({
            "#a/": {
                "browser": "./a/",
                "default": "./b/"
            }
        }),
        "#/",
        vec!["browser"],
        "Import field key should not start with #/, but got #/",
    );
    should_error(
        json!({
            "#a/": {
                "browser": "./a/",
                "default": "./b/"
            }
        }),
        "#a/",
        vec!["browser"],
        "Only requesting file allowed",
    );
}

#[test]
fn check_target_test() {
    assert!(ExportsField::check_target("../a.js").is_err());
    assert!(ExportsField::check_target("../").is_err());
    assert!(ExportsField::check_target("./a/b/../../../c.js").is_err());
    assert!(ExportsField::check_target("./a/b/../../../").is_err());
    assert!(ExportsField::check_target("./../../c.js").is_err());
    assert!(ExportsField::check_target("./../../").is_err());
    assert!(ExportsField::check_target("./a/../b/../../c.js").is_err());
    assert!(ExportsField::check_target("./a/../b/../../").is_err());
    assert!(ExportsField::check_target("./././../").is_err());
}
