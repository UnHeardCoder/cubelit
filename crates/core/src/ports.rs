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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_port_returns_true() {
        // Bind a random high port and verify is_port_available reports it as taken,
        // then after the listener drops it reports it as free.
        let listener = TcpListener::bind("0.0.0.0:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(!is_port_available(port), "port in use should not be available");
        drop(listener);
        assert!(is_port_available(port), "released port should be available");
    }

    #[test]
    fn suggest_port_returns_default_when_free() {
        // Find a free port, then verify suggest_port returns it unchanged.
        let listener = TcpListener::bind("0.0.0.0:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        assert_eq!(suggest_port(port), port);
    }

    #[test]
    fn suggest_port_increments_when_default_taken() {
        // Hold the default port, then verify suggest_port finds the next free one.
        let listener = TcpListener::bind("0.0.0.0:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let suggested = suggest_port(port);
        // The suggested port must differ from the held one and must actually be free.
        assert_ne!(suggested, port);
        assert!(is_port_available(suggested));
    }
}
