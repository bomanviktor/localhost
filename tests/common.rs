#![allow(dead_code)]

use localhost::server::{servers, start};

pub fn setup() {
    start(servers())
}
