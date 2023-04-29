# Getting Started

## Installation

### Supported Platforms

Merlon supports all platforms that the [Paper Mario decompilation](https://papermar.io) supports.

- Linux
    - Debian/Ubuntu
    - Arch Linux
    - openSUSE
    - Alpine Linux
    - Nix (nix-shell)
- macOS
- Windows Subsystem for Linux 2

Additionally, Merlon has a number of runtime dependencies that should be available on your PATH:

- `git`
- `ninja`
- `python3`
- `tar`
- `bzip2`
- `openssl`

````{important}
If you are using Windows, you must use the Windows Subsystem for Linux 2 (WSL 2).

If you don't have WSL, follow these instructions to install it:

1. Install or upgrade to WSL 2 following [these instructions](https://aka.ms/wsl2-install)
2. Open a WSL terminal
3. Run the following command:

```console
$ sudo apt update && sudo apt upgrade && cd ~
```

Crucially, **you must install Rust and Merlon in WSL** rather than Windows. All future commands in this guide should be run in WSL.
````

### Install Rust

Merlon is written in Rust. To install Merlon, you must first install Rust. You can do this by running the following
command:
```console
$ curl https://sh.rustup.rs -sSf | sh
```

### Build Merlon from source

```{note}
In future, Merlon executables will be available for direct download so users will not need to install Rust or build
Merlon from source.
```

Once you have installed Rust, you can install Merlon with the following command:

```console
$ cargo install merlon
```

### Verify Merlon is installed

You can verify Merlon is installed by running the following command:

```console
$ merlon --version
Merlon 1.1.0
```

## Creating a new package

Mods created with Merlon are called **packages**. To create a new package, run the following command:

```console
$ merlon new "My first Merlon package"

Created package: My first Merlon package by Alex Bates <alex@nanaian.town>
To build and run this package, run the following commands:

    cd "my-first-merlon-package"
    merlon init
    merlon run

```

### The package directory structure

Let's take a look at the package directory.

```
$ cd "my-first-merlon-package"
$ tree
.
├── LICENSE
├── README.md
├── merlon.toml
└── patches

1 directory, 3 files
```

This is the general structure of a Merlon package.

#### Package manifest

The most important file is `merlon.toml`, which is called the **package manifest**. If you open this file - for example,
in [Visual Studio Code](https://code.visualstudio.com/) - you'll see something like this:

```toml
dependencies = []

[package]
id = "61a75ba3-32fa-4f26-86e0-6dfb536b561d"
name = "My first Merlon package"
version = "0.1.0"
authors = ["Alex Bates <alex@nanaian.town>"]
description = "An amazing mod"
license = "CC-BY-SA-4.0"
keywords = []
```

The manifest contains information about your package, such as its name, version, and author. You can edit this file to
change the information about your package. However, do not modify the `id` field, as this is used to uniquely identify
your package.

#### Patches

The `patches` directory contains the patches that your package will apply to the game. We'll look at this in more
detail later, once we've actually made some changes.

#### README.md

The `README.md` file is a Markdown file that should describe how to use your package.

#### LICENSE

The `LICENSE` file contains the license for your package. By default, Merlon uses the
[Creative Commons Attribution-ShareAlike 4.0 International](https://creativecommons.org/licenses/by-sa/4.0/) license.

CC BY-SA 4.0 is a good license for mods, as it allows anyone to use your mod for any purpose, as long as they give you
credit and, if they make their own fork of it, share their changes under the same license.

If your package is only code, you may want to [use a software license](https://choosealicense.com/) instead.

You can change the license by editing the `license` field in the package manifest and updating this file.

## Initialisation

Now that we've created our package, we need to initialise it. This will clone the Paper Mario decompilation and apply
any patches in the package to it (which is currently none) so that we can build it.

Initialisation requires a **base ROM**. This should be an unmodified ROM of Paper Mario (N64), US release, in z64
format. To legally obtain this ROM, you can [dump it from the cartridge](https://dumping.guide/carts/nintendo/n64).
If your ROM is in a different format, you can convert it to z64 using
[N64 Rom Swapper](https://hack64.net/tools/swapper.php).

For example, if the base ROM is at `/home/alex/papermario.z64`, you would run the following command:

```console
$ merlon init --baserom /home/alex/papermario.z64
...
Switched to branch '61a75ba3-32fa-4f26-86e0-6dfb536b561d'
``` 

During initialisation, you may be prompted to install additional dependencies. Type `y` and press enter to say yes.

What did this command do? Let's take a look at the package directory again:

```
$ tree -a -L 1
.
├── .gitignore
├── .merlon
├── .vscode
├── LICENSE
├── README.md
├── merlon.toml
├── papermario
└── patches
```

You'll notice that there are a few new files and directories.

- `.gitignore` is a file that tells Git which files to ignore when staging changes.
- `.merlon` is a directory that contains Merlon's internal state. You should not modify this directory.
- `.vscode` is a directory that contains configuration for Visual Studio Code. You can open the package in Visual
  Studio Code by running `code .` in the package directory.
- `papermario` is a Git clone of the Paper Mario decompilation. This is where you will make your changes.

The command also created a new Git branch for your package. This branch is called the **package branch**. Generally,
Merlon expects that you are always on the package branch when using Merlon.

## Building

Now that we've initialised our package, we can build it into a ROM.

This will take a while the first time you run it, as it needs to build the Paper Mario decompilation. However, it will
be much faster on subsequent runs, because decomp supports incremental builds.

To build, run the following command:

```console
$ merlon build
...
Built: papermario/ver/us/build/papermario.z64 (SHA1: e1f9c77fa35549897ace8b8627e821a27309d538)
You can run this ROM with `merlon run`.
Warning: do not distribute this ROM. To distribute this package, use `merlon export`.
```

Because we have yet to make any changes, the output ROM will simply be a [shifted](glossary.md#Shiftability) version
of the base ROM. However, we can still run it in an emulator to verify that it works.
To do this, open the ROM in an N64 emulator of your choice. If you don't have an emulator, see
[Recommended Emulators](emulators.md).

Merlon can also run the ROM in an emulator for you. To do this, run the following command:

```console
$ merlon run
```

This will build the ROM if needed and then run it in an emulator. If you have an emulator installed in a common
location and Merlon can't find it, please [raise an issue](https://github.com/nanaian/merlon/issues/new)
saying which emulator it is and giving the path to the executable.

## Making changes

### Code changes

Let's make some changes to the game. We'll start by changing the music that plays on the title screen.

To do this, we need to find the code that plays the title screen music. We can do this by searching for the song,
`SONG_MAIN_THEME`, in the decompilation. To do this, run the following command:

```console
$ grep -nr SONG_MAIN_THEME papermario/src
papermario/src/state_title_screen.c:160:    bgm_set_song(0, SONG_MAIN_THEME, 0, 500, 8);
```

```{tip}
With Visual Studio Code, Ctrl + Shift + F lets you perform a package-wide text search similar to `grep`.
```

This tells us that the title screen music is played in `papermario/src/state_title_screen.c` on line 160.

Let's open this file in our editor (e.g. `code -g papermario/src/state_title_screen.c:160`) and take a look at that
line:

```c
bgm_set_song(0, SONG_MAIN_THEME, 0, 500, 8);
```

This line sets the song that plays on the main music player, 0, to `SONG_MAIN_THEME`. Let's change this to
`SONG_WHALE_THEME` instead:

```c
bgm_set_song(0, SONG_WHALE_THEME, 0, 500, 8);
```

```{tip}
With Visual Studio Code and the C/C++ extension, you can Ctrl + Click on a symbol to jump to its definition.

Ctrl + Click on `SONG_WHALE_THEME` to jump to its definition in `papermario/include/enums.h`. This will show you
the other songs that are available.
```

Now, let's build the ROM again and run it in an emulator!

```console
$ merlon run
```

You should hear the Whale Theme music playing on the title screen.

Now we know the change worked, we should commit it to Git. To do this, run the following commands:

```console
$ git -C papermario add papermario/src/state_title_screen.c
$ git -C papermario commit -m "play Whale Theme on title screen"
```

```{tip}
If you're using Visual Studio Code, you can stage and commit changes from the *Source Control* tab.
```

### Asset changes

This section is a work-in-progress. For now, look at
[Paper Capio](https://github.com/nanaian/pm-mods/tree/main/paper-capio) as an example.

## Dependencies

Has someone else made a Merlon package that you want to use in yours? You can add it as a dependency!

To do this, you'll need either:

1. A [distributable](glossary.md#distributable) of the package; or
2. The package, as a directory, for example by cloning it with Git.

As an example, we'll add "Skip developer logos" as a dependency. This is a package that skips the developer logos
when you start the game. Download `Skip-developer-logos.merlon` from the
[the latest release of nanaian/pm-mods](https://github.com/nanaian/pm-mods/releases/latest) into your package
directory.

After downloading, run the following command:

```console
$ merlon add --path Skip-developer-logos.merlon
Added dependency: Skip developer logos by Alex Bates <alex@nanaian.town>
```

This will add the package as a dependency. You can now build your package as normal, and the patches from the
dependency will be applied to your package.

```{note}
Users of your package **will** need to download the dependency themselves. Merlon does not automatically download
dependencies for you. A better solution for this is planned.
```

## Distribution

Once your package is complete, you can distribute your package to other people as a file.

To do this, run the following command:

```console
$ merlon export
```

This will export your package to a [distributable `.merlon` file](glossary.md#distributable).
