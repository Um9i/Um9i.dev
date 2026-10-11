//! Runtime configuration, read from the environment.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use thiserror::Error;

const DEFAULT_PORT: u16 = 8080;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid {name}={value:?}: {reason}")]
    Invalid {
        name: &'static str,
        value: String,
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub addr: SocketAddr,
    pub static_dir: PathBuf,
}

impl Config {
    /// Read configuration from the process environment.
    ///
    /// * `HOST` – bind address, defaults to `0.0.0.0` (required inside containers).
    /// * `PORT` – defaults to `8080`, which is what `DigitalOcean` App Platform expects.
    /// * `STATIC_DIR` – directory served at `/static`, defaults to `./static`.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let host = match lookup("HOST") {
            Some(value) => {
                value
                    .parse()
                    .map_err(|e: std::net::AddrParseError| ConfigError::Invalid {
                        name: "HOST",
                        value,
                        reason: e.to_string(),
                    })?
            }
            None => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        };

        let port = match lookup("PORT") {
            Some(value) => {
                value
                    .parse()
                    .map_err(|e: std::num::ParseIntError| ConfigError::Invalid {
                        name: "PORT",
                        value,
                        reason: e.to_string(),
                    })?
            }
            None => DEFAULT_PORT,
        };

        let static_dir =
            lookup("STATIC_DIR").map_or_else(|| PathBuf::from("static"), PathBuf::from);

        Ok(Self {
            addr: SocketAddr::new(host, port),
            static_dir,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_lookup_should_default_to_port_8080_on_all_interfaces() {
        let config = Config::from_lookup(|_| None).unwrap();
        assert_eq!(config.addr, "0.0.0.0:8080".parse().unwrap());
    }

    #[test]
    fn from_lookup_should_reject_non_numeric_port() {
        let result = Config::from_lookup(|key| (key == "PORT").then(|| "eighty".to_owned()));
        assert!(matches!(
            result,
            Err(ConfigError::Invalid { name: "PORT", .. })
        ));
    }
}
