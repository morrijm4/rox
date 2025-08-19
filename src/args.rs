use std::{env, u16};

#[derive(Debug)]
pub struct Args {
    pub port: u16,
    pub protocol: Protocol,
    pub help: bool,
    pub version: bool,
}

impl Args {
    pub fn parse() -> Result<Self, String> {
        Args::parse_with_iter(env::args())
    }

    fn parse_with_iter(it: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut port = 8080;
        let mut protocol = None;
        let mut help = false;
        let mut version = false;

        for arg in it.skip(1) {
            match arg.as_str() {
                "-h" | "--help" => help = true,
                "-v" | "--version" => version = true,
                a if a.starts_with("-p") | a.starts_with("--port") => {
                    port = Self::parse_key_value(a)?
                        .parse::<u16>()
                        .map_err(|_| a.to_string())?;
                }
                a if a.starts_with("-P") | a.starts_with("--protocol") => {
                    let protocol_str = Self::parse_key_value(a)?;

                    protocol = match protocol_str.to_lowercase().as_str() {
                        "http" => Some(Protocol::HTTP),
                        _ => return Err(format!("🚨 Unknown protocol: {} 🚨", arg)),
                    };
                }
                _ => return Err(format!("🚨 Invalid argument: {} 🚨", arg)),
            };
        }

        Ok(Self {
            port,
            protocol: protocol.ok_or("🚨 Protocol is required. 🚨")?,
            help,
            version,
        })
    }

    fn parse_key_value(s: &str) -> Result<String, String> {
        s.split_once("=")
            .map(|(_key, value)| value.to_string())
            .ok_or_else(|| s.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub enum Protocol {
    HTTP,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_can_parse_help() {
        let it = ["rox", "--help"].into_iter().map(|s| s.to_string());

        let args = Args::parse_with_iter(it).unwrap();

        assert_eq!(args.help, true);
    }

    #[test]
    fn it_can_parse_h() {
        let it = ["rox", "-h"].into_iter().map(|s| s.to_string());

        let args = Args::parse_with_iter(it).unwrap();

        assert_eq!(args.help, true);
    }

    #[test]
    fn it_can_parse_version() {
        let it = ["rox", "--version"].into_iter().map(|s| s.to_string());

        let args = Args::parse_with_iter(it).unwrap();

        assert_eq!(args.version, true);
    }

    #[test]
    fn it_can_parse_v() {
        let it = ["rox", "-v"].into_iter().map(|s| s.to_string());

        let args = Args::parse_with_iter(it).unwrap();

        assert_eq!(args.version, true);
    }

    #[test]
    fn it_can_prase_port() {
        let it = ["rox", "--port=9000"].into_iter().map(|s| s.to_string());

        let args = Args::parse_with_iter(it).unwrap();

        assert_eq!(args.port, 9000);
    }

    #[test]
    fn it_can_prase_p() {
        let it = ["rox", "-p=7000"].into_iter().map(|s| s.to_string());

        let args = Args::parse_with_iter(it).unwrap();

        assert_eq!(args.port, 7000);
    }

    #[test]
    fn it_can_prase_protocol() {
        let it = ["rox", "--protocol=HTTP"]
            .into_iter()
            .map(|s| s.to_string());

        let args = Args::parse_with_iter(it).unwrap();

        assert_eq!(args.protocol, Protocol::HTTP);
    }
}
