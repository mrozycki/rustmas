## Running
native:
```
cargo run [--release] --bin rustmas-bevy-visualizer
```
or WASM:
```
# once:
cargo install wasm-server-runner

# then:
cargo run --profile wasm-release --target wasm32-unknown-unknown --bin rustmas-bevy-visualizer
```
## Controls
- RMB: Orbit camera
- MMB: Pan camera
- Scroll: Zoom camera

## Next steps

Investigate if this gives better rendering on WASM:
https://github.com/mrk-its/bevy_webgl2

Mouse picking of entities (should be useful to set colors via visualizer):
https://github.com/aevyrie/bevy_mod_picking
