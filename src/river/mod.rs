pub extern crate wayland_client;
pub use wayland_client::protocol::*;

mod interfaces {
    pub(super) mod rwm {
        pub use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("./protocol/river-window-management-v1.xml");
    }
    pub(super) mod rxkb {
        use super::rwm::*;
        wayland_scanner::generate_interfaces!("./protocol/river-xkb-bindings-v1.xml");
    }

    pub(super) mod rlayer {
        use super::rwm::*;
        wayland_scanner::generate_interfaces!("./protocol/river-layer-shell-v1.xml");
    }
    pub(super) mod rinput {
        use super::rwm::*;
        wayland_scanner::generate_interfaces!("./protocol/river-input-management-v1.xml");
    }
    pub(super) mod rxkb_config {
        use super::rinput::*;
        use super::rwm::*;
        wayland_scanner::generate_interfaces!("./protocol/river-xkb-config-v1.xml");
    }
}

use self::interfaces::rinput::*;
use self::interfaces::rlayer::*;
use self::interfaces::rwm::*;
use self::interfaces::rxkb::*;
use self::interfaces::rxkb_config::*;
wayland_scanner::generate_client_code!("./protocol/river-window-management-v1.xml");
wayland_scanner::generate_client_code!("./protocol/river-xkb-bindings-v1.xml");
wayland_scanner::generate_client_code!("./protocol/river-layer-shell-v1.xml");
wayland_scanner::generate_client_code!("./protocol/river-input-management-v1.xml");
wayland_scanner::generate_client_code!("./protocol/river-xkb-config-v1.xml");
