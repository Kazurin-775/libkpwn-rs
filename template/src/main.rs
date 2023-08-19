use kpwn::all::*;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    whoami();

    let dev = open_dev("/dev/foo").unwrap();

    dev.close().unwrap();
}
