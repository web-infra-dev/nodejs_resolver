/// port from https://github.com/webpack/enhanced-resolve/blob/main/lib/util/entrypoints.js
use crate::{RResult, ResolverError};
use indexmap::IndexMap;
use std::collections::HashSet;
type DirectMapping = String;
type ConditionalMapping = IndexMap<String, MappingValue>;

#[derive(Clone, Debug)]
pub enum AvailableMapping {
    Direct(DirectMapping),
    Conditional(ConditionalMapping),
}
type ArrayMapping = Vec<AvailableMapping>;

#[derive(Clone, Debug)]
pub enum MappingValue {
    Direct(DirectMapping),
    Conditional(ConditionalMapping),
    Array(ArrayMapping),
}

pub type ImportsField = ConditionalMapping;
pub type ExportsField = MappingValue;

fn conditional_mapping<'a>(
    map: &'a ConditionalMapping,
    condition_names: &'a HashSet<String>,
) -> RResult<Option<&'a MappingValue>> {
    let mut lookup: Vec<(&ConditionalMapping, Vec<String>, usize)> =
        vec![(map, map.keys().map(String::from).collect(), 0)];
    'outer: while !lookup.is_empty() {
        let (mapping, conditions, j) = lookup.last().unwrap();
        let len = conditions.len();
        for (i, condition) in conditions.iter().enumerate().skip(*j) {
            if i != len - 1 && condition == "default" {
                return Err(ResolverError::UnexpectedValue(
                    "Default condition should be last one".to_string(),
                ));
            }
            if condition == "default" {
                if let Some(value) = mapping.get("default") {
                    match value {
                        MappingValue::Conditional(inner) => {
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
                        MappingValue::Conditional(inner) => {
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

#[derive(Default, Clone, Debug)]
pub struct PathTreeNode {
    pub children: Option<IndexMap<String, PathTreeNode>>,
    pub folder: Option<MappingValue>,
    pub wildcards: Option<IndexMap<String, MappingValue>>,
    pub files: IndexMap<String, MappingValue>,
}

/// TODO: should seal all functions except
///  `build_field_path_tree` and `field_process`.
pub trait Field {
    fn check_target(relative_path: &str) -> bool {
        let relative_path = relative_path.chars().collect::<Vec<char>>();
        let slash_index_list = PathTreeNode::get_next_list(&relative_path, '/');
        let mut last_non_slash_index = 0;
        let mut cd = 0;
        while let Some(&Some(slash_index)) = slash_index_list.get(last_non_slash_index) {
            if relative_path[last_non_slash_index] == '.'
                && relative_path[last_non_slash_index + 1] == '.'
            {
                cd -= 1;
                if cd < 0 {
                    return false;
                }
            } else if relative_path[last_non_slash_index] == '.' {
            } else {
                cd += 1;
            }
            last_non_slash_index = slash_index + 1;
        }
        true
    }

    fn assert_target(exp: &str, expect_folder: bool) -> RResult<bool>;
    fn assert_request(request: &str) -> RResult<Vec<char>>;
    fn build_field_path_tree(json_value: &serde_json::Value) -> RResult<PathTreeNode>;
    fn from_json(json_value: &serde_json::Value) -> RResult<MappingValue>
    where
        Self: Sized,
    {
        let result = match json_value {
            // TODO: null
            serde_json::Value::String(str) => MappingValue::Direct(str.to_string()),
            serde_json::Value::Array(arr) => {
                let mut temp: ArrayMapping = vec![];
                for item in arr {
                    match Self::from_json(item)? {
                        MappingValue::Direct(direct) => temp.push(AvailableMapping::Direct(direct)),
                        MappingValue::Conditional(conditional) => {
                            temp.push(AvailableMapping::Conditional(conditional))
                        }
                        _ => panic!("Array mapping is not allowed nested in exports field"),
                    }
                }
                MappingValue::Array(temp)
            }
            serde_json::Value::Object(obj) => {
                let mut map = IndexMap::new();
                for (key, value) in obj {
                    map.insert(key.to_string(), Self::from_json(value)?);
                }
                MappingValue::Conditional(map)
            }
            _ => unreachable!(),
        };
        Ok(result)
    }

    fn target_mapping(
        remaining_request: &Option<String>,
        subpath_mapping: bool,
        target: &str,
    ) -> RResult<String> {
        let is_folder = Self::assert_target(target, true)?;
        if let Some(request) = remaining_request {
            match (subpath_mapping, is_folder) {
                (true, true) => Ok(format!("{target}{request}")),
                (true, false) => Err(ResolverError::UnexpectedValue(format!(
                    "Expected {target} is folder mapping"
                ))),
                (false, true) => Err(ResolverError::UnexpectedValue(format!(
                    "Expected {target} is file mapping"
                ))),
                (false, false) => {
                    let request = remaining_request.as_ref().unwrap();
                    let to = &request.replace('$', "$$");
                    Ok(target.replace('*', to))
                }
            }
        } else if !is_folder {
            Ok(target.to_string())
        } else {
            Err(ResolverError::UnexpectedValue(format!(
                "{target} had some wrong"
            )))
        }
    }

    fn mapping(
        remaining_request: &Option<String>,
        subpath_mapping: bool,
        mapping: &MappingValue,
        condition_names: &HashSet<String>,
    ) -> RResult<Vec<String>> {
        Ok(match mapping {
            MappingValue::Direct(target) => {
                vec![Self::target_mapping(
                    remaining_request,
                    subpath_mapping,
                    target,
                )?]
            }
            MappingValue::Array(target) => {
                let mut acc = vec![];
                for exp in target {
                    match exp {
                        AvailableMapping::Direct(target) => acc.push(Self::target_mapping(
                            remaining_request,
                            subpath_mapping,
                            target,
                        )?),
                        AvailableMapping::Conditional(map) => {
                            if let Some(mapping) = conditional_mapping(map, condition_names)? {
                                let inner_exports = Self::mapping(
                                    remaining_request,
                                    subpath_mapping,
                                    mapping,
                                    condition_names,
                                )?;
                                for inner_export in inner_exports {
                                    acc.push(inner_export);
                                }
                            }
                        }
                    };
                }
                acc
            }
            MappingValue::Conditional(map) => match conditional_mapping(map, condition_names)? {
                Some(mapping_value) => Self::mapping(
                    remaining_request,
                    subpath_mapping,
                    mapping_value,
                    condition_names,
                )?,
                None => return Ok(vec![]),
            },
        })
    }

    fn field_process<'a>(
        root: &'a PathTreeNode,
        target: &'a str,
        condition_names: &'a HashSet<String>,
    ) -> RResult<Vec<String>> {
        let request = Self::assert_request(target)?;
        let (mapping, remain_request_index) = match PathTreeNode::find_match(root, &request) {
            Some(result) => result,
            None => return Ok(vec![]),
        };

        let remaining_request: Option<String> =
            if remain_request_index == (request.len() as i32) + 1 {
                None
            } else if remain_request_index < 0 {
                let remaining = request
                    .iter()
                    .skip((remain_request_index.abs() - 1) as usize)
                    .collect();
                Some(remaining)
            } else {
                let remaining = request.iter().skip(remain_request_index as usize).collect();
                Some(remaining)
            };

        Self::mapping(
            &remaining_request,
            remain_request_index < 0,
            mapping,
            condition_names,
        )
    }
}

impl Field for ExportsField {
    fn assert_request(request: &str) -> RResult<Vec<char>> {
        if !request.starts_with('.') {
            Err(ResolverError::UnexpectedValue(format!(
                "Request should be relative path and start with '.', but got {request}"
            )))
        } else if request.len() == 1 {
            Ok(vec![])
        } else if !request.starts_with("./") {
            Err(ResolverError::UnexpectedValue(format!(
                "Request should be relative path and start with '.', but got {request}"
            )))
        } else if request.ends_with('/') {
            Err(ResolverError::UnexpectedValue(
                "Only requesting file allowed".to_string(),
            ))
        } else {
            // To avoid unicode char
            Ok(request.chars().skip(2).collect())
        }
    }

    fn assert_target(exp: &str, expect_folder: bool) -> RResult<bool> {
        if exp.len() < 2 || exp.starts_with('/') || (exp.starts_with('.') && !exp.starts_with("./"))
        {
            Err(ResolverError::UnexpectedValue(format!(
                "Export should be relative path and start with \"./\", but got {exp}"
            )))
        } else if exp.ends_with('/') != expect_folder {
            Ok(!expect_folder)
        } else {
            Ok(true)
        }
    }

    /// reference: https://nodejs.org/api/packages.html#exports
    #[tracing::instrument]
    fn build_field_path_tree(exports_field_value: &serde_json::Value) -> RResult<PathTreeNode> {
        let field = Self::from_json(exports_field_value)?;
        let mut root = PathTreeNode::default();
        match field {
            Self::Conditional(map) => {
                // TODO: should optimize this to once iter.
                let (all_keys_are_conditional, all_keys_are_direct) =
                    map.iter().fold((true, true), |pre, (key, _)| {
                        let is_starts_with_dot = key.starts_with('.');
                        let is_starts_with_slash = key.starts_with('/');
                        let is_direct = is_starts_with_dot || is_starts_with_slash;
                        let is_conditional = !is_direct;
                        (pre.0 & is_conditional, pre.1 & is_direct)
                    });
                if !all_keys_are_conditional && !all_keys_are_direct {
                    return Err(ResolverError::UnexpectedValue(
                        "Export field key can't mixed relative path and conditional object"
                            .to_string(),
                    ));
                } else if all_keys_are_conditional {
                    root.files
                        .insert("".to_string(), MappingValue::Conditional(map));
                } else {
                    for (key, value) in map {
                        if key.len() == 1 {
                            // key eq "."
                            root.files.insert("".to_string(), value);
                        } else if !key.starts_with("./") {
                            return Err(ResolverError::UnexpectedValue(format!(
                                "Export field key should be relative path and start with \"./\", but got {key}",
                            )));
                        } else {
                            PathTreeNode::walk(&mut root, key[2..].chars().collect(), value);
                        }
                    }
                }
            }
            Self::Array(array) => {
                root.files
                    .insert("".to_string(), MappingValue::Array(array));
            }
            Self::Direct(direct) => {
                root.files
                    .insert("".to_string(), MappingValue::Direct(direct));
            }
        }
        Ok(root)
    }
}

impl Field for ImportsField {
    fn assert_request(request: &str) -> RResult<Vec<char>> {
        if !request.starts_with('#') {
            Err(ResolverError::UnexpectedValue(format!(
                "Request should start with #, but got {request}"
            )))
        } else if request.len() == 1 {
            Err(ResolverError::UnexpectedValue(
                "Request should have at least 2 characters".to_string(),
            ))
        } else if request.starts_with("#/") {
            Err(ResolverError::UnexpectedValue(format!(
                "Import field key should not start with #/, but got {request}"
            )))
        } else if request.ends_with('/') {
            Err(ResolverError::UnexpectedValue(
                "Only requesting file allowed".to_string(),
            ))
        } else {
            Ok(request.chars().skip(1).collect())
        }
    }

    fn assert_target(exp: &str, expect_folder: bool) -> RResult<bool> {
        let is_folder = exp.ends_with('/');
        if is_folder != expect_folder {
            Ok(!expect_folder)
        } else {
            Ok(true)
        }
    }

    /// reference: https://nodejs.org/api/packages.html#imports
    #[tracing::instrument]
    fn build_field_path_tree(imports_field_value: &serde_json::Value) -> RResult<PathTreeNode> {
        let field = match Self::from_json(imports_field_value)? {
            MappingValue::Conditional(field) => field,
            _ => unreachable!(),
        };
        let mut root = PathTreeNode::default();
        for (key, value) in field {
            if !key.starts_with('#') {
                return Err(ResolverError::UnexpectedValue(format!(
                    "Imports field key should start with #, but got {key}"
                )));
            } else if key.len() == 1 {
                // key eq "#"
                return Err(ResolverError::UnexpectedValue(format!(
                    "Imports field key should have at least 2 characters, but got {key}"
                )));
            } else if key.starts_with("#/") {
                return Err(ResolverError::UnexpectedValue(format!(
                    "Import field key should not start with #/, but got {key}"
                )));
            }
            PathTreeNode::walk(&mut root, key[1..].chars().collect(), value);
        }
        Ok(root)
    }
}

impl PathTreeNode {
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

    fn apply_folder_mapping<'a>(
        last_folder_match: Option<(&'a MappingValue, i32)>,
        node: &'a PathTreeNode,
        last_non_slash_index: usize,
    ) -> Option<(&'a MappingValue, i32)> {
        if let Some(map) = node.folder.as_ref() {
            Some((map, -(last_non_slash_index as i32) - 1))
        } else {
            last_folder_match
        }
    }

    fn apply_wildcard_mappings<'a, 'b>(
        mut last_folder_match: Option<(&'a MappingValue, i32)>,
        node: &'a PathTreeNode,
        remaining_request: &'b str,
        last_non_slash_index: usize,
    ) -> Option<(&'a MappingValue, i32)> {
        if let Some(map) = &node.wildcards {
            for (key, target) in map {
                if remaining_request.starts_with(key) {
                    let index = (last_non_slash_index + key.len()) as i32;
                    if last_folder_match.is_none() || last_folder_match.unwrap().1 < index {
                        last_folder_match = Some((target, index));
                    }
                }
            }
        }
        last_folder_match
    }

    fn find_match<'a>(
        root: &'a PathTreeNode,
        request: &'a Vec<char>,
    ) -> Option<(&'a MappingValue, i32)> {
        if request.is_empty() {
            root.files.get("").map(|value| (value, 1))
        } else if root.children.is_none() && root.folder.is_none() && root.wildcards.is_none() {
            let key = &request.iter().collect::<String>();
            root.files
                .get(key)
                .map(|value| (value, (request.len() + 1) as i32))
        } else {
            let slash_index_list = Self::get_next_list(request, '/');
            let mut last_non_slash_index = 0;
            let mut node = root;
            let mut last_folder_match = None;
            while let Some(&Some(slash_index)) = slash_index_list.get(last_non_slash_index) {
                last_folder_match =
                    Self::apply_folder_mapping(last_folder_match, node, last_non_slash_index);
                if node.wildcards.is_none() && node.children.is_none() {
                    return last_folder_match;
                }

                let folder = &request
                    .iter()
                    .skip(last_non_slash_index)
                    .take(slash_index - last_non_slash_index)
                    .collect::<String>();
                last_folder_match = Self::apply_wildcard_mappings(
                    last_folder_match,
                    node,
                    folder,
                    last_non_slash_index,
                );
                if node.children.is_none() {
                    return last_folder_match;
                }

                if let Some(new_node) = node.children.as_ref().unwrap().get(folder) {
                    node = new_node;
                    last_non_slash_index = slash_index + 1;
                } else {
                    return last_folder_match;
                }
            }
            let remaining_request = (if last_non_slash_index > 0 {
                &request[last_non_slash_index as usize..]
            } else {
                request
            })
            .iter()
            .collect::<String>();
            if let Some(value) = node.files.get(&remaining_request) {
                Some((value, (remaining_request.len() + 1) as i32))
            } else {
                Self::apply_wildcard_mappings(
                    Self::apply_folder_mapping(last_folder_match, node, last_non_slash_index),
                    node,
                    &remaining_request,
                    last_non_slash_index,
                )
            }
        }
    }

    /// Tire
    fn walk(root: &mut PathTreeNode, path: Vec<char>, target: MappingValue) {
        if path.is_empty() {
            root.folder = Some(target);
            return;
        }
        let slash_index_list = Self::get_next_list(&path, '/');
        let mut last_non_slash_index = 0;
        let mut node = root;
        while let Some(&Some(slash_index)) = slash_index_list.get(last_non_slash_index) {
            let slice: &String = &path[last_non_slash_index..slash_index].iter().collect();
            let folder: String = slice.to_string();
            if node.children.is_none() {
                let mut map = IndexMap::new();
                map.insert(folder, PathTreeNode::default());
                node.children = Some(map);
            } else if let Some(children) = node.children.as_mut() {
                children.entry(folder).or_insert_with(PathTreeNode::default);
            }

            node = node.children.as_mut().unwrap().get_mut(slice).unwrap();
            last_non_slash_index = slash_index + 1;
        }

        if last_non_slash_index >= path.len() {
            node.folder = Some(target);
        } else {
            let file: String = if last_non_slash_index > 0 {
                path[last_non_slash_index..].iter().collect()
            } else {
                path.iter().collect()
            };
            if file.ends_with('*') {
                let file = file[..file.len() - 1].to_string();
                if let Some(wildcards) = node.wildcards.as_mut() {
                    wildcards.insert(file, target);
                } else {
                    let mut map = IndexMap::new();
                    map.insert(file, target);
                    node.wildcards = Some(map);
                }
            } else {
                node.files.insert(file, target);
            }
        }
    }
}

#[test]
fn exports_field_map_test() {
    use crate::test_helper;

    fn process_exports_fields(
        value: serde_json::Value,
        request: &str,
        condition_names: Vec<&str>,
    ) -> RResult<Vec<String>> {
        ExportsField::build_field_path_tree(&value).and_then(|root| {
            ExportsField::field_process(&root, &request, &test_helper::vec_to_set(condition_names))
        })
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
        assert_eq!(
            actual,
            expected
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        )
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
            ResolverError::UnexpectedValue(message) => assert_eq!(expected_error_message, message),
            _ => unreachable!(),
        }
    }

    use serde_json::json;

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
    should_equal(
        json!(["./a.js", "./b.js"]),
        ".",
        vec![],
        vec!["./a.js", "./b.js"],
    );
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
        "Export field key can't mixed relative path and conditional object",
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
        "Export field key can't mixed relative path and conditional object",
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
        "Export field key should be relative path and start with \"./\", but got ../../utils/",
    );
    should_error(
        json!({
            "../../utils/*": "./dist/*"
        }),
        "../../utils/index",
        vec![],
        "Export field key should be relative path and start with \"./\", but got ../../utils/*",
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
fn imports_field_map_test() {
    use crate::test_helper;
    fn process_imports_fields(
        value: serde_json::Value,
        request: &str,
        condition_names: Vec<&str>,
    ) -> RResult<Vec<String>> {
        ImportsField::build_field_path_tree(&value).and_then(|root| {
            ImportsField::field_process(&root, &request, &test_helper::vec_to_set(condition_names))
        })
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
        assert_eq!(
            actual,
            expected
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        )
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
            ResolverError::UnexpectedValue(message) => assert_eq!(expected_error_message, message),
            _ => unreachable!(),
        }
    }

    use serde_json::json;

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
    assert!(!ExportsField::check_target("../a.js"));
    assert!(!ExportsField::check_target("../"));
    assert!(!ExportsField::check_target("./a/b/../../../c.js"));
    assert!(!ExportsField::check_target("./a/b/../../../"));
    assert!(!ExportsField::check_target("./../../c.js"));
    assert!(!ExportsField::check_target("./../../"));
    assert!(!ExportsField::check_target("./a/../b/../../c.js"));
    assert!(!ExportsField::check_target("./a/../b/../../"));
    assert!(!ExportsField::check_target("./././../"));
}
