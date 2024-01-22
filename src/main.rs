use localhost::log::init_logs;
use localhost::server::start;
use localhost::server_config::server_config;

fn main() {
    init_logs();
    start(server_config());
}

#[test]
fn test_main() {
    std::thread::spawn(|| main());
}
