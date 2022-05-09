# Changelog

## 0.0.5

- expose `ResolverOptions`.
 
## 0.0.4

- support `Debug` trait. According to [Debuggability](https://rust-lang.github.io/api-guidelines/debuggability.html#debuggability), all public API types should be implements `Debug`.

## 0.0.3

- (fixture): use `dashmap` to implement cache.
- (fixture): change `resolver.with_base_dir(xxxx).resolve(target)` to `resolver.resolve(xxxx, target)`.
- (chore): add `Windows` and `MacOS` ci environment.
- (refactor): Add coverage test.

## 0.0.2

- support [`exports`](https://nodejs.org/api/packages.html#exports) and [`imports`](https://nodejs.org/api/packages.html#imports) in package.json.

## 0.0.1

init