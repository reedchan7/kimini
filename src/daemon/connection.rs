#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    origin: String,
    token: Option<String>,
}

impl Connection {
    pub fn new(origin: impl Into<String>, token: Option<String>) -> Self {
        Self {
            origin: origin.into().trim_end_matches('/').to_owned(),
            token,
        }
    }

    pub fn origin(&self) -> &str {
        &self.origin
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Kimi Web consumes the bearer token from the fragment and stores it.
    pub fn web_url(&self) -> String {
        match self.token() {
            Some(token) => format!("{}/#token={token}", self.origin),
            None => format!("{}/", self.origin),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_url_keeps_credentials_out_of_the_request() {
        let authenticated = Connection::new("http://127.0.0.1:58627/", Some("abc".into()));
        let anonymous = Connection::new("http://127.0.0.1:58627", None);

        assert_eq!(authenticated.origin(), "http://127.0.0.1:58627");
        assert_eq!(authenticated.web_url(), "http://127.0.0.1:58627/#token=abc");
        assert_eq!(anonymous.web_url(), "http://127.0.0.1:58627/");
    }
}
