use std::net::SocketAddr;

pub struct Config {
    pub addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Self {
        let port = std::env::var("BOOMAI_PORT")
            .unwrap_or_else(|_| "3030".to_string())
            .parse::<u16>()
            .expect("Invalid port number");

        let addr = SocketAddr::from(([127, 0, 0, 1], port));

        Self { addr }
    }
}

