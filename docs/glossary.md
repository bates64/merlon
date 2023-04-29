# Glossary

## Decomp

The Paper Mario (N64) decompilation. This is the source code that Merlon patches.

## Distributable

A distributable is a file that can be distributed to users. It contains the changes made by a package. Distributables
have the `.merlon` extension.

A distributable can be added as a dependency with `merlon add`.

A distributable's source code can be opened with `merlon open`.

## Package

A Merlon package is a mod created with Merlon. It is a directory containing a `merlon.toml` file, a `patches` directory,
a `README.md` file, and a `LICENSE` file.

## Package manifest

The package manifest is a TOML file called `merlon.toml` that contains information about the package.

## Patch

A patch is a file that contains a list of changes to be made to the game. Patches are stored in the `patches` directory
of a package. A patch file is effectively a Git commit in file form.

## Shiftability

A ROM is said to be "shifted" if it has been modified in a way that changes the location of data in the ROM.
For example, if we add a new texture to the ROM, the location of all the data after where the texture was added
will be shifted forward by the size of the texture.

This can be a source of very confusing bugs, so it's important to be aware that **bad shifting** can occur. If you
encounter a bug that seems to be caused by bad shifting, please
[open an issue](https://github.com/pmret/papermario/issues/1034).

To view known shifability issues, see [decomp issues marked "shiftability"](https://github.com/pmret/papermario/issues?q=is%3Aissue+is%3Aopen+label%3Ashiftability).

## ROM

A ROM is a file containing the game's code and data. Both N64 consoles and emulators can run ROMs.
