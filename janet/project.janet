# SPDX-FileCopyrightText: © 2026 Isaac Freund
# SPDX-License-Identifier: 0BSD

(declare-project
  :name "tinyrwm"
  :description "Tiny river window manager"
  :author "Isaac Freund"
  :dependencies [{:url "https://github.com/janet-lang/spork"}
                 {:url "https://codeberg.org/ifreund/janet-wayland"}
                 {:url "https://codeberg.org/ifreund/janet-xkbcommon"}]
  :version "0.0.0")

(declare-executable
  :name "tinyrwm"
  :entry "tinyrwm.janet"
  :install true
  :pkg-config-libs ["wayland-client" "xkbcommon"])
