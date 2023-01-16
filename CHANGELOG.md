# Changelog

# 0.0.61

- add string type side effects [#113](https://github.com/modern-js-dev/nodejs_resolver/pull/113)

# 0.0.59 & 0.0.60

- `Hash` for `AliasMap` and `EnforceExtension`.

# 0.0.57 & 0.0.58

- fix alias logic. Previously it only ensured that the two had the same prefix, now it is somewhat ensured that he is a directory.

# 0.0.56

- support log.

# 0.0.55

- support `ParsePlugin`;
- pref on `ExportsPlugin` and `ConditionalMapping`.

# 0.0.54

- fix a bug when resolve self.

# 0.0.53

- optimize `ExportsFiled` map and error message.

# 0.0.52

- refactor `resolve_as_modules`.

# 0.0.51

- fix bug under pnpm structure.
- remove dbg info.

# 0.0.50

yanked

## 0.0.49

- `resolve_modules` will invoke `_resolve` recursively;
- execute `MainFiledPlugin` before `AliasFiledPlugin` in `resolve_modules`;
- do not judge the kind after `ImportsFieldPlugin` in `resolve_modules`;
- some test cases about `browser` pointed to itself.

## 0.0.48

- return `None` if the sideEffects in package.json had invalid value.

## 0.0.47

- support `string` and `false` in package.json/browser field.

## 0.0.46

- fix alias failed in `node_modules`.
- fix exports map bug.

## 0.0.45

- fix when entry path is pointed package.json file.
- add cache for tsconfig.

## 0.0.44

- support `fileDependencies` and `missingDependencies`.

## 0.0.43

- optimize `options.enforce_extension`

## 0.0.42

- add `state::Failed` for `forEachBail`.
- support `Error:Overflow`

## 0.0.41

- change the type of `options.description_file` to `String`.
- use `IndexMap` in `pkgInfo.alias_fields`.
- rewrite cache.
- cache `Entry`, and use `Entry.Stats` to reduce IO operation.

## 0.0.40

- remove unnecessary `Resolver::adjust` in `normalized`.
- remove `FileSystem`.
- change cache policy for modified files.

Now: 


if 

then reread
```
              Timeline
-------------------------------------->>>>
     file_a                      file_a
  [last_read_time]        if
                            (modify_time > last_read_time)
                            && (now_time - modify_time > duration)
                          then reread.
```

## 0.0.39

- remove `Resolver::Default()`;
- remove `unreachable!()` in `resolve`.

## 0.0.38

- rename `CacheFile` to `FileSystem`.
- hidden `CacheFile` in `ResolverOptions`.

This PR has changed the caching policy for modified files.

Before:

```
          Timeline
-------------------------------------->>>>
     file_a                      file_a
  [stored_modified_time]        if duration > CUSTOM_DURATION then reread
                                else then use cache.
  |----- duration = last_modified_time - stored_modified_time -----|
```


Now:

```
Timeline
-------------------------------------->>>>
  file_a                      file_a
  [last_modified_time]        if duration < CUSTOM_DURATION then reread
                              else then use cache.
  |----- duration = now_time - last_modified_time -----|
```

## 0.0.37

- expose `CacheFile`.

## 0.0.36

- remove build in module support.
- error code

## 0.0.35

- To prevent crashes under multiple threads, the `panic` in the debug_assertions has been commented out.

## 0.0.34

- Prevent infinity loop when `browser` filed map to itself.
- The logic of processing `browser` field had been changed. Before, the final result is obtained by recursive function, but now the mapping will be performed once and the result will `_resolve` again.

## 0.0.33

- Do not support auto prefix '.' any more in `extensions`.

## 0.0.32

- Cached `exportField` and `importsField`.

## 0.0.31

- Use `CacheFile` to read `package.json`, therefore, remove `unsafe_cache` and use `cache` instead.

## 0.0.30

- Do not support `alias_fields` any more, instead of `options.browser_field`.

## 0.0.29

- Fix a bug of resolve `alias_filed`:

  Before:

  ```package.json
  {
    "browser": {
      "./toString": "xxxx"
    }
  }
  ```

  And the file structure is:

  ```
  | xxxx.js
  ```

  Then `resolve('toString')` will return `xxxx.js`, and this bug had fixed, it will throw Error now.

## 0.0.28

- Support `sideEffects` in package.json, and export `load_sideeffects`.

## 0.0.27

- Use `serde_json::from_str` instead of `serde_json::from_reader`.

## 0.0.26

- Fix imports field redirect scope range.
- Support external unsafe cache.

## 0.0.25

- Do not resolve as dir when encounter an in-exists node_modules directory.
- Fix a infinity loop in `AliasPlugin`.

## 0.0.24

- Fix a bug under pnpm which will resolve incorrect package.json and return unexpected result.

## 0.0.23

- Use `jsonc_parse` to parse `tsconfig.json`.

## 0.0.22

- Fix a bug under pnpm.

## 0.0.21

- Optimize `pkg_info` cache.

## 0.0.20

- Fix `pkg_info` cache missing.
- Introduce `tracing`.

## 0.0.19

- Use `Resolver::_resolve` for tsconfig/extends.

## 0.0.18

- Fix error resolve when request has scope path with exportsField.

## 0.0.17

- Code optimization.
- Remove node build_in detection.
- No longer support `modules` filed in options.
- No longer support node buildIn module, such as `resolve(xxx, 'fs')` will throw error when there is no `'fs'` polyfill.
- Changed `Option<String>` to `AliasMap` in `Options.alias` and `PkgInfo.aliasField`.
- Support tsconfig path mapping.

## 0.0.16

- Fix a bug caused by `Path::with_extension`.

  `Path::with_extension` will replace the last string by dot sign, for example, `'a.b'.with_extension('c')` will return `'a.c'`, but we expected `'a.b.c'`.

## 0.0.15

- `forEachBail` for alias.
- Fallback when `base_dir.join(target)` is not a valid path.

## 0.0.14

- Support `enforce_extension` option.

## 0.0.13

- Use `Arc` in cache.

## 0.0.12

- Expose `is_build_in_module`.

## 0.0.11

- Change the property type of `Request` from `String` to `SmolStr`.
- Optimized the `Err` report.

## 0.0.10

- Optimized constants in code.

## 0.0.9

- Add `enable_unsafe_cache` in `ResolverOptions`, because user sometimes change the `DescriptionFile`, which can lead to some potential problems in `self.cache`.

## 0.0.8

- Support `prefer_relative` feature.
- Remove `with_xxx` methods, instead of manual assignment.

## 0.0.7

- Public `Options`, and change it `description_file` type from `String` to `Option<String>`.

## 0.0.5 && 0.0.6

yanked

## 0.0.4

- Support `Debug` trait. According to [Debuggability](https://rust-lang.github.io/api-guidelines/debuggability.html#debuggability), all public API types should be implements `Debug`.

## 0.0.3

- (fixture): use `dashmap` to implement cache.
- (fixture): change `resolver.with_base_dir(xxxx).resolve(target)` to `resolver.resolve(xxxx, target)`.
- (chore): add `Windows` and `MacOS` ci environment.
- (refactor): Add coverage test.

## 0.0.2

- Support [`exports`](https://nodejs.org/api/packages.html#exports) and [`imports`](https://nodejs.org/api/packages.html#imports) in package.json.

## 0.0.1

init
