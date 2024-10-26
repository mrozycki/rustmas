Rustmas Webapp Deployment
=========================

Prerequisites
-------------

1. Lights connected through [pico-w-neopixel-server](http://github.com/krzmaz/pico-w-neopixel-server)
2. Light positions captured and stored in a CSV file
3. (recommended) Raspberry Pi to control the lights. You can use any computer to run Rustmas,
   but the instructions below were written with a Raspberry Pi running Raspbian in mind.

Getting and building the code
-----------------------------

* Install cargo through rustup:
  ```
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source "$HOME/.cargo/env"
  ```

* Install tools and dependencies:
  ```
  sudo apt install git libssl-dev
  ```

* Clone the repository:
  ```
  git clone https://github.com/mrozycki/rustmas
  cd rustmas
  ```

Database setup
--------------

Similarly to local development setup, you will need to provide an empty SQLite 
database file. One has been provided in the repository as [`db.sqlite.example`](../db.sqlite.example) 
in the root of the project. You just need to make a copy named `db.sqlite`.

```
cp db.sqlite.example db.sqlite
```

Alternatively, you can just use the `db.sqlite` file from your local development setup.
Make sure to copy it into the project root directory.

Animation plugin setup
----------------------

Before starting the service you will need to build animations and add your animations
to the plugins directory. Instructions can be found in the provided example [plugins directory](../plugins/README.md).
In short, you can build the animations with:

```
cargo build --release -p animations
```

The plugin directory set up for all the built-in animations is provided 
in the repository already.


WebAPI service
--------------

Copy the [`service.example`](deployment/service.example) file to `/etc/systemd/system/rustmas.service`.

```
sudo cp webapi/deployment/service.example /etc/systemd/system/rustmas.service
```

Make sure all the settings in that file are correct. If you're running the application on
a Raspberry PI, you will likely only need to modify the WorkingDirectory and ExecStart paths to
point to the right place (the example file assumes the repository is located at `/home/pi/rustmas`)
and lights URL.

Create a `Rustmas.toml` file in the working directory specified in the service configuration file.
You can copy the [`Rustmas.example.toml`](../Rustmas.example.toml) file and adjust it as needed.

```
cp Rustmas.example.toml Rustmas.toml
```

Before running the service, you will need to build the WebAPI:

```
cargo build --bin rustmas-webapi --release
```

Enable the service with:

```
sudo systemctl start rustmas
sudo systemctl enable rustmas
```

This will start the service automatically every time you start the machine.

You can verify that the service started successfully with journalctl:

```
journalctl -f -u rustmas
```

If there are no errors, and the logs end with a message similar to this:

```
[INFO] Actix runtime found; starting in Actix runtime
```

Everything is working fine.

WebUI
-----

You can prepare the WebUI for deployment using [trunk](http://trunkrs.dev). Installing it on
a Raspberry Pi might not be optimal, and since it produces the same output regardless of where
it's ran, you might prefer to build the WebUI on a more powerful computer.

Install trunk and build the WebUI by running the following from the `webui` directory:

```
cargo install trunk
rustup target add wasm32-unknown-unknown
trunk build --release
```

You can also include a visualizer embedded in the UI by using the `visualizer` feature:

```
trunk build --release --features visualizer
```

> [!IMPORTANT]
> For this to work, you need `trunk` in version 0.17.0 or newer. Earlier versions
> ignore the `--features` flag.

This will make the compiled WASM file significantly larger and will impact loading time,
so it is turned off by default. The visualizer will also only show up on large displays
(tablets, computer screens), and not on a phone.

Reverse proxy
-------------

We recommend using nginx to proxy the traffic of both WebAPI and WebUI through a single port.
You can install it with:

```
sudo apt install nginx
```

In order to configure the proxy, copy the [`nginx.example`](deployment/nginx.example) file to
`/etc/nginx/sites-available/rustmaspi.local`. You will need to modify the line starting with
`ServerName` to include your server's local IP address and (optionally) its hostname.

After that, you will need to enable the configuration with:

```
sudo ln -s /etc/nginx/sites-available/rustmaspi.local /etc/nginx/sites-enabled/
```

You will also need to copy the compiled WebUI files to `/var/www/rustmas`:

```
sudo mkdir /var/www/rustmas
sudo chown www-data /var/www/rustmas
sudo cp webui/dist/* /var/www/rustmas
```

After that is done, you need to restart nginx:

```
sudo systemctl restart nginx
```

All done! You can now navigate to your machine's address (as specified in the nginx configuration)
and see if it's working.
