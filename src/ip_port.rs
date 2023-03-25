use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::ops::Range;

type Port = u16;

pub fn get_used_port_in_tcp(range: Range<Port>) -> Option<Port> {
    for port in range {
        if is_free_in_tcp(port) {
            return Some(port);
        }
    }
    return None;
}

pub fn is_free_in_tcp(port: Port) -> bool {
    let ipv4 = SocketAddrV4::new(create_default_ipv4(), port);
    TcpListener::bind(ipv4).is_ok()
}

pub fn create_default_ipv4() -> Ipv4Addr {
    Ipv4Addr::new(127, 0, 0, 1)
}
