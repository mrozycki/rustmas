Rustmas WebAPI
==============

Rustmas WebAPI is the backend of the webapp that controls the Rustmas lights. The frontend is
provided by [Rustmas WebUI](../webui/README.md).

Local development
-----------------

### Installing necessary tools

If you don't already have a Rust toolchain installed, you can install it using rustup:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Then you will need to install `trunk` to serve the frontend code and add 
a WASM target for Rust to compile the frontend. This can be done with:

```
cargo install trunk
rustup target add wasm32-unknown-unknown
```

You may also need to install additional dependencies. For example on ubuntu:

```
sudo apt install libxinerama-dev libxcursor-dev xorg-dev libgl1 libgl1-mesa-dev libudev-dev clang libclang-dev 
```

Optionally, if you want to use animations with audio support:

```
sudo apt install libasound2-dev
```

And if you want to run the configurator:

```
sudo apt install libopencv-dev 
```

### Database

Rustmas WebAPI uses an SQLite database to store animation parameter values. 
It runs appropriate migrations at startup, so for an initial run you only need
to provide an empty SQLite database file. One has been provided in the repository
as [`db.sqlite.example`](../db.sqlite.example) in the root of the project.
You just need to make a copy named `db.sqlite`.

```
cp db.sqlite.example db.sqlite
```

### Configuration file

In order to run WebAPI locally, you need to create a Rustmas.toml file.
You can find an example with options explained in the [Rustmas.example.toml](../Rustmas.example.toml) 
file. Simply make a copy of it and adjust it however you need.

```
cp Rustmas.example.toml Rustmas.toml
```

### Building animations

Animations are provided as separate binaries and have to be built separately.
You can do it with:

```
cargo run --release -p animations
```

The `plugin` directory, where the WebAPI will be looking for plugins, has been
provided for you. It contains manifests for all the provided animations and symbolic
links to the appropriate binaries in the target directory.

If you add more animations while the app is running, you can re-run animation
discovery using the *Refresh list* button at the top of the animations list
in the WebUI. If you rebuild an animation that is already on the list and running,
you can load the new version by clicking the *Reload* button at the bottom
of the parameters list on the right.

### Running WebAPI

Once everything is set up, you can start the WebAPI by running the folowing command
from the project root:

```
cargo run --release -p rustmas-webapi
```

WebAPI uses audio by default, to provide data to audio-enabled animations.
You can turn off that support by running the WebAPI without the additional
features:

```
cargo run --release -p rustmas-webapi --no-default-features
```

### Running WebUI

Once the WebAPI is running, you can start WebUI using [trunk](https://trunkrs.dev/).
In order to do that, run trunk serve from `webui` directory:

```
cd webui
trunk serve --features local,visualizer
```

> [!IMPORTANT]
> For this to work, you need `trunk` version 0.17.0 or newer. Earlier versions
> ignore the `--features` flag.

The `local` feature will cause WebUI to connect to a locally running WebAPI. 

The `visualizer` feature will include an embedded visualizer in the app.
This is particularly useful for animation development, since it allows you to
run the entire system without physical lights. It also is the most convenient
way to produce mouse events for animations that make use of them (like Draw).

However, the visualizer is quite resource intensive, especially in a debug build, 
so for testing on less powerful devices and/or with physical lights available, 
you may choose to omit it:

```
trunk serve --features local
```

Deployment
----------

If you want to use Rustmas to control your Christmas lights, we recommend that you follow
the [deployment](DEPLOYMENT.md) instructions instead.
