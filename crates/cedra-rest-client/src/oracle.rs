use crate::{Result};
use reqwest::{Client as ReqwestClient, Response, Url};
use std::time::Duration;

pub struct OracleClient {
    oracle_url: Url,
    inner: ReqwestClient,
}

impl OracleClient {
    pub fn new(oracle_url: Url) -> Self {
        Self {
            oracle_url,
            inner: ReqwestClient::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    pub async fn get_price_list(&self) -> Result<Response> {
        let mut url = self.oracle_url.clone();
        url.set_path("price-feed");

        let response = self.build_and_submit_get_request(url).await?;
        let status_code = response.status();

        if !status_code.is_success() {
            return Err(anyhow::anyhow!("status code: {:?}", status_code.to_string()));
        }

        Ok(response)
    }

    pub fn new_from_rest_client(oracle_url: Url) -> Self {
        Self {
            oracle_url,
            inner: ReqwestClient::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
        }
    }

    // Helper to carry out requests.
    async fn build_and_submit_get_request(&self, url: Url) -> Result<Response> {
        // build request
        let request = self.inner.get(url).header("content-length", 0);
  
        // carry out and return response
        let response = request.send().await.map_err(|e| anyhow::anyhow!("error: {:?}", e) )?;
        Ok(response)
    }
}