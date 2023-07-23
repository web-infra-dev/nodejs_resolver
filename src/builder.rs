use crate::cache::Cache;
use crate::log;
use crate::options::EnforceExtension::{Auto, Disabled, Enabled};
use crate::{fs_interface::FileSystem, Options, Resolver};

#[derive(Debug)]
pub struct ResolverBuilder {
    fs: Box<dyn FileSystem>,
}

impl ResolverBuilder {
    pub fn new(fs: Box<dyn FileSystem>) -> Self {
        ResolverBuilder { fs }
    }

    pub fn build(self, options: Options) -> Resolver {
        log::enable_by_env();

        let cache = if let Some(external_cache) = options.external_cache.as_ref() {
            external_cache.clone()
        } else {
            std::sync::Arc::new(Cache::default())
        };

        let enforce_extension = match options.enforce_extension {
            Auto => {
                if options.extensions.iter().any(|ext| ext.is_empty()) {
                    Enabled
                } else {
                    Disabled
                }
            }
            _ => options.enforce_extension,
        };

        let options = Options { enforce_extension, ..options };
        Resolver { options, cache, fs: self.fs }
    }
}
