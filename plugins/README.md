# Animations plugins directory

Animations for Rustmas are provided in a form of plugins. They can be added, removed
and modified in place even while webapi is running. Plugins should be stored in a single
directory. This directory is an example of a plugin directory, containing configuration
for all the plugins provided by this repository. You can use it as is, once you buiild
all the plugins with:

```
cargo build --release -p animations
```

**Note:** adding a new animation will require you to refresh animation list by clicking
"Refresh" in WebUI. Modifying an existing animation will require reloading it by either
clicking "Reload" in WebUI, or switching to a different animation and back.

Each plugin has its own directory within the plugins dir, with the following structure:

```
my-animation/
    manifest.json
    plugin
```

`manifest.json` contains plugin metadata and looks like this:

```json
{
	"display_name": "My Animation"
}
```

`plugin` is a binary file (or a symbolic link to one), which conforms to the animation
plugin contract. The easiest way to create one is to start with the [animation template](../animation-template),
which will produce the appropriate binary when compiled.