# Star Rod

Star Rod is a modding tool created by Clover. While Star Rod can be used to create mods on its own, it does not support
writing mods in C, and it does not support adding other mods as dependencies. Merlon, however, leverages the
[Paper Mario decompilation](https://papermar.io) to provide much more powerful modding capabilities.

## Comparison with Star Rod

| Feature                                              | Star Rod | Merlon                 |
| ---------------------------------------------------- | -------- | ---------------------- |
| Open source                                          | No       | Yes                    |
| Mod management                                       | Yes      | Yes                    |
| Supported base ROM versions                          | 🇺🇸       | 🇺🇸                     |
| Distributable format                                 | `.mod`   | `.merlon`              |
| View source code of distributable                    | No       | Yes                    |
| Dependency management                                | No       | Yes                    |
| C language support                                   | No       | Yes (decomp)           |
| `.mpat`/`.bpat` language support                     | No       | Yes                    |
| Compile modded ROM                                   | Yes      | Yes (`merlon build`)   |
| Easily run modded ROM in emulator                    | No (manual) | Yes (`merlon run`)  |
| GUI Map editor                                       | Yes      | No (use Star Rod)      |
| GUI Sprite editor                                    | Yes      | No (use Star Rod)      |
| GUI World editor                                     | Yes      | No                     |
| GUI Image editor                                     | Yes      | No                     |
| CLI-GUI feature parity                               | No       | Yes                    |

## Using the Star Rod Map and Sprite Editors with Merlon

Star Rod's map and sprite editors can be used with Merlon. To do so:

1. Open Star Rod.
2. Open the *Mod Manager*.
3. Change the *Mod Folder* to the `papermario` subdirectory of your [initialised](getting_started.md#initialisation) Merlon package.
4. Ensure that assets are dumped. Do not click _Copy Assets to Mod_.
4. Close and reopen Star Rod.
5. Open the *Map Editor* or *Sprite Editor*.
