pub mod actions;
pub mod bindings;
pub mod layout;

use std::collections::{HashMap, VecDeque};

use wayland_backend::client::ObjectId;
use wayland_client::QueueHandle;

use crate::river::wayland_client::Proxy;
use crate::river::{
    river_seat_v1::Modifiers, river_window_manager_v1::RiverWindowManagerV1,
    river_window_v1::Edges, river_xkb_bindings_v1::RiverXkbBindingsV1,
};
use crate::state::{AppData, Output, Seat, Window};
use crate::wm::actions::{Action, SeatOp};

#[derive(Debug, Default)]
pub struct WindowManager {
    pub windows: VecDeque<Window>,
    pub outputs: HashMap<ObjectId, Output>,
    pub seats: HashMap<ObjectId, Seat>,
}

impl WindowManager {
    pub fn handle_manage_start(
        &mut self,
        proxy: &RiverWindowManagerV1,
        river_xkb: &RiverXkbBindingsV1,
        qh: &QueueHandle<AppData>,
    ) {
        self.remove_outputs();
        self.remove_windows();
        self.remove_seats();
        self.init_new_windows();
        self.init_new_seats(river_xkb, qh);
        self.manage_windows();
        self.manage_seats(proxy);
        proxy.manage_finish();
    }

    pub fn handle_render_start(&mut self, proxy: &RiverWindowManagerV1) {
        for seat in self.seats.values_mut() {
            match &seat.op {
                SeatOp::None => {}
                SeatOp::Move {
                    window_proxy,
                    start_x,
                    start_y,
                } => {
                    if let Some(window) = self.windows.iter_mut().find(|w| &w.proxy == window_proxy)
                    {
                        window.set_position(start_x + seat.op_dx, start_y + seat.op_dy);
                    }
                }
                SeatOp::Resize {
                    window_proxy,
                    start_x,
                    start_y,
                    start_width,
                    start_height,
                    edges,
                } => {
                    if let Some(window) = self.windows.iter_mut().find(|w| &w.proxy == window_proxy)
                    {
                        let (mut x, mut y) = (*start_x, *start_y);
                        if edges.contains(Edges::Left) {
                            x += start_width - window.width;
                        }
                        if edges.contains(Edges::Top) {
                            y += start_height - window.height;
                        }
                        window.set_position(x, y);
                    }
                }
            }
        }
        proxy.render_finish();
    }

    fn remove_outputs(&mut self) {
        self.outputs.retain(|_, output| {
            if output.removed {
                output.proxy.destroy();
                return false;
            }
            true
        });
    }

    fn remove_windows(&mut self) {
        let old_windows = std::mem::take(&mut self.windows);
        self.windows = old_windows
            .into_iter()
            .filter(|window| {
                if window.closed {
                    for seat in self.seats.values_mut() {
                        if let SeatOp::Move { window_proxy, .. }
                        | SeatOp::Resize { window_proxy, .. } = &seat.op
                        {
                            if window_proxy == &window.proxy {
                                seat.op_end();
                            }
                        }
                    }
                    return false;
                }
                true
            })
            .collect();
    }

    fn remove_seats(&mut self) {
        self.seats.retain(|_, seat| {
            if seat.removed {
                seat.xkb_bindings
                    .values_mut()
                    .for_each(|b| b.proxy.destroy());
                seat.pointer_bindings
                    .values_mut()
                    .for_each(|b| b.proxy.destroy());
                seat.proxy.destroy();
                return false;
            }
            true
        });
    }

    fn init_new_windows(&mut self) {
        for window in self.windows.iter_mut().filter(|w| w.new) {
            window.set_position(window.x, window.y);
            window.proxy.propose_dimensions(window.width, window.height);
            window.new = false;
        }
    }

    fn init_new_seats(&mut self, river_xkb: &RiverXkbBindingsV1, qh: &QueueHandle<AppData>) {
        const SPACE: u32 = 0x20;
        const N: u32 = 0x6e;
        const Q: u32 = 0x71;
        const ESC: u32 = 0xff1b;
        const BTN_LEFT: u32 = 0x110;
        const BTN_RIGHT: u32 = 0x111;
        let mods = Modifiers::Mod4;

        for seat in self.seats.values_mut() {
            if seat.new {
                seat.create_xkb_binding(
                    river_xkb,
                    qh,
                    mods,
                    SPACE,
                    Action::Spawn(vec!["kitty".to_string()]),
                );
                seat.create_xkb_binding(river_xkb, qh, mods, Q, Action::Close);
                seat.create_xkb_binding(river_xkb, qh, mods, N, Action::FocusNext);
                seat.create_xkb_binding(river_xkb, qh, mods, ESC, Action::Exit);
                seat.create_pointer_binding(qh, mods, BTN_LEFT, Action::Move);
                seat.create_pointer_binding(qh, mods, BTN_RIGHT, Action::Resize);
                seat.new = false;
            }
        }
    }

    fn manage_windows(&mut self) {
        for window in self.windows.iter_mut() {
            if let Some(seat_proxy) = window.pointer_move_requested.take() {
                let seat = self
                    .seats
                    .get_mut(&seat_proxy.id())
                    .expect("Seat not found");
                seat.pointer_move(window);
            }
            if let Some(seat_proxy) = window.pointer_resize_requested.take() {
                let seat = self
                    .seats
                    .get_mut(&seat_proxy.id())
                    .expect("Seat not found");
                seat.pointer_resize(window, window.pointer_resize_requested_edges);
            }
        }
    }

    fn manage_seats(&mut self, wm_proxy: &RiverWindowManagerV1) {
        for seat in self.seats.values_mut() {
            if let Some(window_proxy) = seat.interacted.take() {
                let i = self
                    .windows
                    .iter()
                    .position(|w| w.proxy == window_proxy)
                    .expect("Interacted window not found");
                let window = self.windows.remove(i).unwrap();
                self.windows.push_back(window);
            }
            seat.focus_top(&self.windows);
            seat.do_action(&mut self.windows, wm_proxy);
            if seat.op_release {
                seat.op_end();
                seat.op_release = false;
            } else {
                seat.op_manage();
            }
        }
    }
}
