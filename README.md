Rustmas
=======

Utilities for putting smart Christmas lights on your Christmas tree,
written in Rust, inspired by [Matt Parker's xmastree](https://github.com/standupmaths/xmastree2020).

The interface for controlling lights is provided by [pico-w-neopixel-server](https://github.com/krzmaz/pico-w-neopixel-server/),
which is meant to be installed on a RaspberryPi Pico W. You can use local visualiser
for testing purposes.

Local development setup
-----------------------

### Git hooks

This repository has git hooks prepared that check simple conditions that might
otherwise trip up the CI setup. We recommend that you use them. In order to set
them up, run the following command inside the repository:

```
git config core.hooksPath .githooks
```
