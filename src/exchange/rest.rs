use crate::exchange::{CandleRequest, Exchange};
use crate::models::{Balance, Candle, OrderAck, OrderRequest, Position};
use crate::{Error, Result};
use reqwest::blocking::Client;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct RestExchangeConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub passphrase: Option<String>,
    pub timeout_secs: u64,
}

pub struct RestExchange {
    client: Client,
    config: RestExchangeConfig,
}

impl RestExchange {
    pub fn new(config: RestExchangeConfig) -> Result<Self> {
        if config.base_url.trim().is_empty() {
            return Err(Error::new("base_url must be set"));
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs.max(1)))
            .build()
            .map_err(|err| Error::new(format!("http client build failed: {err}")))?;
        Ok(Self { client, config })
    }

    fn ensure_auth(&self) -> Result<()> {
        if self.config.api_key.as_deref().unwrap_or("").is_empty() {
            return Err(Error::new("api_key must be set"));
        }
        if self.config.api_secret.as_deref().unwrap_or("").is_empty() {
            return Err(Error::new("api_secret must be set"));
        }
        Ok(())
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl Exchange for RestExchange {
    fn place_order(&self, _order: &OrderRequest) -> Result<OrderAck> {
        self.ensure_auth()?;
        Err(Error::new("place_order not implemented"))
    }

    fn cancel_order(&self, _order_id: &str) -> Result<()> {
        self.ensure_auth()?;
        Err(Error::new("cancel_order not implemented"))
    }

    fn fetch_balances(&self) -> Result<Vec<Balance>> {
        self.ensure_auth()?;
        Err(Error::new("fetch_balances not implemented"))
    }

    fn fetch_positions(&self) -> Result<Vec<Position>> {
        self.ensure_auth()?;
        Err(Error::new("fetch_positions not implemented"))
    }

    fn fetch_open_orders(&self) -> Result<Vec<OrderAck>> {
        self.ensure_auth()?;
        Err(Error::new("fetch_open_orders not implemented"))
    }

    fn fetch_candles(&self, _req: &CandleRequest) -> Result<Vec<Candle>> {
        Err(Error::new("fetch_candles not implemented"))
    }
}
