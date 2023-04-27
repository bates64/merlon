# Merlon

[![](https://img.shields.io/github/actions/workflow/status/nanaian/merlon/test.yml?branch=main)](https://github.com/nanaian/merlon/actions)
[![](https://img.shields.io/discord/279322074412089344?color=%237289DA&logo=discord&logoColor=ffffff)](https://discord.gg/paper-mario-modding-279322074412089344)

Merlon is a mod package manager for the Paper Mario (N64) decompilation.

Features:

- [x] Create packages (`merlon new`)
- [x] Export package to a file for distribution (`merlon export`)
- [x] Apply distributable files to a base ROM (`merlon apply`)
- [x] Compile current package to a modded ROM (`merlon build`)
- [x] Run modded ROM in an emulator (`merlon run`)
- [x] Package dependency management (`merlon add`)
- [x] Experimental GUI support (`merlon gui` when built with `--features gui`)

## Installation

> **Note:** If you use Windows, you will need to use WSL 2. See the [decomp installation instructions](https://github.com/pmret/papermario/blob/main/INSTALL.md#wsl-2) for more information.

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

## Usage

Merlon is a command-line tool. Use `merlon help` for more information.

A quick tour:

```
$ merlon new "My mod"
$ cd my-mod
$ merlon init
$ touch papermario/src/my-mod.c && git -C papermario add src/my-mod.c && git -C papermario commit -m "add src/my-mod.c"
$ merlon build
$ merlon export
$ merlon apply "My mod 0.1.0.merlon"
```

Example mods can be found at [nanaian/pm-mods](https://github.com/nanaian/pm-mods).

## Mod file format

Merlon mods are packaged as `.merlon` files. These files are encrypted using the original game ROM, and cannot be used without the original game ROM. Unencrypted, it is a BZ2-compressed tarball of git patch files that can be applied to a copy of the decomp source code. Additionally, `.merlon` files contain only source code *changes*, so they are much smaller than the original game ROM. This means that you can use git to view the history of a mod, and to merge mods together. It also means that all mods distributed as `.merlon` files are source-available. This is in contrast to other patch formats, such as Star Rod's `.mod`, which distribute mods as binary patches that cannot be viewed or merged.

## Legal

This application is licensed under the Mozilla Public License 2.0. See the [LICENSE](LICENSE) file for details.

Mods created with this application are not covered by this license. Mods packaged into a file with this application are encrypted using the original game ROM, and cannot be used without the original game ROM. No guarantees are made about the legality of using this application to create mods.

The authors are not affiliated with Nintendo Co., Ltd. in any way.

The PAPER MARIO trademark owned by Nintendo Co., Ltd. is used in this modding tool under the fair use doctrine, solely for the purpose of enabling users to modify the game in a transformative manner.
