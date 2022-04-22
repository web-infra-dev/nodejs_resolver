// use std::collections::{HashMap, HashSet};

// use crate::RResult;

// pub type DirectMapping = String;
// pub type ConditionalMapping = HashMap<String, MappingValue>;

// #[derive(Clone)]
// pub enum AvaliableMapping {
//     Direct(DirectMapping),
//     Conditional(ConditionalMapping),
// }
// pub type ArrayMapping = Vec<AvaliableMapping>;

// #[derive(Clone)]
// pub enum MappingValue {
//     Direct(DirectMapping),
//     Conditional(ConditionalMapping),
//     Array(ArrayMapping),
// }

// pub type ImportsField = HashMap<String, MappingValue>;
// pub enum ExportsField {
//     Field(HashMap<String, MappingValue>),
//     Conditional(ConditionalMapping),
//     Array(ArrayMapping),
//     Direct(DirectMapping),
// }

// pub enum PathTreeNodeKind {
//     Imports,
//     Exports,
// }

// pub struct PathTreeNode {
//     pub children: Option<HashMap<String, PathTreeNode>>,
//     pub folder: Option<MappingValue>,
//     pub wildcards: Option<HashMap<String, MappingValue>>,
//     pub files: HashMap<String, MappingValue>,
//     pub kind: PathTreeNodeKind,
// }

// trait Field {
//     fn assert_target(exp: &str, expect_folder: bool) -> RResult<()>;
//     fn assert_field_request(request: &str) -> RResult<&str>;
//     fn build_field_path_tree(filed: Self) -> RResult<PathTreeNode>;
// }

// impl Field for ExportsField {
//     fn assert_field_request(request: &str) -> RResult<&str> {
//         if !request.starts_with('.') {
//             Err("Request should be relative path and start with '.'".to_string())
//         } else if request.len() == 1 {
//             Ok("")
//         } else if !request.starts_with("./") {
//             Err("Request should be relative path and start with './'".to_string())
//         } else if request.ends_with('/') {
//             Err("Only requesting file allowed".to_string())
//         } else {
//             Ok(&request[2..])
//         }
//     }

//     fn assert_target(exp: &str, expect_folder: bool) -> RResult<()> {
//         if exp.starts_with('/') || (exp.starts_with('.') && !exp.starts_with("./")) {
//             Err(format!(
//                 "Export should be relative path and start with \"./\", got {exp}"
//             ))
//         } else if exp.ends_with('/') != expect_folder {
//             if expect_folder {
//                 Err(format!("Export should be folder, got {exp}"))
//             } else {
//                 Err(format!("Export should be file, got {exp}"))
//             }
//         } else {
//             Ok(())
//         }
//     }

//     /// reference: https://nodejs.org/api/packages.html#exports
//     fn build_field_path_tree(field: Self) -> RResult<PathTreeNode> {
//         let mut root = PathTreeNode::create_node(PathTreeNodeKind::Exports);
//         match field {
//             Self::Field(map) | Self::Conditional(map) => {
//                 for (key, value) in map {
//                     if key == "." {
//                         root.files
//                             .insert("".to_string(), MappingValue::Direct('.'.to_string()));
//                     }
//                     if !key.starts_with("./") {
//                         return Err(
//                             "Export field key should be relative path and start with \"./\""
//                                 .to_string(),
//                         );
//                     }
//                     root.files
//                         .insert("".to_string(), MappingValue::Conditional(value));
//                 }
//             }
//             Self::Array(array) => {
//                 root.files
//                     .insert("".to_string(), MappingValue::Array(array));
//             }
//             Self::Direct(direct) => {
//                 root.files
//                     .insert("".to_string(), MappingValue::Direct(direct));
//             }
//         }
//         Ok(root)
//     }
// }

// impl PathTreeNode {
//     pub fn create_node(kind: PathTreeNodeKind) -> Self {
//         PathTreeNode {
//             children: None,
//             folder: None,
//             wildcards: None,
//             files: HashMap::new(),
//             kind,
//         }
//     }
// }

// fn find_match<'a>(root: &'a PathTreeNode, request: &'a str) -> Option<(&'a MappingValue, i32)> {
//     if request.is_empty() {
//         root.files.get("").and_then(|value| Some((value, 1)))
//     } else {
//         None
//     }
// }

// fn field_processor<'a>(
//     root: &'a PathTreeNode,
//     request: &'a str,
//     condition_names: &'a HashSet<String>,
// ) -> RResult<Vec<()>> {
//     let request =   (request)?;
//     let (mapping, remain_request_index) = match find_match(root, request) {
//         Some(result) => result,
//         None => return Ok(vec![]),
//     };

//     match mapping {
//         MappingValue::Array(array) => {}
//         MappingValue::Direct(directory) => todo!(),
//         MappingValue::Conditional(conditional) => todo!(),
//     };

//     let remaining_request = if remain_request_index == (request.len() as i32) + 1 {
//         return Ok(vec![]);
//     } else if remain_request_index < 0 {
//         &request[(remain_request_index.abs() - 1) as usize..]
//     } else {
//         &request[remain_request_index as usize..]
//     };

//     Ok(vec![])
// }

// fn direct_mapping(
//     remaining_request: &str,
//     subpath_mapping: bool,
//     mapping_target: Option<DirectMapping>,
//     condition_names: &HashSet<String>,
// ) -> RResult<Vec<()>> {
//     let mut result = vec![];
//     if subpath_mapping {
//         result.push(());
//     }
//     Ok(result)
// }

// pub fn build_imports_field_path_tree(filed: ImportsField) -> RResult<PathTreeNode> {
//     let mut root = PathTreeNode::create_node(PathTreeNodeKind::Imports);
//     Ok(root)
// }

// #[test]
// fn map_test() {
//     println!("{:?}", file!());
//     assert!(false);

//     let fileds = r#"
//   {
//     "./utils/index.mjs": "./a/index.js"
//   }"#;
//     // let expect = ["./A/b/d.js"];
//     // let suite = [
//     //   {
//     //     "./a/": "./A/",
//     //     "./a/b/c": "./c.js"
//     //   },
//     //   "./a/b/d.js",
//     //   []
//     // ]
//     // suite[0]
// }
