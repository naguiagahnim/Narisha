<!--
SPDX-FileCopyrightText: © 2026 Isaac Freund
SPDX-License-Identifier: 0BSD
-->

# tinyrwm.janet

Tiny river window manager implemented in [Janet](https://janet-lang.org).

## Dependencies

System dependencies:
- janet 1.38.0 or newer
- libwayland 1.25.0 or newer
- libxkbcommon

Install [spork](https://github.com/janet-lang/spork), which includes `janet-pm`.
I recommend setting `$JANET_PATH` to e.g. `~/.local/janet` in your shell `.profile`
if you do not what to install spork system-wide using root privileges.

```
git clone https://github.com/janet-lang/spork
cd spork
janet --install .
```

Optionally, create and activate a virtual environment with:

```
janet-pm env venv
source ./venv/bin/activate
```

Fetch and build janet dependencies with:

```
janet-pm deps
```

## Building

With all dependencies installed tinyrwm.janet can be run directly with

```
janet tinyrwm.janet
```

A standalone executable can also be built with
```
janet-pm build
```
