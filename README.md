Rustmas
=======

*Santa Crab is coming to town!*

Utilities for putting smart Christmas lights on your Christmas tree, written in Rust, inspired by
[Matt Parker's xmastree](https://github.com/standupmaths/xmastree2020).

The interface for controlling lights is provided by [pico-w-neopixel-server](https://github.com/krzmaz/pico-w-neopixel-server/),
which is meant to be installed on a RaspberryPi Pico W. You can use local visualizer
for testing purposes.

## Demo (YouTube)
[![Demo](https://img.youtube.com/vi/UKONMvyDPdw/sddefault.jpg)](https://www.youtube.com/watch?v=UKONMvyDPdw)

Local development setup
-----------------------

### You will need

* Rust toolkit (see [rustup](http://rustup.rs))
* dependencies installed (see [Workflow file](.github/workflows/rust.yml) for what to install on Ubuntu)
* (optionally) programmable lights set up with [pico-w-neopixel-server](http://github.com/krzmaz/pico-w-neopixel-server),
  or you can use our visualizer instead

### Setting up your lights

If you are using physical lights, you need to connect them to a Raspberry Pi Pico W running
[pico-w-neopixel-server](http://github.com/krzmaz/pico-w-neopixel-server) or
[pico-usb-neopixel-driver](https://github.com/krzmaz/pico-usb-neopixel-driver), and configure their
positions using [our configurator](configurator/README.md). This will produce a CSV file with light
positions. Alternatively you can use the visualizer for testing with the [example CSV file](lights.csv.example).

### Running code locally

The easiest way to test your animations is to run our [web application](webapi/README.md) locally.
You can use either physical lights or our visualizer.

### Git hooks

This repository has git hooks prepared that check simple conditions that might otherwise trip up
the CI setup. We recommend that you use them. In order to set them up, run the following command
inside the repository:

```
git config core.hooksPath .githooks
```

Deployment
----------

If you would like to use Rustmas to control your Christmas lights, you will first need to
[configure your lights](configurator/README.md) and then [deploy our web application](webapi/DEPLOYMENT.md)
to a local server.

Attribution
-----------

* [Christmas lights icons created by BomSymbols - Flaticon](https://www.flaticon.com/free-icons/christmas-lights)
* [Settings icons created by Freepik - Flaticon](https://www.flaticon.com/free-icons/settings)