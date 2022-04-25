/// port from https://github.com/webpack/enhanced-resolve/blob/main/lib/util/entrypoints.js
use crate::RResult;
use indexmap::IndexMap;
use std::collections::HashSet;
type DirectMapping = String;
type ConditionalMapping = IndexMap<String, MappingValue>;

#[derive(Debug)]
pub enum AvailableMapping {
    Direct(DirectMapping),
    Conditional(ConditionalMapping),
}
type ArrayMapping = Vec<AvailableMapping>;

#[derive(Debug)]
pub enum MappingValue {
    Direct(DirectMapping),
    Conditional(ConditionalMapping),
    Array(ArrayMapping),
}

// type ImportsField = ConditionalMapping;
type ExportsField = MappingValue;

fn conditional_mapping<'a>(
    map: &'a ConditionalMapping,
    conditional_names: &'a HashSet<String>,
) -> RResult<Option<&'a MappingValue>> {
    let mut lookup: Vec<(&ConditionalMapping, Vec<String>, usize)> =
        vec![(map, map.keys().map(String::from).collect(), 0)];
    'outer: while !lookup.is_empty() {
        let (mapping, conditions, j) = lookup.last().unwrap();
        let len = conditions.len();
        for (i, condition) in conditions.iter().enumerate().skip(*j) {
            if i != len - 1 && condition == "default" {
                return Err("Default condition should be last one".to_string());
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

            if conditional_names.contains(condition) {
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

#[derive(Default, Debug)]
pub struct PathTreeNode {
    pub children: Option<IndexMap<String, PathTreeNode>>,
    pub folder: Option<MappingValue>,
    pub wildcards: Option<IndexMap<String, MappingValue>>,
    pub files: IndexMap<String, MappingValue>,
}

trait Field {
    fn assert_target(exp: &str, expect_folder: bool) -> RResult<bool>;
    fn assert_request(request: &str) -> RResult<&str>;
    fn build_field_path_tree(filed: Self) -> RResult<PathTreeNode>;
    fn from_json(json_value: &serde_json::Value) -> RResult<Self>
    where
        Self: Sized;

    fn process_field(
        json_value: &serde_json::Value,
        request: &str,
        condition_names: &HashSet<String>,
    ) -> RResult<Vec<String>>;

    fn target_mapping(
        remaining_request: Option<&str>,
        subpath_mapping: bool,
        target: &str,
    ) -> RResult<String> {
        let is_folder = Self::assert_target(target, true)?;
        if let Some(request) = remaining_request {
            match (subpath_mapping, is_folder) {
                (true, true) => Ok(format!("{target}{request}")),
                (true, false) => Err(format!("Expected {target} is folder mapping")),
                (false, true) => Err(format!("Expected {target} is file mapping")),
                (false, false) => {
                    let request = remaining_request.unwrap();
                    let to = &request.replace('$', "$$");
                    Ok(target.replace('*', to))
                }
            }
        } else if !is_folder {
            Ok(target.to_string())
        } else {
            Err(format!("{target} had some wrong"))
        }
    }

    fn mapping(
        remaining_request: Option<&str>,
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
                            match conditional_mapping(map, condition_names)? {
                                Some(mapping) => {
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
                                None => (),
                            };
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
        request: &'a str,
        condition_names: &'a HashSet<String>,
    ) -> RResult<Vec<String>> {
        let request = Self::assert_request(request)?;
        let (mapping, remain_request_index) = match PathTreeNode::find_match(root, request) {
            Some(result) => result,
            None => return Ok(vec![]),
        };
        let remaining_request = if remain_request_index == (request.len() as i32) + 1 {
            None
        } else if remain_request_index < 0 {
            Some(&request[(remain_request_index.abs() - 1) as usize..])
        } else {
            Some(&request[remain_request_index as usize..])
        };

        Self::mapping(
            remaining_request,
            remain_request_index < 0,
            mapping,
            condition_names,
        )
    }
}

impl Field for ExportsField {
    fn from_json(json_value: &serde_json::Value) -> RResult<Self> {
        let result = match json_value {
            serde_json::Value::String(str) => ExportsField::Direct(str.to_string()),
            serde_json::Value::Array(arr) => {
                let mut temp: ArrayMapping = vec![];
                for item in arr {
                    match Self::from_json(item)? {
                        ExportsField::Direct(direct) => temp.push(AvailableMapping::Direct(direct)),
                        ExportsField::Conditional(conditional) => {
                            temp.push(AvailableMapping::Conditional(conditional))
                        }
                        _ => panic!("array mapping is not allowed nested in exports field"),
                    }
                }
                ExportsField::Array(temp)
            }
            serde_json::Value::Object(obj) => {
                let mut map = IndexMap::new();
                for (key, value) in obj {
                    map.insert(key.to_string(), Self::from_json(value)?);
                }
                ExportsField::Conditional(map)
            }
            _ => unreachable!(),
        };
        Ok(result)
    }

    fn assert_request(request: &str) -> RResult<&str> {
        if !request.starts_with('.') {
            Err(format!(
                "Request should be relative path and start with '.', but got {request}"
            ))
        } else if request.len() == 1 {
            Ok("")
        } else if !request.starts_with("./") {
            Err(format!(
                "Request should be relative path and start with '.', but got {request}"
            ))
        } else if request.ends_with('/') {
            Err("Only requesting file allowed".to_string())
        } else {
            Ok(&request[2..])
        }
    }

    fn assert_target(exp: &str, expect_folder: bool) -> RResult<bool> {
        if exp.len() < 2 || exp.starts_with('/') || (exp.starts_with('.') && !exp.starts_with("./"))
        {
            Err(format!(
                "Export should be relative path and start with \"./\", but got {exp}"
            ))
        } else if exp.ends_with('/') != expect_folder {
            Ok(!expect_folder)
        } else {
            Ok(true)
        }
    }

    /// reference: https://nodejs.org/api/packages.html#exports
    fn build_field_path_tree(field: Self) -> RResult<PathTreeNode> {
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
                    return Err(
                        "Export field key can't mixed relative path and conditional object"
                            .to_string(),
                    );
                } else if all_keys_are_conditional {
                    root.files
                        .insert("".to_string(), MappingValue::Conditional(map));
                } else {
                    for (key, value) in map {
                        if key.len() == 1 {
                            // key eq "."
                            root.files.insert("".to_string(), value);
                        } else if !key.starts_with("./") {
                            return Err(format!(
                                "Export field key should be relative path and start with \"./\", but got {key}",
                            ));
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

    fn process_field(
        json_value: &serde_json::Value,
        request: &str,
        condition_names: &HashSet<String>,
    ) -> RResult<Vec<String>> {
        let field = Self::from_json(json_value)?;
        let root = Self::build_field_path_tree(field)?;
        Self::field_process(&root, request, condition_names)
    }
}

// impl Field for ImportsField {
//     fn assert_target(exp: &str, expect_folder: bool) -> bool {
//         false
//     }

//     fn assert_request(request: &str) -> RResult<&str> {
//         Ok("")
//     }

//     fn build_field_path_tree(filed: Self) -> RResult<PathTreeNode> {
//         let root = PathTreeNode::default();
//         Ok(root)
//     }

//     fn from_json(json_value: &serde_json::Value) -> RResult<Self> {
//         Ok(HashMap::new())
//     }

//     fn process_field(
//         json_value: &serde_json::Value,
//         request: &str,
//         condition_names: &HashSet<String>,
//     ) -> RResult<Vec<String>> {
//         Ok(vec![])
//     }
// }

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

    pub fn apply_folder_mapping<'a>(
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

    pub fn apply_wildcard_mappings<'a>(
        mut last_folder_match: Option<(&'a MappingValue, i32)>,
        node: &'a PathTreeNode,
        remaining_request: &'a str,
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

    pub fn find_match<'a>(
        root: &'a PathTreeNode,
        request: &'a str,
    ) -> Option<(&'a MappingValue, i32)> {
        if request.is_empty() {
            root.files.get("").map(|value| (value, 1))
        } else if root.children.is_none() && root.folder.is_none() && root.wildcards.is_none() {
            root.files
                .get(request)
                .map(|value| (value, (request.len() + 1) as i32))
        } else {
            let path: Vec<char> = request.chars().collect();
            // TODO: cache
            let slash_index_list = Self::get_next_list(&path, '/');
            let mut last_non_slash_index = 0;
            let mut node = root;
            let mut last_folder_match = None;
            while let Some(&Some(slash_index)) = slash_index_list.get(last_non_slash_index) {
                last_folder_match =
                    Self::apply_folder_mapping(last_folder_match, node, last_non_slash_index);
                if node.wildcards.is_none() && node.children.is_none() {
                    return last_folder_match;
                }

                let folder = &request[last_non_slash_index..slash_index];
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
            let remaining_request = if last_non_slash_index > 0 {
                &request[last_non_slash_index as usize..]
            } else {
                request
            };
            if let Some(value) = node.files.get(remaining_request) {
                Some((value, (remaining_request.len() + 1) as i32))
            } else {
                Self::apply_wildcard_mappings(
                    Self::apply_folder_mapping(last_folder_match, node, last_non_slash_index),
                    node,
                    remaining_request,
                    last_non_slash_index,
                )
            }
        }
    }

    /// Tire
    pub fn walk(root: &mut PathTreeNode, path: Vec<char>, target: MappingValue) {
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
fn exports_fields_map_test() {
    use serde_json::json;

    macro_rules! process_exports_fields {
        ($exports_field: expr, $request: expr, $condition_names: expr) => {
            ExportsField::process_field(
                &json!($exports_field),
                $request,
                &HashSet::from_iter($condition_names.into_iter().map(|s: &str| s.to_string())),
            )
        };
    }

    macro_rules! should_equal {
        ($exports_field: expr, $request: expr, $condition_names: expr; $expect: expr) => {
            assert_eq!(
                process_exports_fields!($exports_field, $request, $condition_names),
                Ok($expect.into_iter().map(|s: &str| s.to_string()).collect())
            );
        };
    }

    macro_rules! should_error {
        ($exports_field: expr, $request: expr, $condition_names: expr; $expect_msg: expr) => {
            assert_eq!(
                process_exports_fields!($exports_field, $request, $condition_names),
                Err($expect_msg.to_string())
            );
        };
    }

    should_equal!(json!({
        "./utils/": {
            "webpack": "./wpk/",
            "browser": ["lodash/", "./utils/"],
            "node": ["./utils/"]
        }
    }), "./utils/index.mjs", ["browser", "webpack"]; ["./wpk/index.mjs"]);

    should_equal!("./main.js", ".", []; ["./main.js"]);
    should_equal!("./main.js", "./main.js", []; []);
    should_equal!("./main.js", "./lib.js", []; []);
    should_equal!(["./a.js", "./b.js"], ".", []; ["./a.js", "./b.js"]);
    should_equal!(["./a.js", "./b.js"], "./a.js", []; []);
    should_equal!(["./a.js", "./b.js"], "./lib.js", []; []);
    should_equal!(json!({
        "./a/": "./A/",
        "./a/b/c": "./c.js",
      }), "./a/b/d.js", []; ["./A/b/d.js"]);
    should_equal!(json!({
        "./a/": "./A/",
        "./a/b": "./b.js",
      }), "./a/c.js", []; ["./A/c.js"]);
    should_equal!(json!({
        "./a/": "./A/",
        "./a/b/c/d": "./c.js",
      }), "./a/b/d/c.js", []; ["./A/b/d/c.js"]);
    should_equal!(json!({
        "./a/*": "./A/*",
        "./a/b/c": "./c.js",
      }), "./a/b/d.js", []; ["./A/b/d.js"]);
    should_equal!(json!({
        "./a/*": "./A/*",
        "./a/b": "./b.js",
      }), "./a/c.js", []; ["./A/c.js"]);
    should_equal!(json!({
        "./a/*": "./A/*",
        "./a/b/c/d": "./b.js",
     }), "./a/b/d/c.js", []; ["./A/b/d/c.js"]);
    should_equal!(json!({
        "./ab*": "./ab/*",
        "./abc*": "./abc/*",
        "./a*": "./a/*",
      }), "./abcd", ["browser"]; ["./abc/d"]);
    should_equal!(json!({
        "./ab*": "./ab/*",
        "./abc*": "./abc/*",
        "./a*": "./a/*",
      }), "./abcd", []; ["./abc/d"]);
    should_equal!(json!({
        "./ab*": "./ab/*",
        "./abc*": "./abc/*",
        "./a*": "./a/*",
      }), "./abcd/e", ["browser"]; ["./abc/d/e"]);
    should_equal!(json!({
        "./x/ab*": "./ab/*",
        "./x/abc*": "./abc/*",
        "./x/a*": "./a/*",
      }), "./x/abcd", ["browser"]; ["./abc/d"]);
    should_equal!(json!({
        "./x/ab*": "./ab/*",
        "./x/abc*": "./abc/*",
        "./x/a*": "./a/*",
      }), "./x/abcd/e", ["browser"]; ["./abc/d/e"]);
    should_equal!(json!({
        "browser": {
            "default": "./index.js"
        }
    }), "./lib.js", ["browser"]; []);
    should_equal!(json!({
        "browser": {
            "default": "./index.js"
        }
    }), ".", ["browser"]; ["./index.js"]);
    should_equal!(json!({
        "./foo/": {
            "import": ["./dist/", "./src/"],
            "webpack": "./wp/" 
        },
        ".": "./main.js"
    }), "./foo/test/file.js", ["import", "webpack"]; ["./dist/test/file.js", "./src/test/file.js"]);
    should_equal!(json!({
        "./foo/*": {
            "import": ["./dist/*", "./src/*"],
            "webpack": "./wp/*" 
        },
        ".": "./main.js"
    }), "./foo/test/file.js", ["import", "webpack"]; ["./dist/test/file.js", "./src/test/file.js"]);
    should_equal!(json!({
        "./timezones/": "./data/timezones/"
    }), "./timezones/pdt.mjs", []; ["./data/timezones/pdt.mjs"]);
    should_equal!(json!({
        "./": "./data/timezones/"
    }), "./timezones/pdt.mjs", []; ["./data/timezones/timezones/pdt.mjs"]);
    should_equal!(json!({
        "./*": "./data/timezones/*.mjs"
    }), "./timezones/pdt", []; ["./data/timezones/timezones/pdt.mjs"]);
    should_equal!(json!({
        "./lib/": {
            "browser": ["./browser/"]
        },
        "./dist/index.js": {
            "node": "./index.js"
        }
    }), "./dist/index.js", ["browser"]; []);
    should_equal!(json!({
        "./lib/*": {
            "browser": ["./browser/*"]
        },
        "./dist/index.js": {
            "node": "./index.js"
        }
    }), "./dist/index.js", ["browser"]; []);
    should_equal!(json!({
        "./lib/": {
            "browser": ["./browser/"]
        },
        "./dist/index.js": {
            "node": "./index.js",
            "default": "./browser/index.js"
        }
    }), "./dist/index.js", ["browser"]; ["./browser/index.js"]);
    should_equal!(json!({
        "./lib/*": {
            "browser": ["./browser/*"]
        },
        "./dist/index.js": {
            "node": "./index.js",
            "default": "./browser/index.js"
        }
    }), "./dist/index.js", ["browser"]; ["./browser/index.js"]);
    should_equal!(json!({
        "./dist/a": "./dist/index.js"
    }), "./dist/aaa", []; []);
    should_equal!(json!({
        "./dist/a/a/": "./dist/index.js"
    }), "./dist/a", []; []);
    should_equal!(json!({
        "./dist/a/a/*": "./dist/index.js"
    }), "./dist/a", []; []);
    should_equal!(json!({
        ".": "./index.js"
    }), "./timezones/pdt.mjs", []; []);
    should_equal!(json!({
        "./index.js": "./main.js"
    }), "./index.js", []; ["./main.js"]);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./#foo", []; ["./ok.js"]);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./bar#foo", []; ["./ok.js"]);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./#zapp/ok.js#abc", []; ["./ok.js#abc"]);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./#zapp/ok.js#abc", []; ["./ok.js#abc"]);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./#zapp/ok.js?abc", []; ["./ok.js?abc"]);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./#zapp/🎉.js", []; ["./🎉.js"]);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./#zapp/%F0%9F%8E%89.js", []; ["./%F0%9F%8E%89.js"]);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./🎉", []; ["./ok.js"]);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./%F0%9F%8E%89", []; ["./other.js"]);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./module", []; ["./ok.js"]);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./module#foo", []; []);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./zzz*"
    }), "./module?foo", []; []);
    should_equal!(json!({
        "./#foo": "./ok.js",
        "./module": "./ok.js",
        "./🎉": "./ok.js",
        "./%F0%9F%8E%89": "./other.js",
        "./bar#foo": "./ok.js",
        "./#zapp/": "./",
        "./#zipp*": "./z*z*z*"
    }), "./#zippi", []; ["./zizizi"]);
    should_equal!(json!({
        "./a?b?c/": "./"
    }), "./a?b?c/d?e?f", []; ["./d?e?f"]);
    should_equal!(json!({
        ".": "./dist/index.js"
    }), ".", []; ["./dist/index.js"]);
    should_equal!(json!({
        "./": "./",
        "./*": "./*",
        "./dist/index.js": "./dist/index.js",
    }), ".", []; []);
    should_equal!(json!({
        "./dist/": "./dist/",
        "./dist/*": "./dist/*",
        "./dist*": "./dist*",
        "./dist/index.js": "./dist/a.js"
    }), "./dist/index.js", []; ["./dist/a.js"]);
    should_equal!(json!({
        "./": {
            "browser": ["./browser/"]
        },
        "./*": {
            "browser": ["./browser/*"]
        },
        "./dist/index.js": {
            "browser": "./index.js"
        },
    }), "./dist/index.js", ["browser"]; ["./index.js"]);
    should_equal!(json!({
        "./a?b?c/": "./"
    }), "./a?b?c/d?e?f", []; ["./d?e?f"]);
    should_equal!(json!({
        "./": {
            "browser": ["./browser/"]
        },
        "./*": {
            "browser": ["./browser/*"]
        },
        "./dist/index.js": {
            "node": "./node.js"
        },
    }), "./dist/index.js", ["browser"]; []);
    should_equal!(json!({
        ".": {
            "browser": "./index.js",
            "node": "./src/node/index.js",
            "default": "./src/index.js"
        }
    }), ".", ["browser"]; ["./index.js"]);
    should_equal!(json!({
        ".": {
            "browser": "./index.js",
            "node": "./src/node/index.js",
            "default": "./src/index.js"
        }
    }), ".", []; ["./src/index.js"]);
    should_equal!(json!({
        ".": "./index" 
    }), ".", []; ["./index"]);
    should_equal!(json!({
        "./index": "./index.js" 
    }), "./index", []; ["./index.js"]);
    should_equal!(json!({
        ".": [
            { "browser": "./browser.js" },
			{ "require": "./require.js" },
    		{ "import": "./import.mjs" }
        ]
    }), ".", []; []);
    should_equal!(json!({
        ".": [
            { "browser": "./browser.js" },
			{ "require": "./require.js" },
    		{ "import": "./import.mjs" }
        ]
    }), ".", ["import"]; ["./import.mjs"]);
    should_equal!(json!({
        ".": [
            { "browser": "./browser.js" },
			{ "require": "./require.js" },
    		{ "import": "./import.mjs" }
        ]
    }), ".", ["import", "require"]; ["./require.js", "./import.mjs"]);
    should_equal!(json!({
        ".": [
            { "browser": "./browser.js" },
			{ "require": "./require.js" },
    		{ "import": ["./import.mjs", "./import.js"] }
        ]
    }), ".", ["import", "require"]; ["./require.js", "./import.mjs", "./import.js"]);
    should_equal!(json!({
        "./timezones": "./data/timezones",
    }), "./timezones/pdt.mjs", []; []);
    should_equal!(json!({
        "./timezones/pdt/": "./data/timezones/pdt/",
    }), "./timezones/pdt/index.mjs", []; ["./data/timezones/pdt/index.mjs"]);
    should_equal!(json!({
        "./timezones/pdt/*": "./data/timezones/pdt/*",
    }), "./timezones/pdt/index.mjs", []; ["./data/timezones/pdt/index.mjs"]);
    should_equal!(json!({
        "./": "./timezones/",
    }), "./pdt.mjs", []; ["./timezones/pdt.mjs"]);
    should_equal!(json!({
        "./*": "./timezones/*",
    }), "./pdt.mjs", []; ["./timezones/pdt.mjs"]);
    should_equal!(json!({
        "./": "./",
    }), "./timezones/pdt.mjs", []; ["./timezones/pdt.mjs"]);
    should_equal!(json!({
        "./*": "./*",
    }), "./timezones/pdt.mjs", []; ["./timezones/pdt.mjs"]);
    should_equal!(json!({
        ".": "./",
    }), "./timezones/pdt.mjs", []; []);
    should_equal!(json!({
        ".": "./*",
    }), "./timezones/pdt.mjs", []; []);
    should_equal!(json!({
        "./": "./",
        "./dist/": "./lib/"
    }), "./dist/index.mjs", []; ["./lib/index.mjs"]);
    should_equal!(json!({
        "./*": "./*",
        "./dist/*": "./lib/*"
    }), "./dist/index.mjs", []; ["./lib/index.mjs"]);
    should_equal!(json!({
        "./dist/utils/": "./dist/utils/",
        "./dist/": "./lib/"
    }), "./dist/utils/index.js", []; ["./dist/utils/index.js"]);
    should_equal!(json!({
        "./dist/utils/*": "./dist/utils/*",
        "./dist/*": "./lib/*"
    }), "./dist/utils/index.js", []; ["./dist/utils/index.js"]);
    should_equal!(json!({
        "./dist/utils/index.js": "./dist/utils/index.js",
        "./dist/utils/": "./dist/utils/index.mjs",
        "./dist/": "./lib/"
    }), "./dist/utils/index.js", []; ["./dist/utils/index.js"]);
    should_equal!(json!({
        "./dist/utils/index.js": "./dist/utils/index.js",
        "./dist/utils/*": "./dist/utils/index.mjs",
        "./dist/*": "./lib/*"
    }), "./dist/utils/index.js", []; ["./dist/utils/index.js"]);
    should_equal!(json!({
        "./": {
            "browser": "./browser/"
        },
        "./dist/": "./lib/"
    }), "./dist/index.mjs", ["browser"]; ["./lib/index.mjs"]);
    should_equal!(json!({
        "./*": {
            "browser": "./browser/*"
        },
        "./dist/*": "./lib/*"
    }), "./dist/index.mjs", ["browser"]; ["./lib/index.mjs"]);
    should_equal!(json!({
        "./utils/": {
            "browser": ["lodash/", "./utils/"],
            "node": ["./utils-node/"]
        }
    }), "./utils/index.js", ["browser"]; ["lodash/index.js", "./utils/index.js"]);
    should_equal!(json!({
        "./utils/*": {
            "browser": ["lodash/*", "./utils/*"],
            "node": ["./utils-node/*"]
        }
    }), "./utils/index.js", ["browser"]; ["lodash/index.js", "./utils/index.js"]);
    should_equal!(json!({
        "./utils/": {
            "webpack": "./wpk/",
            "browser": ["lodash/", "./utils/"],
            "node": ["./node/"]
        }
    }), "./utils/index.mjs", []; []);
    should_equal!(json!({
        "./utils/*": {
            "webpack": "./wpk/*",
            "browser": ["lodash/*", "./utils/*"],
            "node": ["./node/*"]
        }
    }), "./utils/index.mjs", []; []);
    should_equal!(json!({
        "./utils/": {
            "webpack": "./wpk/",
            "browser": ["lodash/", "./utils/"],
            "node": ["./utils/"]
        }
    }), "./utils/index.mjs", ["browser", "webpack"]; ["./wpk/index.mjs"]);
    should_equal!(json!({
        "./utils/*": {
            "webpack": "./wpk/*",
            "browser": ["lodash/*", "./utils/*"],
            "node": ["./utils/*"]
        }
    }), "./utils/index.mjs", ["browser", "webpack"]; ["./wpk/index.mjs"]);
    should_equal!(json!({
        "./utils/index": "./a/index.js" 
      }), "./utils/index.mjs", []; []);
    should_equal!(json!({
        "./utils/index.mjs": "./a/index.js" 
      }), "./utils/index", []; []);
    should_equal!(json!({
        "./utils/index": {
            "browser": "./a/index.js",
            "default": "./b/index.js",
        } 
      }), "./utils/index.mjs", ["browser"]; []);
    should_equal!(json!({
        "./utils/index.mjs": {
            "browser": "./a/index.js",
            "default": "./b/index.js",
        } 
      }), "./utils/index", ["browser"]; []);
    should_equal!(json!({
        "./../../utils/": "./dist/"
      }), "./../../utils/index", []; ["./dist/index"]);
    should_equal!(json!({
        "./../../utils/*": "./dist/*"
      }), "./../../utils/index", []; ["./dist/index"]);
    should_equal!(json!({
        "./utils/": "./../src/"
      }), "./utils/index", []; ["./../src/index"]);
    should_equal!(json!({
        "./utils/*": "./../src/*"
      }), "./utils/index", []; ["./../src/index"]);
    should_equal!(json!({
        "./utils/index": "./src/../index.js"
      }), "./utils/index", []; ["./src/../index.js"]);
    should_equal!(json!({
        "./utils/../utils/index": "./src/../index.js"
      }), "./utils/index", []; []);
    should_equal!(json!({
        "./utils/": {
            "browser": "./utils/../"
        }
    }), "./utils/index", ["browser"]; ["./utils/../index"]);
    should_equal!(json!({
        "./": "./src/../../",
        "./dist/": "./dist/"
    }), "./dist/index", ["browser"]; ["./dist/index"]);
    should_equal!(json!({
        "./*": "./src/../../*",
        "./dist/*": "./dist/*"
    }), "./dist/index", ["browser"]; ["./dist/index"]);
    should_equal!(json!({
        "./utils/": "./dist/"
    }), "./utils/timezone/../../index", []; ["./dist/timezone/../../index"]);
    should_equal!(json!({
        "./utils/*": "./dist/*"
    }), "./utils/timezone/../../index", []; ["./dist/timezone/../../index"]);
    should_equal!(json!({
        "./utils/": "./dist/"
    }), "./utils/timezone/../index", []; ["./dist/timezone/../index"]);
    should_equal!(json!({
        "./utils/*": "./dist/*"
    }), "./utils/timezone/../index", []; ["./dist/timezone/../index"]);
    should_equal!(json!({
        "./utils/": "./dist/target/"
    }), "./utils/../../index", []; ["./dist/target/../../index"]);
    should_equal!(json!({
        "./utils/*": "./dist/target/*"
    }), "./utils/../../index", []; ["./dist/target/../../index"]);
    should_equal!(json!({
        "./utils/": {
            "browser": "./node_modules/"
        }
    }), "./utils/lodash/dist/index.js", ["browser"]; ["./node_modules/lodash/dist/index.js"]);
    should_equal!(json!({
        "./utils/*": {
            "browser": "./node_modules/*"
        }
    }), "./utils/lodash/dist/index.js", ["browser"]; ["./node_modules/lodash/dist/index.js"]);
    should_equal!(json!({
        "./utils/": "./utils/../node_modules/"
    }), "./utils/lodash/dist/index.js", ["browser"]; ["./utils/../node_modules/lodash/dist/index.js"]);
    should_equal!(json!({
        "./utils/*": "./utils/../node_modules/*"
    }), "./utils/lodash/dist/index.js", ["browser"]; ["./utils/../node_modules/lodash/dist/index.js"]);
    should_equal!(json!({
        "./utils/": {
            "browser": {
                "webpack": "./",
                "default": {
                    "node": "./node/"
                }
            }
        }
    }), "./utils/index.js", ["browser"]; []);
    should_equal!(json!({
        "./utils/*": {
            "browser": {
                "webpack": "./*",
                "default": {
                    "node": "./node/*"
                }
            }
        }
    }), "./utils/index.js", ["browser"]; []);
    should_equal!(json!({
        "./utils/": {
            "browser": {
                "webpack": ["./", "./node/"],
                "default": {
                    "node": "./node/"
                }
            }
        }
    }), "./utils/index.js", ["browser", "webpack"]; ["./index.js", "./node/index.js"]);
    should_equal!(json!({
        "./utils/*": {
            "browser": {
                "webpack": ["./*", "./node/*"],
                "default": {
                    "node": "./node/*"
                }
            }
        }
    }), "./utils/index.js", ["browser", "webpack"]; ["./index.js", "./node/index.js"]);
    should_equal!(json!({
        "./utils/": {
            "browser": {
                "webpack": ["./", "./node/"],
                "default": {
                    "node": "./node/"
                }
            }
        }
    }), "./utils/index.js", ["webpack"]; []);
    should_equal!(json!({
        "./utils/*": {
            "browser": {
                "webpack": ["./*", "./node/*"],
                "default": {
                    "node": "./node/*"
                }
            }
        }
    }), "./utils/index.js", ["webpack"]; []);
    should_equal!(json!({
        "./utils/": {
            "browser": {
                "webpack": ["./", "./node/"],
                "default": {
                    "node": "./node/"
                }
            }
        }
    }), "./utils/index.js", ["node", "browser"]; ["./node/index.js"]);
    should_equal!(json!({
        "./utils/*": {
            "browser": {
                "webpack": ["./*", "./node/*"],
                "default": {
                    "node": "./node/*"
                }
            }
        }
    }), "./utils/index.js", ["node", "browser"]; ["./node/index.js"]);
    should_equal!(json!({
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
    }), "./utils/index.js", ["browser", "node"]; []);
    should_equal!(json!({
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
    }), "./utils/index.js", ["browser", "node"]; []);
    should_equal!(json!({
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
    }), "./utils/index.js", ["browser", "node", "webpack"]; ["./index.js", "./node/index.js"]);
    should_equal!(json!({
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
    }), "./utils/index.js", ["browser", "node", "webpack"]; ["./index.js", "./node/index.js"]);
    should_equal!(json!({
        "./a.js": {
            "abc": {
                "def": "./x.js"
            },
            "ghi": "./y.js"
        }
    }), "./a.js", ["abc", "ghi"]; ["./y.js"]);
    should_equal!(json!({
        "./a.js": {
            "abc": {
                "def": "./x.js",
                "default": []
            },
            "ghi": "./y.js"
        }
    }), "./a.js", ["abc", "ghi"]; []);

    should_error!(json!({
        "./utils/": {
            "browser": "../this/"
        }
    }), "./utils/index", ["browser"]; "Export should be relative path and start with \"./\", but got ../this/");
    should_error!(json!({
        "./utils/*": {
            "browser": "../this/*"
        }
    }), "./utils/index", ["browser"]; "Export should be relative path and start with \"./\", but got ../this/*");
    should_error!(json!({
        ".": {
            "default": "./src/index.js",
            "browser": "./index.js",
            "node": "./src/node/index.js"
        }
      }), ".", ["browser"]; "Default condition should be last one");
    should_error!(json!({
        "./*": "."
    }), "./timezones/pdt.mjs", []; "Export should be relative path and start with \"./\", but got .");
    should_error!(json!({
        "./": "."
    }), "./timezones/pdt.mjs", []; "Export should be relative path and start with \"./\", but got .");
    should_error!(json!({
        "./timezones/": "./data/timezones"
    }), "./timezones/pdt.mjs", []; "Expected ./data/timezones is folder mapping");
    should_error!(json!({
        "./node": "./node.js",
        "browser": {
          "default": "./index.js"
        },
      }), ".", ["browser"]; "Export field key can't mixed relative path and conditional object");
    should_error!(json!({
        "browser": {
          "default": "./index.js"
        },
        "./node": "./node.js"
      }), ".", ["browser"]; "Export field key can't mixed relative path and conditional object");
    should_error!(json!({
        "/utils/": "./a/" 
      }), "./utils/index.mjs", []; "Export field key should be relative path and start with \"./\", but got /utils/");
    should_error!(json!({
        "./utils/": "/a/" 
      }), "./utils/index.mjs", []; "Export should be relative path and start with \"./\", but got /a/");
    should_error!(json!({
        "./utils/": "./a/" 
      }), "/utils/index.mjs", []; "Request should be relative path and start with '.', but got /utils/index.mjs");
    should_error!(json!({
        "./utils/": {
            "browser": "./a/",
            "default": "./b/"
        }
      }), "/utils/index.mjs", ["browser"]; "Request should be relative path and start with '.', but got /utils/index.mjs");
    should_error!(json!({
        "./utils/": {
            "browser": "./a/",
            "default": "./b/"
        }
      }), "/utils/index.mjs/", ["browser"]; "Request should be relative path and start with '.', but got /utils/index.mjs/");
    should_error!(json!({
        "./utils/": {
            "browser": "./a/",
            "default": "./b/"
        }
      }), "../utils/index.mjs", ["browser"]; "Request should be relative path and start with '.', but got ../utils/index.mjs");
    should_error!(json!({
        "../../utils/": "./dist/"
    }), "../../utils/index", []; "Export field key should be relative path and start with \"./\", but got ../../utils/");
    should_error!(json!({
        "../../utils/*": "./dist/*"
    }), "../../utils/index", []; "Export field key should be relative path and start with \"./\", but got ../../utils/*");
    should_error!(json!({
        "./utils/": "../src/"
    }), "./utils/index", []; "Export should be relative path and start with \"./\", but got ../src/");
    should_error!(json!({
        "./utils/*": "../src/*"
    }), "./utils/index", []; "Export should be relative path and start with \"./\", but got ../src/*");
    should_error!(json!({
        "/utils/": {
            "browser": "./a/",
            "default": "./b/"
        }
    }), "./utils/index.mjs", ["browser"]; "Export field key should be relative path and start with \"./\", but got /utils/");

    should_error!(json!({
        "./utils/": {
            "browser": "/a/",
            "default": "/b/"
        }
    }), "./utils/index.mjs", ["browser"]; "Export should be relative path and start with \"./\", but got /a/");
    should_error!(json!({
        "./utils/*": {
            "browser": "/a/",
            "default": "/b/"
        }
    }), "./utils/index.mjs", ["browser"]; "Export should be relative path and start with \"./\", but got /a/");
}
