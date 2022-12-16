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
$ cargo install migrant --features sqlite
$ migrant setup
$ migrant apply
```

This will produce a `db.sqlite` file with appropriate tables set up.

### Environment variables

In order to run WebAPI locally, you need to set up some environment variables:
* `RUSTMAS_POINTS_PATH` is the path to CSV file with light positions
* `RUSTMAS_LIGHTS_URL` is the URL of the pico-w-neopixel-server endpoint
* `RUSTMAS_DB_PATH` is the path to the SQLite database (for parameter storage)

You can set these up with a `.env` file. An example `.env` file is provided in [.env.example](../.env.example).

### Running WebAPI

Once everything is set up, you can start the WebAPI by simply running:

```
$ cargo run --bin rustmas-webapi
```

The WebAPI needs to be restarted every time you modify the code (including animations).

### Running WebUI

Once the WebAPI is running, you can start WebUI using [trunk](https://trunkrs.dev/).
Run the following from the `webui` directory:

```
$ trunk serve
```

Deployment
----------

If you want to use Rustmas to control your Christmas lights, we recommend that you follow
the [deployment](DEPLOYMENT.md) instructions instead.
