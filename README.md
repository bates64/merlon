# Merlon

[![](https://img.shields.io/github/actions/workflow/status/nanaian/merlon/test.yml?branch=main)](https://github.com/nanaian/merlon/actions)
[![](https://img.shields.io/discord/279322074412089344?color=%237289DA&logo=discord&logoColor=ffffff)](https://discord.gg/paper-mario-modding-279322074412089344)

Merlon is a mod manager for the Paper Mario (N64) decompilation. It creates patches that apply to source code rather than binary files.

Features:

- [x] Create mods (`merlon new`)
- [x] Package mods for distribution (`merlon pack`)
- [x] Apply mod packages (`merlon apply`)
- [x] Compile to a modded ROM (`merlon build`)
- [x] Run the modded ROM (`merlon run`)
- [x] Merge mods together to create modpacks (`merlon apply` multiple times)
- [ ] Shortcut to apply mods and compile
- [ ] Shortcut to combine mods together to create modpacks

## Quickstart

1.  [Install Rust](https://rustup.rs/)
2. `cargo install merlon`
3. `merlon new my-mod`
4. `cd my-mod`
5. Make changes to `my-mod/papermario`. You can `git commit` within as usual.
6. `merlon run` to compile and run the game with your mod applied
7. `merlon pack my-mod.merlon` to package your mod into a distributable file
8. `merlon apply other-mod.merlon` to apply another mod to your mod

## Supported platforms

Merlon supports all platforms that the Paper Mario decompilation supports.

- Linux
    - Debian/Ubuntu
    - Arch Linux
    - openSUSE
    - Alpine Linux
    - Nix (nix-shell)
- macOS
- Windows (Windows Subsystem for Linux 2)

See the [decomp installation instructions](https://github.com/pmret/papermario/blob/main/INSTALL.md) for more information.

Additionally, Merlon has a number of runtime dependencies that should be available on your PATH:

- `git`
- `ninja`
- `python3`
- `tar`
- `bzip2`
- `openssl`

## Installation

See [releases](https://github.com/nanaian/merlon/releases) for pre-built binaries.

### From source

Merlon is written in Rust. To build from source, you will need to install the Rust toolchain. See [rustup.rs](https://rustup.rs/) for instructions.

Once you have the Rust toolchain installed, you can install Merlon from crates.io with:

```bash
cargo install merlon
```

If you have a clone of this repository, you can install Merlon with:

```bash
cargo install --path .
```

## Usage

Merlon is a command-line tool. Use `merlon help` for more information.

## Mod file format

Merlon mods are packaged as `.merlon` files. These files are encrypted using the original game ROM, and cannot be used without the original game ROM. Unencrypted, it is a BZ2-compressed tarball of git patch files that can be applied to a copy of the decomp source code. Additionally, `.merlon` files contain only source code *changes*, so they are much smaller than the original game ROM. This means that you can use git to view the history of a mod, and to merge mods together. It also means that all mods distributed as `.merlon` files are source-available. This is in contrast to other patch formats, such as Star Rod's `.mod`, which distribute mods as binary patches that cannot be viewed or merged.

## Legal

This application is licensed under the Mozilla Public License 2.0. See the [LICENSE](LICENSE) file for details.

Mods created with this application are not covered by this license. Mods packaged into a file with this application are encrypted using the original game ROM, and cannot be used without the original game ROM. No guarantees are made about the legality of using this application to create mods.

The authors are not affiliated with Nintendo Co., Ltd. in any way.

The PAPER MARIO trademark owned by Nintendo Co., Ltd. is used in this modding tool under the fair use doctrine, solely for the purpose of enabling users to modify the game in a transformative manner.
