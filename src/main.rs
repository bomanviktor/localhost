use localhost::log::init_logs;
use localhost::server::{servers, start};

fn main() {
    init_logs();
    start(servers());
}
