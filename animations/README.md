Rustmas Animations
==================

This directory provides some starting animations for you to display on
your tree. 
Animations for Rustmas are provided in a form of plugins. They can be added, removed
and modified in place even while Web API is running. We recommend that you
build the starting plugins using the provided script:

```
cargo install --path animation-wrapper
cd animations
./build_all.sh
```

This will build, package and gather all the starting plugins under `target/animations/`
in the root of the Rustmas repository. You can then point your Web API to
that directory by setting `plugin_path` in `Rustmas.toml` to `target/animations/`.


Docker
------

In case you're running Rustmas under docker (see [`docker/README.md`](../docker/README.md)),
the animations will already be built for you, and there's nothing more
you need to do.


Your own animations
-------------------

The recommended way to create your own animations is to make a copy of the `animation-template-wasm`
crate in the root of this project, make appropriate changes to the code and the manifest file,
and then package it using the provided `crabwrap` utility, which you can install with:

```
cargo install --path animation-wrapper
```

Running this utility will produce a `.crab` file, which then needs to be copied
to the `plugin_path` directory specified in `Rustmas.toml`.

**Note:** adding a new animation will require you to refresh animation list by going
into *Settings* -> *Animations* in WebUI. Modifying an existing animation will 
require reloading it by either clicking *Reload* in WebUI, or switching 
to a different animation and back.