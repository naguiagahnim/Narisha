mod dispatch;
mod river;
mod state;
mod wm;

use state::AppData;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = wayland_client::Connection::connect_to_env()?;
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let _registry = display.get_registry(&event_queue.handle(), ());

    let mut app_data = AppData::default();

    event_queue.roundtrip(&mut app_data)?;
    if app_data.river_wm.is_none() {
        eprintln!("river_window_manager_v1 global not found! Is river running?");
        std::process::exit(1);
    }
    if app_data.river_xkb.is_none() {
        eprintln!("river_xkb_bindings_v1 global not found! Is river running with xkb support?");
        std::process::exit(1);
    }
    if app_data.river_layershell.is_none() {
        eprintln!(
            "river_layer_shell_v1 global not found! Is river running with layershell support?"
        );
        std::process::exit(1);
    }

    loop {
        event_queue.blocking_dispatch(&mut app_data)?;
    }
}
