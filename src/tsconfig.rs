// copy from https://github.com/drivasperez/tsconfig

use crate::context::Context;
use crate::{Error, Info, RResult, ResolveResult, Resolver, State};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct TsConfig {
    pub extends: Option<String>,
    pub compiler_options: Option<CompilerOptions>,
}

#[derive(Debug, Clone)]
pub struct CompilerOptions {
    pub base_url: Option<String>,
    pub paths: Option<HashMap<String, Vec<String>>>,
}

impl Resolver {
    pub(super) fn parse_ts_file(
        &self,
        location: &Path,
        context: &mut Context,
    ) -> RResult<TsConfig> {
        let json = self.parse_file_to_value(location, context)?;
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

    fn parse_file_to_value(
        &self,
        location: &Path,
        context: &mut Context,
    ) -> RResult<serde_json::Value> {
        let entry = match self.load_entry(location) {
            Ok(entry) => entry,
            Err(error) => return Err(error),
        };
        if !entry.is_file() {
            // Its role is to ensure that `stat` exists
            return Err(Error::CantFindTsConfig);
        }
        let reader = entry.stat.read().unwrap();
        let stat = reader.as_ref().unwrap();
        let json_str = self.cache.fs.read_file(location, stat).map_err(Error::Io)?;
        // TODO: should cache `json_str` -> TsConfig
        let mut json: serde_json::Value =
            jsonc_parser::parse_to_serde_value(&json_str, &Default::default())
                .map_err(|err| {
                    Error::UnexpectedValue(format!(
                        "Parse {} failed. Error: {err}",
                        location.display()
                    ))
                })?
                .unwrap_or_else(|| panic!("Transfer {} to serde value failed", location.display()));

        // merge `extends`.
        if let serde_json::Value::String(s) = &json["extends"] {
            // `location` pointed to `dir/tsconfig.json`
            let dir = location.parent().unwrap().to_path_buf();
            let request = self.parse(s);
            let state = self._resolve(Info::from(dir, request), context);
            // Is it better to use cache?
            if let State::Success(result) = state {
                let extends_tsconfig_json = match result {
                    ResolveResult::Info(info) => {
                        self.parse_file_to_value(&info.get_path(), context)
                    }
                    ResolveResult::Ignored => {
                        return Err(Error::UnexpectedValue(format!(
                            "{s} had been ignored in {}",
                            location.display()
                        )))
                    }
                }?;
                merge(&mut json, extends_tsconfig_json);
            }
        }
        Ok(json)
    }
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
