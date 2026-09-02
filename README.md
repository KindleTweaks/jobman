# Jobman

> [!NOTE]
> A Simple Kindle Jailbreak Manager, that Performs Certain Jobs! :) WIP

<a href='https://ko-fi.com/W7W31J9IS0' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi5.png?v=6' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>

*Like [my](https://penguins184.xyz/) work? Consider donating or just starring my repo! :)*

## Building

Great thanks to [slint-kindle-backend](https://github.com/sverrejb/slint-kindle-backend).

## PC

To quickly view UI changes, use: 

```sh
cargo run
```

Alternatively: 

```sh
cargo install slint-viewer # One-Time

# Then:
slint-viewer ui/app.slint
```

## Kindle

Cross-compilation setup:

```sh
rustup target add armv7-unknown-linux-musleabihf
cargo install cargo-zigbuild
sudo pacman -S zig # Or Platform Equivalent
```

To build:

```sh
RUST_FONTCONFIG_DLOPEN=1 cargo zigbuild --release --target armv7-unknown-linux-musleabihf
```

## Notes

Inter, Libre Baskerville uses **SIL Open Font License (OFL) Version 1.1**. This project is licensed under **GNU GPLv3**.
