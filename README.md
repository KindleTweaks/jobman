# Jobman

> [!NOTE]
> A Simple Kindle Jailbreak Manager, that Performs Certain Jobs! :) WIP

<a href='https://ko-fi.com/W7W31J9IS0' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi5.png?v=6' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>

*Like [my](https://penguins184.xyz/) work? Consider donating or just starring my repo! :)*

<img width="350" height="500" alt="recovery" src="https://github.com/user-attachments/assets/ef95ba78-2495-47e2-be9d-b612a12d8e67" />

P.S The bee is called Jay Bee! - JB - get it? (thanks to Mergen for this pun!)

## Installation 

Jobman is hosted on the official KindleModding KPM [repository](https://repo.kindlemodding.org/).

To install, run the following in the search bar (one at a time):

```
;kpm update
;kpm install jobman
```

## Features

- Update blocking/unblocking
- Battery health estimates
- Environment updates
- USB SSH 
- Wi-Fi SSH
- Recovery mode
- Log generation
- Our mascot, Jay Bee! 

## Tested devices (default, & recovery mode)

- 5.19.2 KT5
- 5.20.1, 5.19.2 KT6
- 5.16.21, 5.18.1.1.1 PW4
- 5.20.1, 5.19.2 PW6
- 5.17.1.0.3 PW5
- 5.20.1 KS1
- 5.20.1 CS

## Building

Great thanks to [slint-kindle-backend](https://github.com/sverrejb/slint-kindle-backend).

### Prerequisites

- [koxtoolchain & Kindle SDK](https://kindlemodding.org/kindle-dev/gtk-tutorial/prerequisites.html)
- cargo-zigbuild
    - `rustup target add armv7-unknown-linux-musleabihf`
    - `cargo install cargo-zigbuild`
    - `sudo pacman -S zig # or platform equivalent`

### Alternative: Nix

If you use [Nix](https://nixos.org/download/) (with flakes enabled), `nix develop` gives you a shell with all of the above, including a pinned koxtoolchain, and the build commands below work unchanged:

```sh
git submodule update --init
nix develop -c ./build.sh kindle
```

## PC

Use `slint-viewer`, bundled with the Slint vscode plugin, or:

`./build.sh`

## Kindle

Run `./build.sh kindle`, and copy both folders in `build/` to the device.

## Notes & Credits

Inter, Libre Baskerville uses **SIL Open Font License (OFL) Version 1.1**. This project is licensed under **GNU GPLv3**.

- Penguins184: Creator of Jobman
- AcidWeb: Coloursoft launching support (slint-kindle-backend)
- SverreJB: Original slint-kindle-backend library itself
- Sparklerfish: Tested and helped out with SSH and launching bits and bobs!
- Clint, Ava: Further testing,
- Katadelos: Kindle emulator creation which allowed for the safe testing of recovery mode.
