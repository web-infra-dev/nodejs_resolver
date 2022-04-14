# node_resolver


## How to use?

```rust
/// Dir structure
/// |-- node_modules
/// |---- foo
/// |------ index.js
/// | src
/// |-- foo.ts
/// |-- foo.js
/// | tests
/// 
let cwd = std::env::current_dir().unwrap();
let mut resolver = Resolver:default().with_base_dir(cwd.join("./src"));

// <cwd>/node_modules/foo/index.js
resolver.resolve("foo")
// <cwd>/src/foo.js
resolver.resolve("./foo")
```