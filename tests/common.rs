#![allow(dead_code)]

use localhost::log::init_logs;
use localhost::server::{servers, start};
pub fn setup() {
    init_logs();
    start(servers())
}
