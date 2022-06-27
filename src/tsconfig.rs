// copy from https://github.com/drivasperez/tsconfig

use crate::{RResult, Resolver, ResolverInfo, ResolverResult, ResolverStats};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TsConfig {
    pub extends: Option<String>,
    pub compiler_options: Option<CompilerOptions>,
}

#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub base_url: Option<String>,
    pub paths: Option<HashMap<String, Vec<String>>>,
}

impl TsConfig {
    pub fn parse_file(location: &Path, resolver: &Resolver) -> RResult<TsConfig> {
        let json = parse_file_to_value(location, resolver)?;
        let compiler_options = json.get("compilerOptions").map(|options| {
            // TODO: should optimized
            let base_url = options
                .get("baseUrl")
                .map(|v| v.as_str().unwrap().to_string());
            let paths = options.get("paths").map(|v| {
                let mut map = HashMap::new();
                // TODO: should optimized
                for (key, obj) in v.as_object().unwrap() {
                    map.insert(
                        key.to_string(),
                        obj.as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_str().unwrap().to_string())
                            .collect(),
                    );
                }
                map
            });
            CompilerOptions { base_url, paths }
        });
        let extends: Option<String> = json.get("extends").map(|v| v.to_string());
        Ok(TsConfig {
            extends,
            compiler_options,
        })
    }
}

fn parse_file_to_value(location: &Path, resolver: &Resolver) -> RResult<serde_json::Value> {
    let json_str =
        read_to_string(location).map_err(|_| format!("Open {} failed", location.display()))?;
    let mut json: serde_json::Value = jsonc_parser::parse_to_serde_value(&json_str)
        .map_err(|err| format!("Parse {} failed. Error: {err}", location.display()))?
        .unwrap();

    // merge `extends`.
    if let serde_json::Value::String(s) = &json["extends"] {
        // `location` pointed to `dir/tsconfig.json`
        let dir = location.parent().unwrap().to_path_buf();
        let stats = resolver._resolve(ResolverInfo::from(dir, resolver.parse(s)));
        // Is it better to use cache?
        if let ResolverStats::Success(result) = stats {
            let extends_tsconfig_json = match result {
                ResolverResult::Info(info) => parse_file_to_value(&info.get_path(), resolver),
                ResolverResult::Ignored => {
                    return Err(format!("{s} had been ignored in {}", location.display()))
                }
            }?;
            merge(&mut json, extends_tsconfig_json);
        }
    }
    Ok(json)
}

fn merge(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) {
        (&mut serde_json::Value::Object(ref mut a), serde_json::Value::Object(b)) => {
            for (k, v) in b {
                merge(a.entry(k).or_insert(serde_json::Value::Null), v);
            }
        }
        (a, b) => {
            if let serde_json::Value::Null = a {
                *a = b;
            }
        }
    }
}
