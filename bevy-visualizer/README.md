# Rustmas Visualizer

## Running

You can run the visualizer as a native binary with:

```
cargo run [--release] --bin rustmas-bevy-visualizer
```

Alternatively, you can run it in the web browser. First you will have to
install `wasm-server-runner`:

```
cargo install wasm-server-runner
```

and then you can start the visualizer with:

```
cd bevy-visualizer
cargo run [--release] --target wasm32-unknown-unknown --bin rustmas-bevy-visualizer
```

Visualizer needs to be started after WebAPI.

## Controls

You can control the view of the visualizer with your mouse. Scroll to zoom,
move your mouse while holding left mouse button to move, move your mouse while
holding right mouse button to rotate the view.
