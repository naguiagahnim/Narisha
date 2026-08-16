<!--
SPDX-FileCopyrightText: © 2026 Agahnim
SPDX-License-Identifier: Unlicense
-->

# Narisha

A tiling window manager for River

## Dependencies

The following system dependencies are required:

- pkg-config / u-config
- meson / muon
- ninja / samurai
- wayland
- xkbcommon

The "development" versions are required if applicable to your distribution.

## Building

```sh
{ meson | muon } setup build
{ ninja | samu } -C build
```

## Running

```
river -c ./build/narisha
```
