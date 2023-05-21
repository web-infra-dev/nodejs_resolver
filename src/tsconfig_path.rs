// Copy from https://github.com/dividab/tsconfig-paths

use crate::{context::Context, Info, RResult, Resolver, State};
use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
pub struct TsConfigInfo {
    pub paths: Option<FxHashMap<String, Vec<String>>>,
    pub base_url: Option<String>,
}

#[derive(Debug, PartialEq)]
struct MappingEntry {
    pub(crate) pattern: String,
    /// The item in `paths` maybe contains '*' tag
    pub(crate) paths: Vec<PathBuf>,
}

impl Resolver {
    fn get_absolute_mapping_entries(
        absolute_base_url: &Path,
        paths: &FxHashMap<String, Vec<String>>,
    ) -> Vec<MappingEntry> {
        paths
            .iter()
            .map(|(key, paths)| {
                let pattern = key.to_string();
                let paths = paths
                    .iter()
                    .map(|path| absolute_base_url.join(path))
                    .collect();
                MappingEntry { pattern, paths }
            })
            .collect()
    }

    fn parse_tsconfig(&self, location: &Path, context: &mut Context) -> RResult<TsConfigInfo> {
        let tsconfig = self.parse_ts_file(location, context)?;
        let base_url = tsconfig
            .compiler_options
            .as_ref()
            .and_then(|options| options.base_url.clone());
        let paths = tsconfig.compiler_options.and_then(|options| options.paths);
        Ok(TsConfigInfo { paths, base_url })
    }

    fn match_star<'a>(pattern: &'a str, search: &'a str) -> Option<&'a str> {
        if search.len() < pattern.len() {
            return None;
        }
        if pattern == "*" {
            return Some(search);
        }
        let p = pattern.as_bytes();
        let s = search.as_bytes();
        p.iter()
            .enumerate()
            .find(|&(_, &c)| c == b'*')
            .and_then(|(star_index, _)| {
                let part1 = &p[..star_index];
                if &s[0..star_index] != part1 {
                    return None;
                }
                let part2 = &p[star_index + 1..];
                if &s[s.len() - part2.len()..] != part2 {
                    return None;
                }
                let len = s.len() - part2.len() - part1.len();
                match std::str::from_utf8(&s[star_index..star_index + len]) {
                    Ok(result) => Some(result),
                    Err(error) => {
                        panic!(
                            "There has unexpected error when matching {pattern} and {search}, error: {error}"
                        )
                    }
                }
            })
    }

    fn create_match_list(
        absolute_base_url: &Path,
        paths: &Option<FxHashMap<String, Vec<String>>>,
    ) -> Vec<MappingEntry> {
        paths
            .as_ref()
            .map(|paths| Self::get_absolute_mapping_entries(absolute_base_url, paths))
            .unwrap_or_default()
    }

    pub(super) fn _resolve_with_tsconfig(
        &self,
        info: Info,
        location: &Path,
        context: &mut Context,
    ) -> State {
        let tsconfig = match self.parse_tsconfig(location, context) {
            Ok(tsconfig) => tsconfig,
            Err(error) => return State::Error(error),
        };
        let location_dir = location.parent().unwrap();
        let absolute_base_url = if let Some(base_url) = tsconfig.base_url.as_ref() {
            location_dir.join(base_url)
        } else {
            location_dir.into()
        };

        // resolve absolute path that relative from base_url
        if tsconfig.base_url.is_some() && !info.request().target().starts_with('.') {
            let target = absolute_base_url.join(info.request().target());
            let info = info.clone().with_path(target).with_target("");
            let result = self._resolve(info, context);
            if result.is_finished() {
                return result;
            }
        }

        let absolute_path_mappings =
            Resolver::create_match_list(&absolute_base_url, &tsconfig.paths);

        for entry in absolute_path_mappings {
            let star_match = if entry.pattern == info.request().target() {
                ""
            } else if let Some(s) = Self::match_star(&entry.pattern, info.request().target()) {
                s
            } else {
                continue;
            };

            for physical_path_pattern in &entry.paths {
                let physical_path = &physical_path_pattern
                    .display()
                    .to_string()
                    .replace('*', star_match);
                let info = info.clone().with_path(physical_path).with_target("");
                let result = self._resolve(info, context);
                if result.is_finished() {
                    return result;
                }
            }
        }
        self._resolve(info, context)
    }
}

#[test]
fn test_get_absolute_mapping_entries() {
    let result = Resolver::get_absolute_mapping_entries(
        Path::new("/absolute/base/url"),
        &FxHashMap::from_iter(vec![
            (
                "*".to_string(),
                (vec!["/foo1", "./foo2"])
                    .into_iter()
                    .map(String::from)
                    .collect(),
            ),
            (
                "longest/pre/fix/*".to_string(),
                vec!["./foo2/bar".to_string()],
            ),
            ("pre/fix/*".to_string(), vec!["/foo3".to_string()]),
        ]),
    );
    assert!(result.len() == 3);
    assert!(result.contains(&MappingEntry {
        pattern: "longest/pre/fix/*".to_string(),
        paths: vec![PathBuf::from("/absolute/base/url/foo2/bar")],
    }));
    assert!(result.contains(&MappingEntry {
        pattern: "pre/fix/*".to_string(),
        paths: vec![PathBuf::from("/foo3")],
    },));
    assert!(result.contains(&MappingEntry {
        pattern: "*".to_string(),
        paths: vec![
            PathBuf::from("/foo1"),
            PathBuf::from("/absolute/base/url/foo2")
        ],
    }));

    let result = Resolver::get_absolute_mapping_entries(
        Path::new("/absolute/base/url"),
        &FxHashMap::from_iter([]),
    );
    assert!(result.is_empty());
}

#[test]
fn test_match_star() {
    // should not panic
    assert_eq!(Resolver::match_star("abc/*", "./中文"), None)
}
