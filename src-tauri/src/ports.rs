use std::net::TcpListener;

pub fn is_port_available(port: u16) -> bool {
    TcpListener::bind(("0.0.0.0", port)).is_ok()
}

pub fn suggest_port(default_port: u16) -> u16 {
    if is_port_available(default_port) {
        return default_port;
    }
    for offset in 1..100 {
        let port = default_port + offset;
        if is_port_available(port) {
            return port;
        }
    }
    default_port
}
