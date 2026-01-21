use crate::exchange::{CandleRequest, Exchange};
use crate::models::{Balance, Candle, OrderAck, OrderRequest, OrderStatus, OrderType, Position, Side};
use crate::{Error, Result};
use hmac::{Hmac, Mac};
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::Value;
use sha2::Sha256;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug)]
pub struct BinanceConfig {
    pub base_url: String,
    pub api_key: String,
    pub api_secret: String,
    pub recv_window: u64,
    pub timeout_secs: u64,
    pub default_symbol: Option<String>,
}

pub struct BinanceExchange {
    client: Client,
    config: BinanceConfig,
}

impl BinanceExchange {
    pub fn new(config: BinanceConfig) -> Result<Self> {
        if config.base_url.trim().is_empty() {
            return Err(Error::new("base_url must be set"));
        }
        if config.api_key.trim().is_empty() {
            return Err(Error::new("api_key must be set"));
        }
        if config.api_secret.trim().is_empty() {
            return Err(Error::new("api_secret must be set"));
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs.max(1)))
            .build()
            .map_err(|err| Error::new(format!("http client build failed: {err}")))?;
        Ok(Self { client, config })
    }

    fn timestamp_ms() -> Result<u64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::new("system time before unix epoch"))?;
        Ok(now.as_millis() as u64)
    }

    pub fn hmac_sha256_hex(secret: &str, message: &str) -> Result<String> {
        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| Error::new("invalid key"))?;
        mac.update(message.as_bytes());
        let result = mac.finalize().into_bytes();
        Ok(bytes_to_hex(&result))
    }

    fn signed_request(
        &self,
        method: Method,
        path: &str,
        mut params: Vec<(String, String)>,
    ) -> Result<Value> {
        let timestamp = Self::timestamp_ms()?;
        params.push(("timestamp".to_string(), timestamp.to_string()));
        if self.config.recv_window > 0 {
            params.push(("recvWindow".to_string(), self.config.recv_window.to_string()));
        }
        let query = build_query_string(&params);
        let signature = Self::hmac_sha256_hex(&self.config.api_secret, &query)?;
        let signed_query = format!("{query}&signature={signature}");
        let url = format!("{}{}?{}", self.config.base_url, path, signed_query);

        let response = self
            .client
            .request(method, url)
            .header("X-MBX-APIKEY", self.config.api_key.as_str())
            .send()
            .map_err(|err| Error::new(format!("http request failed: {err}")))?;

        if !response.status().is_success() {
            return Err(Error::new(format!(
                "binance response status: {}",
                response.status()
            )));
        }

        response
            .json::<Value>()
            .map_err(|err| Error::new(format!("json parse failed: {err}")))
    }

    fn public_request(&self, path: &str, params: Vec<(String, String)>) -> Result<Value> {
        let query = build_query_string(&params);
        let url = if query.is_empty() {
            format!("{}{}", self.config.base_url, path)
        } else {
            format!("{}{}?{}", self.config.base_url, path, query)
        };
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|err| Error::new(format!("http request failed: {err}")))?;
        if !response.status().is_success() {
            return Err(Error::new(format!(
                "binance response status: {}",
                response.status()
            )));
        }
        response
            .json::<Value>()
            .map_err(|err| Error::new(format!("json parse failed: {err}")))
    }
}

impl Exchange for BinanceExchange {
    fn place_order(&self, order: &OrderRequest) -> Result<OrderAck> {
        let mut params = vec![
            ("symbol".to_string(), order.symbol.clone()),
            ("side".to_string(), side_label(&order.side).to_string()),
        ];

        match order.order_type {
            OrderType::Market => {
                params.push(("type".to_string(), "MARKET".to_string()));
                params.push(("quantity".to_string(), order.quantity.to_string()));
            }
            OrderType::Limit { price } => {
                params.push(("type".to_string(), "LIMIT".to_string()));
                params.push(("timeInForce".to_string(), "GTC".to_string()));
                params.push(("quantity".to_string(), order.quantity.to_string()));
                params.push(("price".to_string(), price.to_string()));
            }
        }

        params.push(("newClientOrderId".to_string(), order.client_order_id.clone()));
        params.push(("newOrderRespType".to_string(), "ACK".to_string()));

        let json = self.signed_request(Method::POST, "/api/v3/order", params)?;
        let exchange_order_id = json
            .get("orderId")
            .and_then(|value| value.as_i64())
            .map(|id| id.to_string());

        Ok(OrderAck {
            client_order_id: order.client_order_id.clone(),
            exchange_order_id,
            status: OrderStatus::New,
        })
    }

    fn cancel_order(&self, order_id: &str) -> Result<()> {
        let symbol = self
            .config
            .default_symbol
            .as_ref()
            .ok_or_else(|| Error::new("default_symbol must be set for cancel_order"))?;
        let params = vec![
            ("symbol".to_string(), symbol.clone()),
            ("origClientOrderId".to_string(), order_id.to_string()),
        ];
        let _ = self.signed_request(Method::DELETE, "/api/v3/order", params)?;
        Ok(())
    }

    fn fetch_balances(&self) -> Result<Vec<Balance>> {
        let json = self.signed_request(Method::GET, "/api/v3/account", Vec::new())?;
        let balances = json
            .get("balances")
            .and_then(|value| value.as_array())
            .ok_or_else(|| Error::new("balances missing"))?;
        let mut result = Vec::new();
        for balance in balances {
            let asset = balance
                .get("asset")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            if asset.is_empty() {
                continue;
            }
            let free = balance
                .get("free")
                .and_then(|value| value.as_str())
                .unwrap_or("0")
                .parse::<f64>()
                .unwrap_or(0.0);
            let locked = balance
                .get("locked")
                .and_then(|value| value.as_str())
                .unwrap_or("0")
                .parse::<f64>()
                .unwrap_or(0.0);
            result.push(Balance { asset, free, locked });
        }
        Ok(result)
    }

    fn fetch_positions(&self) -> Result<Vec<Position>> {
        Ok(Vec::new())
    }

    fn fetch_open_orders(&self) -> Result<Vec<OrderAck>> {
        let json = self.signed_request(Method::GET, "/api/v3/openOrders", Vec::new())?;
        let array = json
            .as_array()
            .ok_or_else(|| Error::new("openOrders should be array"))?;
        let mut result = Vec::new();
        for item in array {
            let client_order_id = item
                .get("clientOrderId")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            let exchange_order_id = item
                .get("orderId")
                .and_then(|value| value.as_i64())
                .map(|id| id.to_string());
            let status = item
                .get("status")
                .and_then(|value| value.as_str())
                .map(parse_status)
                .unwrap_or(OrderStatus::New);
            if client_order_id.is_empty() {
                continue;
            }
            result.push(OrderAck {
                client_order_id,
                exchange_order_id,
                status,
            });
        }
        Ok(result)
    }

    fn fetch_candles(&self, req: &CandleRequest) -> Result<Vec<Candle>> {
        let params = vec![
            ("symbol".to_string(), req.symbol.clone()),
            ("interval".to_string(), req.interval.clone()),
            ("startTime".to_string(), req.start_time.to_string()),
            ("endTime".to_string(), req.end_time.to_string()),
        ];
        let json = self.public_request("/api/v3/klines", params)?;
        let array = json
            .as_array()
            .ok_or_else(|| Error::new("klines response should be array"))?;
        let mut result = Vec::with_capacity(array.len());
        for row in array {
            let row = row
                .as_array()
                .ok_or_else(|| Error::new("kline row is not array"))?;
            if row.len() < 7 {
                return Err(Error::new("kline row has insufficient fields"));
            }
            let close_time_ms = value_to_i64(&row[6])?;
            let open = value_to_f64(&row[1])?;
            let high = value_to_f64(&row[2])?;
            let low = value_to_f64(&row[3])?;
            let close = value_to_f64(&row[4])?;
            let volume = value_to_f64(&row[5])?;
            result.push(Candle {
                time: close_time_ms / 1000,
                open,
                high,
                low,
                close,
                volume,
            });
        }
        Ok(result)
    }
}

fn build_query_string(params: &[(String, String)]) -> String {
    params
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<String>>()
        .join("&")
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{:02x}", byte));
    }
    output
}

fn side_label(side: &Side) -> &'static str {
    match side {
        Side::Buy => "BUY",
        Side::Sell => "SELL",
    }
}

fn parse_status(status: &str) -> OrderStatus {
    match status {
        "NEW" => OrderStatus::New,
        "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
        "FILLED" => OrderStatus::Filled,
        "CANCELED" | "CANCELLED" => OrderStatus::Canceled,
        "REJECTED" | "EXPIRED" => OrderStatus::Rejected,
        _ => OrderStatus::New,
    }
}

fn value_to_i64(value: &Value) -> Result<i64> {
    match value {
        Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| Error::new("number is not i64")),
        Value::String(text) => text
            .parse::<i64>()
            .map_err(|err| Error::new(format!("invalid i64: {err}"))),
        _ => Err(Error::new("unexpected value type for i64")),
    }
}

fn value_to_f64(value: &Value) -> Result<f64> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .ok_or_else(|| Error::new("number is not f64")),
        Value::String(text) => text
            .parse::<f64>()
            .map_err(|err| Error::new(format!("invalid f64: {err}"))),
        _ => Err(Error::new("unexpected value type for f64")),
    }
}
