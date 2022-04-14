use crate::Resolver;

pub enum PathKind {
    Empty,
    Normal,
    Relative,
    // TODO: win or unix
    Absolute,
    Internal,
}

impl Resolver {
    pub fn get_path_kind(&self, target: &str) -> PathKind {
        if target.is_empty() {
            PathKind::Empty
        } else if Resolver::is_absolute_path(target) {
            PathKind::Absolute
        } else if Resolver::is_relative_path(target) {
            PathKind::Relative
        } else if Resolver::is_build_in_module(target) {
            PathKind::Internal
        } else {
            PathKind::Normal
        }
    }

    fn is_build_in_module(target: &str) -> bool {
        target == "_http_agent"
            || target == "_http_client"
            || target == "_http_common"
            || target == "_http_incoming"
            || target == "_http_outgoing"
            || target == "_http_server"
            || target == "_stream_duplex"
            || target == "_stream_passthrough"
            || target == "_stream_readable"
            || target == "_stream_transform"
            || target == "_stream_wrap"
            || target == "_stream_writable"
            || target == "_tls_common"
            || target == "_tls_wrap"
            || target == "assert"
            || target == "assert/strict"
            || target == "async_hooks"
            || target == "buffer"
            || target == "child_process"
            || target == "cluster"
            || target == "console"
            || target == "constants"
            || target == "crypto"
            || target == "dgram"
            || target == "diagnostics_channel"
            || target == "dns"
            || target == "dns/promises"
            || target == "domain"
            || target == "events"
            || target == "fs"
            || target == "fs/promises"
            || target == "http"
            || target == "http2"
            || target == "https"
            || target == "inspector"
            || target == "module"
            || target == "net"
            || target == "os"
            || target == "path"
            || target == "path/posix"
            || target == "path/win32"
            || target == "perf_hooks"
            || target == "process"
            || target == "punycode"
            || target == "querystring"
            || target == "readline"
            || target == "repl"
            || target == "stream"
            || target == "stream/consumers"
            || target == "stream/promises"
            || target == "stream/web"
            || target == "string_decoder"
            || target == "sys"
            || target == "timers"
            || target == "timers/promises"
            || target == "tls"
            || target == "trace_events"
            || target == "tty"
            || target == "url"
            || target == "util"
            || target == "util/types"
            || target == "v8"
            || target == "vm"
            || target == "wasi"
            || target == "worker_threads"
            || target == "zlib"
    }

    fn is_relative_path(target: &str) -> bool {
        target.starts_with('.') || target.starts_with("..")
    }

    fn is_absolute_path(target: &str) -> bool {
        target.starts_with('/')
    }

    pub fn may_be_dir(target: &str) -> bool {
        target.ends_with('/')
    }
}

#[test]
fn test_resolver() {
    assert!(Resolver::is_build_in_module("fs"));
    assert!(Resolver::is_relative_path("./a"));
    assert!(Resolver::is_relative_path("../a"));
    assert!(Resolver::is_relative_path("../a"));
    assert!(Resolver::is_absolute_path("/"));
    assert!(Resolver::is_absolute_path("/a/a"));
    assert!(Resolver::may_be_dir("/a/a/"));
}
