# SPDX-FileCopyrightText: © 2026 Isaac Freund
# SPDX-License-Identifier: 0BSD

(if (dyn :install-time-syspath)
  (use @install-time-syspath/spork/declare-cc)
  (use spork/declare-cc))
(dofile "project.janet" :env (jpm-shim-env))
