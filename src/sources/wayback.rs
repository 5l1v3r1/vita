use crate::IntoSubdomain;
use serde_json::value::Value;
use std::collections::HashSet;
use std::sync::Arc;
use std::{error::Error, fmt};
use url::Url;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct WaybackResult {
    data: Value,
}

impl WaybackResult {
    fn new(data: Value) -> Self {
        Self { data }
    }
}

//TODO: this could be cleaned up, to avoid creating the extra vec `vecs`
impl IntoSubdomain for WaybackResult {
    fn subdomains(&self) -> HashSet<String> {
        let arr = self.data.as_array().unwrap();
        let vecs: Vec<&str> = arr.iter().map(|s| s[0].as_str().unwrap()).collect();
        vecs.into_iter()
            .filter_map(|a| Url::parse(a).ok())
            .map(|u| u.host_str().unwrap().into())
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct WaybackError {
    host: Arc<String>,
}

impl WaybackError {
    fn new(host: Arc<String>) -> Self {
        Self { host: host }
    }
}

impl Error for WaybackError {}

impl fmt::Display for WaybackError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "WaybackMachine couldn't find any results for: {}",
            self.host
        )
    }
}

fn build_url(host: &str) -> String {
    format!(
        "https://web.archive.org/cdx/search/cdx?url=*.{}/*&output=json\
    &fl=original&collapse=urlkey&limit=100000",
        host
    )
}

pub async fn run(host: Arc<String>) -> Result<HashSet<String>> {
    let uri = build_url(&host);
    let resp: Option<Value> = surf::get(uri).recv_json().await?;

    match resp {
        Some(data) => {
            let subdomains = WaybackResult::new(data).subdomains();

            if subdomains.len() != 0 {
                Ok(subdomains)
            } else {
                Err(Box::new(WaybackError::new(host)))
            }
        }

        None => Err(Box::new(WaybackError::new(host))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_await_test::async_test;

    #[test]
    fn url_builder() {
        let correct_uri =
            "https://web.archive.org/cdx/search/cdx?url=*.hackerone.com/*&output=json\
    &fl=original&collapse=urlkey&limit=100000";
        assert_eq!(correct_uri, build_url("hackerone.com"));
    }

    // Checks to see if the run function returns subdomains
    //
    #[ignore] // hangs forever on windows for some reasons?
    #[async_test]
    async fn returns_results() {
        let host = Arc::new("hackerone.com".to_owned());
        let results = run(host).await.unwrap();
        assert!(results.len() > 0);
    }

    //Some("WaybackMachine couldn't find results for: anVubmxpa2VzdGVh.com")
    #[async_test]
    async fn handle_no_results() {
        let host = Arc::new("anVubmxpa2VzdGVh.com".to_string());
        let res = run(host).await;
        let e = res.unwrap_err();
        assert_eq!(
            e.to_string(),
            "WaybackMachine couldn't find any results for: anVubmxpa2VzdGVh.com"
        );
    }
}
