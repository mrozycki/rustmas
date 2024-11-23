Animation template (Wasm)
=========================

This is an animation template for a Wasm animation plugin. It will produce
a cross-platform plugin, which can be compiled once and then used on any
deployment of Rustmas WebAPI, regardless of the target machine architecture.

Make a clone of this crate to create your own animation. Everything you
need is provided for you. For a simple animation, you will need to
provide your own implementation of the `new`, `update`, and `render`.

Optionally, you can add support for parameters by adding your own parameters
to the `Parameters` structure. You might add extra logic to `set_parameters`
if you need to do any pre-processing on new parameter values. For some common
paramters, like speed or brightness, you can specify a `Wrapped` type with
appropriate decorators.

You can also handle events by providing your implementation of the `on_event`
method.

In order to build this plugin, you will have to install a `wasm32-wasip2`
Rust target. This requires Rust version 1.82 or newer.

```
rustup target add wasm32-wasip2
```