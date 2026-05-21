use std::collections::VecDeque;

use crate::river::river_window_manager_v1::RiverWindowManagerV1;
use crate::river::river_window_v1::{Edges, RiverWindowV1};
use crate::state::{Seat, Window};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    None,
    SpawnKitty,
    Close,
    FocusNext,
    Move,
    Resize,
    Exit,
}

#[derive(Debug, Clone)]
pub enum SeatOp {
    None,
    Move {
        window_proxy: RiverWindowV1,
        start_x: i32,
        start_y: i32,
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
        match self.pending_action {
            Action::None => {}
            Action::SpawnKitty => match std::process::Command::new("kitty")
                .env_remove("WAYLAND_DEBUG")
                .spawn()
            {
                Ok(_) => {}
                Err(e) => eprintln!("Failed to spawn kitty: {e}"),
            },
            Action::Close => {
                if let Some(window_proxy) = self.focused.as_ref() {
                    window_proxy.close();
                }
            }
            Action::FocusNext => {
                windows.rotate_left(1);
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
