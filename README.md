# dynamic-leds
Match cheap LED light strips to the dominant colour of a screen.

This script can be built as a small Windows executable that will update a cheap set of Bluetooth Low Energy controllable LED strips to match the dominant colour on the screen. It can achieve this at ~20 frames a second fairly reliably and includes an automatic restart for when unexpected dropouts occur. There is a [demo recorded here](https://www.youtube.com/watch?v=lnAgft4b0Bs).

## Build
If you check out this source code, ensure you have the Rust tools installed and run `cargo build`. You will probably need to update `peripheral_name` to match your device.

Performance will be better actually running it as a release build `cargo build --release`.

The executable will then be available at `target/[debug|release]/dynamic-leds.exe`.

## Notes
- The BLE on this hardware drops out occasionally, there is a built in retry in this script but it may result in a few dropped frames.
- This script is in theory cross platform with Linux and Mac, but it hasn't been tested under either of these.
- This may work for some other Govee hardware, possibly with minor tweeks but it's only been tested with a single H6181 unit.
