The `chik_wasm` package has JavaScript bindings for the rust implementation of the `chik` crate in wasm.

Build
-----

Use `wasm-pack` to build the wasm `pkg` file used with npm. Install it with:

```
$ cargo install wasm-pack
```

Then build with

```
$ wasm-pack build --release
```
