use crate::IntoSubdomain;
use crate::Result;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use std::{error::Error, fmt};

#[derive(Deserialize, Hash, PartialEq, Debug, Eq)]
struct CrtshResult {
    name_value: String,
}

impl IntoSubdomain for Vec<CrtshResult> {
    fn subdomains(&self) -> HashSet<String> {
        self.iter().map(|s| s.name_value.to_owned()).collect()
    }
}

#[derive(Debug)]
struct CrtshError {
    host: Arc<String>,
}

impl CrtshError {
    fn new(host: Arc<String>) -> Self {
        Self { host: host }
    }
}

impl Error for CrtshError {}

impl fmt::Display for CrtshError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Crtsh couldn't find any results for: {}", self.host)
    }
}

fn build_url(host: &str) -> String {
    format!("https://crt.sh/?q=%.{}&output=json", host)
}

pub async fn run(host: Arc<String>) -> Result<HashSet<String>> {
    let uri = build_url(&host);
    let resp: Option<Vec<CrtshResult>> = surf::get(uri).recv_json().await?;

    match resp {
        Some(data) => Ok(data.subdomains()),
        None => Err(Box::new(CrtshError::new(host))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_await_test::async_test;

    #[test]
    fn url_builder() {
        let correct_uri = "https://crt.sh/?q=%.hackerone.com&output=json";
        assert_eq!(correct_uri, build_url("hackerone.com"));
    }

    #[ignore]
    #[async_test]
    async fn returns_results() {
        let host = Arc::new("hackerone.com".to_owned());
        let results = run(host).await.unwrap();
        assert!(results.len() > 5);
    }

    #[ignore] // tests passing locally but failing on linux ci?
    #[async_test]
    async fn handle_no_results() {
        let host = Arc::new("anVubmxpa2VzdGVh.com".to_string());
        let res = run(host).await;
        let e = res.unwrap_err();
        assert_eq!(
            e.to_string(),
            "Crtsh couldn't find any results for: anVubmxpa2VzdGVh.com"
        );
    }
}
