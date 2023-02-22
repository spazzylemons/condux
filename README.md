# Condux

An anti-gravity racing game.

## Building

### For Linux, MacOS or Windows

Follow the instructions for setting up SDL2 at
[the sdl2 crate page](https://crates.io/crates/sdl2), and then build the
condux-app package.

### For Nintendo 3DS

Install the 3DS development tools from
[devKitPro](https://devkitpro.org/wiki/Getting_Started), install
[cargo-3ds](https://github.com/rust3ds/cargo-3ds), and then build the condux-app
package using cargo-3ds, with the devKitARM binaries on your `$PATH`.

Example command: (builds app in debug mode)

```sh
# in the condux-app directory:
PATH=$PATH:$DEVKITARM/bin cargo +nightly 3ds build
```

### For Web

Enter the condux-web directory, run `npm install`, and then run a command to
build or run the program:

- `npm run build-debug` builds in debug mode
- `npm run serve-debug` runs in debug mode
- `npm run build-release` builds in release mode
- `npm run serve-release` runs in release mode
