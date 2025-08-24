use std::u16;

#[derive(Debug)]
pub struct Args {
    pub port: u16,
    pub protocol: Protocol,
    pub help: bool,
    pub version: bool,
}

impl Args {
    pub fn parse(it: &mut impl Iterator<Item = String>) -> Result<Self, String> {
        let mut port = 8080;
        let mut protocol = Protocol::HTTP;
        let mut help = false;
        let mut version = false;

        it.next(); // "rox"
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "-h" | "--help" => help = true,
                "-v" | "--version" => version = true,
                a if a.starts_with("-p") | a.starts_with("--port") => {
                    port = it
                        .next()
                        .ok_or("🚨 Error: no port provided 🚨")?
                        .parse()
                        .map_err(|_| "Error parsing port")?;
                }
                a if a.starts_with("-P") | a.starts_with("--protocol") => {
                    let proto_str = it.next().ok_or("🚨 Error: no protocol provided 🚨")?;

                    protocol = match proto_str.to_lowercase().as_str() {
                        "http" => Protocol::HTTP,
                        _ => return Err(format!("🚨 Unknown protocol: {} 🚨", proto_str)),
                    }
                }
                _ => return Err(format!("🚨 Invalid argument: {} 🚨", arg)),
            };
        }

        Ok(Self {
            port,
            protocol,
            help,
            version,
        })
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
        let mut it = ["rox", "--help"].into_iter().map(|s| s.to_string());

        let args = Args::parse(&mut it).unwrap();

        assert_eq!(args.help, true);
    }

    #[test]
    fn it_can_parse_h() {
        let mut it = ["rox", "-h"].into_iter().map(|s| s.to_string());

        let args = Args::parse(&mut it).unwrap();

        assert_eq!(args.help, true);
    }

    #[test]
    fn it_can_parse_version() {
        let mut it = ["rox", "--version"].into_iter().map(|s| s.to_string());

        let args = Args::parse(&mut it).unwrap();

        assert_eq!(args.version, true);
    }

    #[test]
    fn it_can_parse_v() {
        let mut it = ["rox", "-v"].into_iter().map(|s| s.to_string());

        let args = Args::parse(&mut it).unwrap();

        assert_eq!(args.version, true);
    }

    #[test]
    fn it_can_prase_port() {
        let mut it = ["rox", "--port", "9000"].into_iter().map(|s| s.to_string());

        let args = Args::parse(&mut it).unwrap();

        assert_eq!(args.port, 9000);
    }

    #[test]
    fn it_can_prase_p() {
        let mut it = ["rox", "-p", "7000"].into_iter().map(|s| s.to_string());

        let args = Args::parse(&mut it).unwrap();

        assert_eq!(args.port, 7000);
    }

    #[test]
    fn it_can_prase_protocol() {
        let mut it = ["rox", "--protocol", "HTTP"]
            .into_iter()
            .map(|s| s.to_string());

        let args = Args::parse(&mut it).unwrap();

        assert_eq!(args.protocol, Protocol::HTTP);
    }
}
