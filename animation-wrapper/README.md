Rustmas Animation Wrapper
=========================

This crate contains utilities for wrapping animations into Compiled Rustmas 
Animation Bundles (yes, we worked very hard on this acronym, glad you noticed).

Additionally, an executable `crabwrap` is provided to help create animation
plugin files from animation code. In order to create an animation plugin file,
run `crabwrap` from the root directory of your animation. Before that,
you might need to install the `wasm32-wasip2` target for your Rust toolchain:

```
rustup target add wasm32-wasip2
```