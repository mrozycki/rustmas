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

### Configuration file

In order to run WebAPI locally, you need to create a Rustmas.toml file.
You can find an example with options explained in the [Rustmas.example.toml](../Rustmas.example.toml) 
file. Simply make a copy of it and adjust it however you need.

### Running WebAPI

Once everything is set up, you can start the WebAPI by simply running:

```
cargo run --release -p rustmas-webapi
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

> [!IMPORTANT]
> For this to work, you need `trunk` in version 0.17.0 or newer. Earlier versions
> ignore the `--features` flag.

The `local` feature will connect WebUI to a locally running WebAPI.

You can also include a visualizer embedded in the UI by using the `visualizer` feature:

```
trunk serve --features local,visualizer
```

Deployment
----------

If you want to use Rustmas to control your Christmas lights, we recommend that you follow
the [deployment](DEPLOYMENT.md) instructions instead.
