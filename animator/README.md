Rustmas Animator
================

Rustmas Animator is the library that controls animations on Rustmas lights.

Animator CLI
------------

In order to run the Animator CLI, you will need a CSV file with light positions.
The default file name is `lights.csv`, but you can provide a different one
using the `-p` option.

In order to run an animation `rainbow_waterfall` in a visualiser:

```
$ cargo run --bin animator-cli -- -a rainbow_waterfall
```

With your own light positions file:

```
$ cargo run --bin animator-cli -- -a rainbow_waterfall -p my_lights.csv
```

Or on physical lights (remember to provide correct IP address!):

```
$ cargo run --bin animator-cli -- -a rainbow_waterfall -l http://127.0.0.1/pixels
```

There is currently no way to provide parameters for the animation through
Animator CLI. If you would like to test your animation's parameters, use
the rustmas [web application](../webapi/README.md).
