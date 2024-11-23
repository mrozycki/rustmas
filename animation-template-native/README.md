Animation template (Native)
===========================

This is an animation template for a native animation plugin. It will compile
to native machine code, which will have to be recompiled for any new target
architecture you want to run it on. While this is an older, and probably more
stable, solution, for new animations we recommend using the Wasm plugin template.

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

