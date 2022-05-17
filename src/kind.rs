use crate::Resolver;

pub enum PathKind {
    Empty,
    Relative,
    AbsoluteWin,
    AbsolutePosix,
    Internal,
    BuildInModule,
    Normal,
}
use daachorse::DoubleArrayAhoCorasick;
use once_cell::sync::Lazy;

static PATTERNS: [&str; 66] = [
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
    "assert/,strict",
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
];
static PMA: Lazy<DoubleArrayAhoCorasick> =
    Lazy::new(|| DoubleArrayAhoCorasick::new(PATTERNS).unwrap());
impl Resolver {
    pub fn get_target_kind(target: &str) -> PathKind {
        if target.is_empty() {
            PathKind::Empty
        } else if Self::is_build_in_module(target) {
            PathKind::BuildInModule
        } else if target.starts_with('#') {
            PathKind::Internal
        } else if target.starts_with('/') {
            PathKind::AbsolutePosix
        } else if target == "."
            || target.starts_with("./")
            || target.starts_with("../")
            || (target.len() == 2 && target.starts_with(".."))
        {
            PathKind::Relative
        } else if (target.len() == 2
            && (('a'..='z').any(|c| target.starts_with(&format!("{c}:")))
                || ('A'..='Z').any(|c| target.starts_with(&format!("{c}:")))))
            || ('a'..='z').any(|c| {
                target.starts_with(&format!("{c}:\\")) || target.starts_with(&format!("{c}:/"))
            })
            || ('A'..='Z').any(|c| {
                target.starts_with(&format!("{c}:\\")) || target.starts_with(&format!("{c}:/"))
            })
        {
            PathKind::AbsoluteWin
        } else {
            PathKind::Normal
        }
    }

    fn is_build_in_module(target: &str) -> bool {
        // for mat in PMA.find_iter(target) {
        //     if mat.start() == 0 && mat.end() == target.len() {
        //         return true;
        //     }
        // }
        PATTERNS.contains(&target)
    }
}

#[test]
fn test_resolver() {
    assert!(Resolver::is_build_in_module("fs"));
    assert!(!Resolver::is_build_in_module("a"));
}
