use std::collections::VecDeque;

use crate::river::river_window_manager_v1::RiverWindowManagerV1;
use crate::river::river_window_v1::{Edges, RiverWindowV1};
use crate::state::{Seat, Window};

#[derive(Debug, Clone)]
pub enum Action {
    None,
    Spawn(Vec<String>),
    Close,
    FocusRight,
    FocusLeft,
    Move,
    Resize,
    Exit,
    Fullscreen,
}

#[derive(Debug, Clone)]
pub enum SeatOp {
    None,
    Move {
        window_proxy: RiverWindowV1,
        start_x: i32,
        start_y: i32,
    },
    Spawn {
        window_proxy: RiverWindowV1,
    },
    Resize {
        window_proxy: RiverWindowV1,
        start_x: i32,
        start_y: i32,
        start_width: i32,
        start_height: i32,
        edges: Edges,
    },
}

impl Seat {
    pub fn do_action(&mut self, windows: &mut VecDeque<Window>, wm_proxy: &RiverWindowManagerV1) {
        match &self.pending_action {
            Action::None => {}
            Action::Spawn(cmd) => match std::process::Command::new(&cmd[0])
                .args(&cmd[1..])
                .env_remove("WAYLAND_DEBUG")
                .spawn()
            {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to spawn process: {e}"),
            },
            Action::Fullscreen => {
                if let Some(window_proxy) = self.focused.as_ref() {
                    // window_proxy.fullscreen(window_proxy.get);
                }
            }
            Action::Close => {
                if let Some(window_proxy) = self.focused.as_ref() {
                    window_proxy.close();
                }
            }
            Action::FocusRight => {
                windows.rotate_left(1);
                self.focus_top(windows);
            }
            Action::FocusLeft => {
                windows.rotate_right(1);
                self.focus_top(windows);
            }

            Action::Move => {
                if let (Some(window_proxy), SeatOp::None) = (self.hovered.as_ref(), &self.op) {
                    let window = windows
                        .iter()
                        .find(|w| &w.proxy == window_proxy)
                        .expect("Hovered window not found");
                    self.pointer_move(window);
                }
            }
            Action::Resize => {
                if let (Some(window_proxy), SeatOp::None) = (self.hovered.as_ref(), &self.op) {
                    let window = windows
                        .iter()
                        .find(|w| &w.proxy == window_proxy)
                        .expect("Hovered window not found");
                    self.pointer_resize(window, Edges::Bottom.union(Edges::Right));
                }
            }
            Action::Exit => wm_proxy.exit_session(),
        }
        self.pending_action = Action::None;
    }
}
