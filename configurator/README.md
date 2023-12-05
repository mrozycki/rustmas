Rustmas Configurator
====================

Rustmas Configurator allows you to capture light positions on your Christmas tree. This is
a necessary step if you want to run Rustmas on actual physical lights.

This process takes a while and requires some technical knowledge, but it only needs to be done once,
when the lights are first set up.

Configuration process
---------------------

Before you start the configuration process, you will need to know the IP address of
the Raspberry Pi Pico W that is controlling your lights. You will also need a camera connected to
the computer you're running the configurator from. You can either use a webcam connected to
the computer (or built in), or an IP Camera application on your phone.

### Using a built-in/USB webcam

Rustmas Configurator will use the built-in/USB-connected webcam by default, so if this is your
preferred method, simply skip the `-c` option. If you have multiple cameras, and the one chosen
by default is not the one you want, you can try using the `-c` option with a different camera
index, e.g. `-c 1`. The default is 0.

At this point, there is unfortunately no way to list available cameras, as Rustmas uses OpenCV
for image capture, which does not support listing devices.

### Using a remote IP camera

You will need to provide the full URL to an endpoint that produces a live feed video. If the camera
is password-protected, you will need to provide the username and password in the URL as well.
This might look something like this:

```
http://admin:admin@<address>:<port>/video
```

You can test the URL by opening it in your web browser. If you can see live video from your camera,
the URL is most likely correct.

### Capturing light positions

You will need to capture the light positions from 4 directions, going counterclockwise.
The configurator will refer to them as "front", "right", "back" and "left". Which side of your
lights set up is the "front" is up to you, but for each following direction you will need to move
90 degrees counterclockwise around your tree (or rotate your tree by 90 degrees clockwise).

Before you start the process, make sure that you can either set up your camera from each of
the four sides, or you can rotate your tree so that you will be able to capture each side clearly.

Once you are ready, run:

```
cargo run --bin rustmas-configurator -- capture -n <count> -l <pico_url> -c <camera_url> -s
```

where `<count>` is the number of lights, `<pico_url>` is the URL of the pico-w-neopixel-webserver
endpoint (remember the `/pixels` at the end!) and `<camera_url>` is the URL of the IP camera
as described above (or skip if using local camera).

Before each direction is captured, the configurator will turn on all the lights to make it easier
for you to position your camera so that all lights are in shot.  Once you are ready, press Enter
and wait for the lights to be captured.

For best results make sure there are no other bright light sources visible (including reflections)
and avoid any movement in the background of the shot during capture.

### Resuming a stopped process/redoing capture of one or more sides

By default, the configurator will not save any intermediate results, but we recommend enabling it by
adding the `-s` option. This will cause the configurator to store intermediate results, which can
then be used in case the process fails for any reason, or any of the sides need to be re-done.

The intermediate results will be stored in the `captures` directory. Each side will be stored in
a separate directory, named with the date and time of the start of the capture. The directory will
contain images for single lights detected, as well as a reference image with all the lights marked,
and a CSV file with measurements made from that side.

If you would like to resume a failed capture process or redo one or mode sides, you can skip
capturing the sides that have been successful by providing paths to the CSV files for that side,
e.g. to skip front and left:

```
cargo run --bin rustmas-configurator -- capture -n <count> -l <pico_url> -c <camera> -s \
    --front captures/2022-12-05T19:23:07/front.csv --left captures/2022-12-05T19:29:12/left.csv
```

### Centering the lights

Light positions are normalized to fall between -1 and 1 on each axis, which should put the center
of the coordinate system in the center of all the lights. However, since the lights will likely
not be located perfectly symmetrically on the tree, their center does not necessarily have to align
with the center of the tree.

In order to align the center of the lights with the center of the tree, you can use
the `Testing: Manual Sweep` animation to find the alignment, and then run the `configurator center`
command on your lights CSV file:

```
cargo run --bin rustmas-configurator -- center -x 0.08 -z -0.12
```

The coordinates passed as arguments are where the center of the tree is currently located in the
lights coordinate system. This means you can just provide the values found using Manual Sweep.
You can omit an axis, if the alignment in that direction already matches.
