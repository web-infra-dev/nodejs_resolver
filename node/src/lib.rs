use napi::bindgen_prelude::External;
use napi_derive::napi;
use nodejs_resolver::{Resolver, ResolverOptions};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawResolverOptions {
    pub extensions: Option<Vec<String>>,
    pub enforce_extension: Option<Option<bool>>,
    pub alias: Option<HashMap<String, Option<String>>>,
    pub alias_fields: Option<Vec<String>>,
    pub condition_names: Option<HashSet<String>>,
    pub symlinks: Option<bool>,
    pub description_file: Option<Option<String>>,
    pub main_files: Option<Vec<String>>,
    pub main_fields: Option<Vec<String>>,
    pub modules: Option<Vec<String>>,
    pub prefer_relative: Option<bool>,
    pub enable_unsafe_cache: Option<bool>,
}

impl RawResolverOptions {
    pub fn normalized(&self) -> ResolverOptions {
        let default = ResolverOptions::default();
        ResolverOptions {
            enforce_extension: self.enforce_extension.unwrap_or(default.enforce_extension),
            extensions: self.extensions.to_owned().unwrap_or(default.extensions),
            alias: self.alias.to_owned().unwrap_or(default.alias),
            alias_fields: self.alias_fields.to_owned().unwrap_or(default.alias_fields),
            condition_names: self
                .condition_names
                .to_owned()
                .unwrap_or(default.condition_names),
            symlinks: self.symlinks.unwrap_or(default.symlinks),
            description_file: self
                .description_file
                .to_owned()
                .unwrap_or(default.description_file),
            main_files: self.main_files.to_owned().unwrap_or(default.main_files),
            main_fields: self.main_fields.to_owned().unwrap_or(default.main_fields),
            modules: self.modules.to_owned().unwrap_or(default.modules),
            prefer_relative: self.prefer_relative.unwrap_or(default.prefer_relative),
            enable_unsafe_cache: self
                .enable_unsafe_cache
                .unwrap_or(default.enable_unsafe_cache),
        }
    }
}

#[napi(object)]
pub struct ResolverInternal {}

#[napi(ts_return_type = "ExternalObject<ResolverInternal>")]
pub fn create(options: String) -> Result<External<Resolver>, napi::Error> {
    let options: RawResolverOptions = serde_json::from_str(options.as_str())
        .map_err(|err| napi::Error::new(napi::Status::InvalidArg, err.to_string()))?;
    let resolver = Resolver::new(options.normalized());
    Ok(External::new(resolver))
}

#[napi(object)]
pub struct ResolveResult {
    pub status: bool,
    pub path: Option<String>,
}

#[napi]
pub fn resolve(
    resolver: External<Resolver>,
    base_dir: String,
    id: String,
) -> Result<ResolveResult, napi::Error> {
    match (*resolver).resolve(Path::new(&base_dir), &id) {
        Ok(val) => {
            if let nodejs_resolver::ResolveResult::Path(p) = val {
                Ok(ResolveResult {
                    status: true,
                    path: Some(p.to_string_lossy().to_string()),
                })
            } else {
                Ok(ResolveResult {
                    status: false,
                    path: None,
                })
            }
        }
        Err(err) => Err(napi::Error::new(napi::Status::GenericFailure, err)),
    }
}
