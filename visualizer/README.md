# Rustmas Visualizer

## Running

Running the visualizer as a standalone native application is no longer supported.
The recommended way to use the visualizer is to [run is as a part of WebUI](../webapi/README.md#running-webui).
The following instructions are not guaranteed to work.

You can run the visualizer as a native binary with:

```
cargo run [--release] --bin rustmas-visualizer
```

Visualizer needs to be started after WebAPI.

## Controls

You can control the view of the visualizer with your mouse. Scroll to zoom,
move your mouse while holding left mouse button to move, move your mouse while
holding right mouse button to rotate the view.
