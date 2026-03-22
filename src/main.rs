mod menubar;
mod server;

use std::time::Duration;

fn main() {
    let url = server::start_server_background();
    std::thread::sleep(Duration::from_millis(200));
    menubar::run(&url);
}
