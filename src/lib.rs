use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

pub struct Resolver {
    pub extensions: Vec<String>,
    build_in_node_modules: HashSet<String>,
    base_dir: PathBuf,
}

type ResolverError = String;
type ResolverResult = Result<PathBuf, ResolverError>;

impl Resolver {
    pub fn new(base_dir: PathBuf) -> Self {
        let extensions = (vec!["js", "json", "node"])
            .iter()
            .map(|&s| s.into())
            .collect();
        let build_in_node_modules = HashSet::from_iter(
            [
                "_http_agent",
                "_http_client",
                "_http_common",
                "_http_incoming",
                "_http_outgoing",
                "_http_server",
                "_stream_duplex",
                "_stream_passthrough",
                "_stream_readable",
                "_stream_transform",
                "_stream_wrap",
                "_stream_writable",
                "_tls_common",
                "_tls_wrap",
                "assert",
                "assert/strict",
                "async_hooks",
                "buffer",
                "child_process",
                "cluster",
                "console",
                "constants",
                "crypto",
                "dgram",
                "diagnostics_channel",
                "dns",
                "dns/promises",
                "domain",
                "events",
                "fs",
                "fs/promises",
                "http",
                "http2",
                "https",
                "inspector",
                "module",
                "net",
                "os",
                "path",
                "path/posix",
                "path/win32",
                "perf_hooks",
                "process",
                "punycode",
                "querystring",
                "readline",
                "repl",
                "stream",
                "stream/consumers",
                "stream/promises",
                "stream/web",
                "string_decoder",
                "sys",
                "timers",
                "timers/promises",
                "tls",
                "trace_events",
                "tty",
                "url",
                "util",
                "util/types",
                "v8",
                "vm",
                "wasi",
                "worker_threads",
                "zlib",
            ]
            .iter()
            .map(|&s| s.into()),
        );
        Self {
            base_dir,
            extensions,
            build_in_node_modules,
        }
    }

    pub fn resolve(&self, target: &str) -> ResolverResult {
        if self.is_absolute_path(target) {
            let path = &Path::new("/").join(target);
            self.resolve_as_file_path(path)
        } else if self.is_relative_path(target) {
            let path = &self.base_dir.join(target);
            self.resolve_as_file_path(path)
        } else if self.is_build_in_module(target) {
            Ok(PathBuf::from(target))
        } else {
            // it should be located at node_modules
            self.resolve_as_node_modules(target)
        }
    }

    fn resolve_as_file_path(&self, path: &Path) -> ResolverResult {
        if path.is_file() {
            Ok(path.to_path_buf())
        } else {
            let mut path = path.to_path_buf();
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(String::from)
                .unwrap();
            for extensions in &self.extensions {
                path.set_file_name(format!("{}.{}", file_name, extensions));
                if path.is_file() {
                    return Ok(path);
                }
            }
            Err("Not found file".to_string())
        }
    }

    fn resolve_as_node_modules(&self, target: &str) -> ResolverResult {
        let node_modules = self.base_dir.join("node_modules");
        if node_modules.is_dir() {
            let path = node_modules.join(target);
            let result = self.resolve_as_file_path(&path);
            if result.is_ok() {
                return result;
            }
        }
        match self.base_dir.parent() {
            Some(parent_dir) => Self::new(parent_dir.to_path_buf()).resolve_as_node_modules(target),
            None => Err("Not fount".to_string()),
        }
    }

    fn is_build_in_module(&self, target: &str) -> bool {
        self.build_in_node_modules.contains(target)
    }

    fn is_relative_path(&self, target: &str) -> bool {
        target.starts_with(".") || target.starts_with("..")
    }

    fn is_absolute_path(&self, target: &str) -> bool {
        target.starts_with('/')
    }
}

#[test]
fn test_resolver() {
    assert!(Resolver::new(PathBuf::new()).is_build_in_module("fs"));
    assert!(Resolver::new(PathBuf::new()).is_relative_path("./a"));
    assert!(Resolver::new(PathBuf::new()).is_relative_path("../a"));
    assert!(Resolver::new(PathBuf::new()).is_relative_path("../a"));
    assert!(Resolver::new(PathBuf::new()).is_absolute_path("/"));
    assert!(Resolver::new(PathBuf::new()).is_absolute_path("/a/a"));
}
