use local_ip_address::local_ip;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};

pub type Port = u16;

pub struct Addr {
    local_ip: Ipv4Addr,
    network_ip: IpAddr,
}

impl Addr {
    pub fn new() -> Self {
        Addr {
            local_ip: Ipv4Addr::new(127, 0, 0, 1),
            network_ip: local_ip().unwrap(),
        }
    }
    pub fn is_free_port(&self, port: Port) -> Option<(SocketAddr, SocketAddr)> {
        let Addr {
            local_ip,
            network_ip,
        } = self;
        let local_addr = SocketAddr::new(IpAddr::V4(*local_ip), port);
        let network_addr = SocketAddr::new(*network_ip, port);
        if TcpListener::bind(local_addr).is_ok() | TcpListener::bind(network_addr).is_ok() {
            return Some((local_addr, network_addr));
        }
        None
    }
    pub fn get_address(&self, port: Port) -> (SocketAddr, SocketAddr) {
        if let Some(addr) = self.is_free_port(port) {
            return addr;
        }
        let start = 7878;
        let end = 8989;
        for other_port in start..end {
            if other_port == port {
                continue;
            }
            if let Some(addr) = self.is_free_port(other_port) {
                return addr;
            }
        }
        panic!("the {} port is not free, chost try to find a free port from {} to {}, but also didn\'t find a free port, please use --port flag to specify a free port", port, start, end)
    }
}

impl Default for Addr {
    fn default() -> Self {
        Self::new()
    }
}
