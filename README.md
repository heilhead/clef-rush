# Clef Rush

Piano learning app that helps train music sheet reading skills and keyboard layout.

Available online at https://heilhead.github.io/clef-rush/.

## Hardware Requirements

While the app doesn't require a hardware keyboard, it's best to train with one connected via MIDI interface. Alternatively, the on-screen virtual keyboard can be used (NOTE: only mouse clicking is currently supported, not compatible with touch screen).

## Compatibility

Some browsers are currently buggy when working with MIDI devices, and if you encounter problems with connecting your keyboard, you may need to restart your browser, OS or switch to a different browser. Google Chrome seems to be the most compatible and least buggy.

## How To Play

- Navigate to https://heilhead.github.io/clef-rush/ on the device you want to use. Google Chrome works best, but other browsers may also work.
- Select the device you'll use. This can be either a connected MIDI device, or a virtual on-screen keyboard. If the browser can't detect your MIDI device, try restarting it or switching to a different one.
- Configure key ranges/octaves you want to practice and press 'Play'.
- Read the generated sheet and press corresponding keys on your keyboard to progress.

Note: Click 'Toggle Fullscreen' button to go fullscreen and prevent screen saver/lock.

## Development

Make sure you have `rust` installed. Follow instructions at https://www.rust-lang.org/tools/install.

Once `rust` has been installed, you'll need to add the WASM build target:

```sh
rustup target add wasm32-unknown-unknown
```

Next, install `wasm-pack` and `trunk`:

```sh
cargo install wasm-pack trunk
```

Run the project using the following command:

```sh
trunk serve --release
```

Or build it:

```sh
trunk build --release
```

Build artifacts are located in `dist/` directory.

# License

[Apache 2.0](LICENSE)
