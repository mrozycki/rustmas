Rustmas WebAPI
==============

Rustmas WebAPI is the backend of the webapp that controls the Rustmas lights. The frontend is
provided by [Rustmas WebUI](../webui/README.md).

Local development
-----------------

### Database

Rustmas WebAPI uses an SQLite database to store animation parameter values. Migrations for that
database are provided in the `migrations` directory. You can run them using
the [migrant CLI](https://crates.io/crates/migrant) from the `webapi` directory:

```
cargo install migrant --features sqlite
migrant setup
migrant apply
```

This will produce a `db.sqlite` file with appropriate tables set up.

### Environment variables

In order to run WebAPI locally, you need to set up some environment variables:
* `RUSTMAS_POINTS_PATH` is the path to CSV file with light positions
* `RUSTMAS_LIGHTS_URL` is the URL of the pico-w-neopixel-server endpoint
    or `RUSTMAS_USE_TTY=1` to use lights connected via USB
* `RUSTMAS_DB_PATH` is the path to the SQLite database (for parameter storage)
* `RUSTMAS_PLUGIN_DIR` is the path to the [animation plugins directory](../plugins/README.md)

You can set these up with a `.env` file. An example `.env` file is provided in [.env.example](../.env.example).

### Running WebAPI

Once everything is set up, you can start the WebAPI by simply running:

```
cargo run --release -p rustmas-webapi
```

If you want to use the visualizer, you can start it once the WebAPI is running.

```
cargo run --release -p rustmas-visualizer
```

### Running WebUI

Once the WebAPI is running, you can start WebUI using [trunk](https://trunkrs.dev/).
This will require installing trunk first:

```
cargo install trunk
```

And adding the wasm target:

```
rustup target add wasm32-unknown-unknown
```

Then run the following from the `webui` directory:

```
trunk serve --features local
```

The `local` feature will connect WebUI to a locally running WebAPI.

You can also include a visualizer embedded in the UI by using the `visualizer` feature:

```
trunk serve --features local,visualizer
```

Deployment
----------

If you want to use Rustmas to control your Christmas lights, we recommend that you follow
the [deployment](DEPLOYMENT.md) instructions instead.
