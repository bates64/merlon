# Merlon

Merlon is a general-purpose modding tool for Paper Mario (N64).

Features:

- [x] Package mods (`merlon pack`)
- [x] Apply mods (`merlon apply`)
- [x] View mod source code
- [x] Merge mods together to create modpacks
- [ ] Shortcut to apply mods and compile
- [ ] Shortcut to combine mods together to create modpacks

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

Merlon is a command-line tool. Use `merlon help` to see a list of commands.

## Mod file format

Merlon mods are packaged as `.merlon` files. These files are encrypted using the original game ROM, and cannot be used without the original game ROM. Unencrypted, it is a BZ2-compressed tarball of git patch files that can be applied to a copy of the decomp source code. Additionally, `.merlon` files contain only source code *changes*, so they are much smaller than the original game ROM. This means that you can use git to view the history of a mod, and to merge mods together. It also means that all mods distributed as `.merlon` files are source-available. This is in contrast to other patch formats, such as Star Rod's `.mod`, which distribute mods as binary patches that cannot be viewed or merged.

## Legal

This application is licensed under the Mozilla Public License 2.0. See the [LICENSE](LICENSE) file for details.

Mods created with this application are not covered by this license. Mods packaged into a file with this application are encrypted using the original game ROM, and cannot be used without the original game ROM. No guarantees are made about the legality of using this application to create mods.

The authors are not affiliated with Nintendo Co., Ltd. in any way.

The PAPER MARIO trademark owned by Nintendo Co., Ltd. is used in this modding tool under the fair use doctrine, solely for the purpose of enabling users to modify the game in a transformative manner.
