use colored::{ColoredString, Colorize};
use std::net::SocketAddr;

pub fn get_full_addr_string(addr: &SocketAddr) -> ColoredString {
    let mut addr_string = addr.to_string();
    addr_string.insert_str(0, "http://");
    addr_string.blue()
}
