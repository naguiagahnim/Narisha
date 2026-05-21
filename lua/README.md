<!--
SPDX-FileCopyrightText: © 2026 FireFly
SPDX-License-Identifier: 0BSD
-->

# tinyrwm.lua

Tiny river window manager implemented in Lua.

## Dependencies

System dependencies:
- lua (5.4 tested)
- luarocks
- libwayland
- libxkbcommon

The lua-ecosystem dependencies should be handled by luarocks.

## Building

To fetch lua dependencies, build, and install to `~/.luarocks/bin`, run:

```sh
eval $(luarocks --path bin)
luarocks --local make
```

## Running

Make sure libxkbcommon.so and libwayland-client.so are present in
`LD_LIBRARY_PATH` (and `river` and `foot` in your `PATH`).  You should be able
to run river with the installed Lua tinyrwm with

```sh
river -c ~/.luarocks/bin/tinyrwm
```
