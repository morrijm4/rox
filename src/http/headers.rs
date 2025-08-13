use std::{collections::HashMap, fmt::Display, hash::Hash};

use super::StatusCode;

#[derive(Debug)]
pub struct Headers {
    map: HashMap<HeaderKey, String>,
    order: Vec<HeaderKey>,
}

impl Headers {
    pub fn new() -> Headers {
        Headers {
            map: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn parse(headers: &str) -> Result<Headers, StatusCode> {
        let mut map = Headers::new();

        for header in headers.split("\r\n") {
            let (key, value) = match header.split_once(':') {
                Some(h) => h,
                None => {
                    eprintln!("Invalid header: {}", header);
                    return Err(StatusCode::BadRequest);
                }
            };

            map.insert(key, value.trim());
        }

        Ok(map)
    }

    pub fn get(&self, key: impl Into<String>) -> Option<&String> {
        self.map.get(&HeaderKey::new(key.into()))
    }

    pub fn insert<K, V>(&mut self, key: K, value: V) -> Option<String>
    where
        K: Into<String>,
        V: ToString,
    {
        let key = HeaderKey::new(key.into());

        if !self.order.contains(&key) {
            self.order.push(key.clone());
        }

        self.map.insert(key, value.to_string())
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for key in &self.order {
            write!(f, "{}: {}\r\n", key, self.map[key])?;
        }

        Ok(())
    }
}

#[derive(Debug, Eq, Clone)]
struct HeaderKey {
    original: String,
    lowercase: String,
}

impl HeaderKey {
    fn new(s: String) -> Self {
        let lowercase = s.to_lowercase();
        Self {
            original: s,
            lowercase,
        }
    }
}

impl PartialEq for HeaderKey {
    fn eq(&self, other: &Self) -> bool {
        self.lowercase == other.lowercase
    }
}

impl Hash for HeaderKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.lowercase.hash(state);
    }
}

impl Display for HeaderKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.original)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_can_parse_headers() {
        let raw = concat!(
            "Host: google.com:80\r\n",
            "User-Agent: curl/8.7.1\r\n",
            "Proxy-Connection: Keep-Alive",
        );

        let headers = Headers::parse(raw).unwrap();

        assert!(matches!(headers.get("Host"), Some(host) if host == "google.com:80"));
        assert!(matches!(headers.get("USER-AGENT"), Some(host) if host == "curl/8.7.1"));
        assert!(matches!(headers.get("proxy-connection"), Some(host) if host == "Keep-Alive"));
        assert_eq!(format!("{}", headers), raw.to_string() + "\r\n");
    }
}
