use localhost::log::rename_server_and_client_logs;
use localhost::server::{servers, start};

fn main() {
    rename_server_and_client_logs();
    start(servers());
}
