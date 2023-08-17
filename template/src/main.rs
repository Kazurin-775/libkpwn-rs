use std::fs::OpenOptions;

use kpwn::all::*;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    whoami();

    let dev = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/foo")
        .unwrap();

    dev.close().unwrap();
}
