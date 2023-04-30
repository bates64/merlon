[![Merlon](assets/logo/logotype.png)](https://merlon.readthedocs.io)

[![](https://img.shields.io/github/actions/workflow/status/nanaian/merlon/release.yml?branch=main)](https://github.com/nanaian/merlon/actions)
[![](https://img.shields.io/discord/279322074412089344?color=%237289DA&logo=discord&logoColor=ffffff)](https://discord.gg/paper-mario-modding-279322074412089344)
[![](https://img.shields.io/crates/v/merlon)](https://crates.io/crates/merlon)
[![](https://img.shields.io/pypi/v/merlon)](https://pypi.org/project/merlon/)

Merlon is a mod manager for the Paper Mario (N64) decompilation.

[Documentation](https://merlon.readthedocs.io/)

## Distributable file format

Merlon packages are distributed as `.merlon` files. These files are encrypted using the original game ROM, and cannot be used without the original game ROM. Unencrypted, it is a BZ2-compressed tarball of git patch files that can be applied to a copy of the decomp source code. Additionally, `.merlon` files contain only source code *changes*, so they are much smaller than the original game ROM. This means that you can use git to view the history of a mod, and to merge packages together. It also means that all distributables are source-available. This is in contrast to other formats, such as Star Rod's `.mod`, which distribute mods as binary patches that cannot be viewed or merged.

## Legal

This application is licensed under the Mozilla Public License 2.0. See the [LICENSE](LICENSE) file for details.

Mods created with this application are not covered by this license. Mods packaged into a file with this application are encrypted using the original game ROM, and cannot be used without the original game ROM. No guarantees are made about the legality of using this application to create mods.

The authors are not affiliated with Nintendo Co., Ltd. in any way.

The PAPER MARIO trademark owned by Nintendo Co., Ltd. is used in this modding tool under the fair use doctrine, solely for the purpose of enabling users to modify the game in a transformative manner.
