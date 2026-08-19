// SPDX-FileCopyrightText: © 2026 Agahnim
// SPDX-License-Identifier: GPL-3.0-only

#include <stdbool.h>
#include <wayland-util.h>

typedef union {
  const void *v;
  int i;
  unsigned int ui;
  float f;
} Arg;

struct Output {
  struct river_output_v1 *obj;
  bool removed;
  struct wl_list link; // WindowManager.outputs

  int width;
  int height;
  int x;
  int y;
};

enum Layout {
  Monocle,
  Master,
};

struct Window {
  struct river_window_v1 *obj;
  struct river_node_v1 *node;

  bool new;
  bool closed;

  int32_t x;
  int32_t y;
  int32_t width;
  int32_t height;

  struct Seat *pointer_move_requested;
  struct Seat *pointer_resize_requested;
  uint32_t pointer_resize_requested_edges;

  struct wl_list link; // WindowManager.windows
};

enum Action {
  ACTION_NONE,
  ACTION_SPAWN,
  ACTION_CLOSE,
  ACTION_FOCUS_NEXT,
  ACTION_MOVE,
  ACTION_RESIZE,
  ACTION_EXIT,
};

struct XkbBinding {
  struct river_xkb_binding_v1 *obj;
  struct Seat *seat;
  enum Action action;
  struct wl_list link;
  Arg arg;
};

struct PointerBinding {
  struct river_pointer_binding_v1 *obj;
  struct Seat *seat;
  enum Action action;
  struct wl_list link;
  Arg arg;
};

enum SeatOp {
  SEAT_OP_NONE,
  SEAT_OP_MOVE,
  SEAT_OP_RESIZE,
};

struct Seat {
  struct river_seat_v1 *obj;
  bool new;
  bool removed;

  struct Window *focused;
  struct Window *hovered;
  struct Window *interacted;

  struct wl_list xkb_bindings;     // XkbBinding
  struct wl_list pointer_bindings; // PointerBinding
  enum Action pending_action;

  enum SeatOp op;
  // For SEAT_OP_MOVE and SEAT_OP_RESIZE
  struct Window *op_window;
  int32_t op_start_x, op_start_y;
  int32_t op_dx, op_dy;
  bool op_release;
  // For SEAT_OP_RESIZE only
  int32_t op_start_width, op_start_height;
  uint32_t op_edges;

  Arg pending_arg;

  struct wl_list link; // WindowManager.seats
};

struct WindowManager {
  struct wl_list outputs; // Output
  struct wl_list windows; // Window
  struct wl_list seats;   // Seat
};