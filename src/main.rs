
mod evenger;
mod evdev;
mod foreign;
mod muxer;

use evenger::Evenger;

fn main() {
    let mut app = Evenger::new()
        .expect("app init failed");
    
    app.open_device("mouse", "/dev/input/event2")
        .expect("can't open mouse");
    app.open_device("keyboard", "/dev/input/event4")
        .expect("can't open keyboard");

    app.run()
        .expect("error during runtime");
}