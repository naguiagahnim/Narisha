use std::collections::HashMap;

use wayland_backend::client::ObjectId;
use wayland_client::QueueHandle;

use crate::river::{
    river_node_v1::RiverNodeV1,
    river_output_v1::RiverOutputV1,
    river_pointer_binding_v1::RiverPointerBindingV1,
    river_seat_v1::RiverSeatV1,
    river_window_manager_v1::RiverWindowManagerV1,
    river_window_v1::{Edges, RiverWindowV1},
    river_xkb_binding_v1::RiverXkbBindingV1,
    river_xkb_bindings_v1::RiverXkbBindingsV1,
};

use crate::wm::WindowManager;
use crate::wm::actions::Action;
use crate::wm::actions::SeatOp;

#[derive(Debug, Default)]
pub struct AppData {
    pub river_wm: Option<RiverWindowManagerV1>,
    pub river_xkb: Option<RiverXkbBindingsV1>,
    pub wm: WindowManager,
}

#[derive(Debug)]
pub struct Window {
    pub proxy: RiverWindowV1,
    pub node: RiverNodeV1,
    pub new: bool,
    pub closed: bool,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub pointer_move_requested: Option<RiverSeatV1>,
    pub pointer_resize_requested: Option<RiverSeatV1>,
    pub pointer_resize_requested_edges: Edges,
}

impl Window {
    pub fn new(proxy: RiverWindowV1, qh: &QueueHandle<AppData>, (x, y): (i32, i32)) -> Self {
        let node = proxy.get_node(qh, ());
        Window {
            proxy,
            node,
            new: true,
            closed: false,
            x,
            y,
            width: 0,
            height: 0,
            pointer_move_requested: None,
            pointer_resize_requested: None,
            pointer_resize_requested_edges: Edges::None,
        }
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.node.set_position(x, y);
        self.x = x;
        self.y = y;
    }
}

#[derive(Debug)]
pub struct Output {
    pub proxy: RiverOutputV1,
    pub removed: bool,
}

impl Output {
    pub fn new(proxy: RiverOutputV1) -> Self {
        Self {
            proxy,
            removed: false,
        }
    }
}

#[derive(Debug)]
pub struct Seat {
    pub proxy: RiverSeatV1,
    pub new: bool,
    pub removed: bool,
    pub focused: Option<RiverWindowV1>,
    pub hovered: Option<RiverWindowV1>,
    pub interacted: Option<RiverWindowV1>,
    pub xkb_bindings: HashMap<ObjectId, XkbBinding>,
    pub pointer_bindings: HashMap<ObjectId, PointerBinding>,
    pub pending_action: Action,
    pub op: SeatOp,
    pub op_dx: i32,
    pub op_dy: i32,
    pub op_release: bool,
}

impl Seat {
    pub fn new(proxy: RiverSeatV1) -> Self {
        Self {
            proxy,
            new: true,
            removed: false,
            focused: None,
            hovered: None,
            interacted: None,
            xkb_bindings: HashMap::new(),
            pointer_bindings: HashMap::new(),
            pending_action: Action::None,
            op: SeatOp::None,
            op_dx: 0,
            op_dy: 0,
            op_release: false,
        }
    }
}

#[derive(Debug)]
pub struct XkbBinding {
    pub proxy: RiverXkbBindingV1,
    pub action: Action,
}

#[derive(Debug)]
pub struct PointerBinding {
    pub proxy: RiverPointerBindingV1,
    pub action: Action,
}
