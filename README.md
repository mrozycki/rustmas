Rustmas
=======

*Santa Crab is coming to town!*

Utilities for putting smart Christmas lights on your Christmas tree, written in Rust, inspired by
[Matt Parker's xmastree](https://github.com/standupmaths/xmastree2020).

The interface for controlling lights is provided by [pico-w-neopixel-server](https://github.com/krzmaz/pico-w-neopixel-server/),
which is meant to be installed on a RaspberryPi Pico W. You can use local visualiser
for testing purposes.

## Demo (YouTube)
[![Demo](https://img.youtube.com/vi/UKONMvyDPdw/sddefault.jpg)](https://www.youtube.com/watch?v=UKONMvyDPdw)

Local development setup
-----------------------

### You will need

* Rust toolkit (see [rustup](http://rustup.rs))
* dependencies installed (see [Workflow file](.github/workflows/rust.yml) for what to install on Ubuntu)
* (optionally) programmable lights set up with [pico-w-neopixel-server](http://github.com/krzmaz/pico-w-neopixel-server),
  or you can use the OpenGL visualiser instead

### Setting up your lights

If you are using physical lights, you need to connect them to a Raspberry Pi Pico W running
[pico-w-neopixel-server](http://github.com/krzmaz/pico-w-neopixel-server), and configure their
positions using [our configurator](configurator/README.md). This will produce a CSV file with light
positions. Alternatively you can use the visualiser for testing with the [example CSV file](lights.csv.example).

### Running code locally

The easiest option to test an animation is to run it with [animator-cli](animator/README.md).
This might be the better option for quick testing, although it does not support switching animations
or changing its parameters.

The alternative is to run the whole [web application](webapi/README.md) locally. This is slightly
more involved, but will allow you to quickly switch between animations and test your animation's
parameters.

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
