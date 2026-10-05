use crate::{config::ServerConfig, model::Inventory};
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

#[derive(Clone)]
pub struct CompanionClient {
    client: Client,
}

impl CompanionClient {
    pub fn new(timeout: Duration) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(timeout)
                .user_agent(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION")
                ))
                .build()?,
        })
    }

    pub async fn inventory(&self, server: &ServerConfig) -> Result<Inventory> {
        let token = tokio::fs::read_to_string(&server.token_file)
            .await
            .with_context(|| format!("read token file {}", server.token_file.display()))?;
        anyhow::ensure!(!token.trim().is_empty(), "token file is empty");
        let endpoint = server
            .url
            .join("v1/inventory")
            .context("build inventory endpoint")?;
        let response = self
            .client
            .get(endpoint)
            .bearer_auth(token.trim())
            .send()
            .await
            .context("request inventory")?
            .error_for_status()
            .context("inventory HTTP status")?;
        let inventory: Inventory = response.json().await.context("decode inventory response")?;
        inventory.validate().map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            inventory.server.id == server.id,
            "server identity mismatch: expected {}, received {}",
            server.id,
            inventory.server.id
        );
        Ok(inventory)
    }
}
