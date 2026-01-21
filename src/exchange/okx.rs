use crate::data::exchange_loader::{map_okx_interval, parse_okx_candles};
use crate::exchange::{CandleRequest, Exchange};
use crate::models::{Balance, Candle, OrderAck, OrderRequest, OrderStatus, OrderType, Position, Side};
use crate::{Error, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::{SecondsFormat, Utc};
use hmac::{Hmac, Mac};
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::{json, Value};
use sha2::Sha256;
use std::time::Duration;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug)]
pub struct OkxConfig {
    pub base_url: String,
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: String,
    pub timeout_secs: u64,
    pub default_symbol: Option<String>,
}

pub struct OkxExchange {
    client: Client,
    config: OkxConfig,
}

impl OkxExchange {
    pub fn new(config: OkxConfig) -> Result<Self> {
        if config.base_url.trim().is_empty() {
            return Err(Error::new("base_url must be set"));
        }
        if config.api_key.trim().is_empty() {
            return Err(Error::new("api_key must be set"));
        }
        if config.api_secret.trim().is_empty() {
            return Err(Error::new("api_secret must be set"));
        }
        if config.passphrase.trim().is_empty() {
            return Err(Error::new("passphrase must be set"));
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs.max(1)))
            .build()
            .map_err(|err| Error::new(format!("http client build failed: {err}")))?;
        Ok(Self { client, config })
    }

    fn timestamp() -> String {
        Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
    }

    fn sign(&self, timestamp: &str, method: &str, request_path: &str, body: &str) -> Result<String> {
        let prehash = format!("{timestamp}{method}{request_path}{body}");
        let mut mac =
            HmacSha256::new_from_slice(self.config.api_secret.as_bytes())
                .map_err(|_| Error::new("invalid key"))?;
        mac.update(prehash.as_bytes());
        let result = mac.finalize().into_bytes();
        Ok(STANDARD.encode(result))
    }

    fn signed_request(
        &self,
        method: Method,
        path: &str,
        params: Vec<(String, String)>,
        body: Option<Value>,
    ) -> Result<Value> {
        let timestamp = Self::timestamp();
        let query = build_query_string(&params);
        let request_path = if query.is_empty() {
            path.to_string()
        } else {
            format!("{path}?{query}")
        };
        let body_str = if let Some(body) = body {
            serde_json::to_string(&body)
                .map_err(|err| Error::new(format!("json encode failed: {err}")))?
        } else {
            String::new()
        };
        let sign = self.sign(&timestamp, method.as_str(), &request_path, &body_str)?;

        let url = format!("{}{}", self.config.base_url, request_path);
        let mut request = self
            .client
            .request(method, url)
            .header("OK-ACCESS-KEY", self.config.api_key.as_str())
            .header("OK-ACCESS-SIGN", sign)
            .header("OK-ACCESS-TIMESTAMP", timestamp)
            .header("OK-ACCESS-PASSPHRASE", self.config.passphrase.as_str())
            .header("Content-Type", "application/json");

        if !body_str.is_empty() {
            request = request.body(body_str);
        }

        let response = request
            .send()
            .map_err(|err| Error::new(format!("http request failed: {err}")))?;
        if !response.status().is_success() {
            return Err(Error::new(format!(
                "okx response status: {}",
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
                "okx response status: {}",
                response.status()
            )));
        }
        response
            .json::<Value>()
            .map_err(|err| Error::new(format!("json parse failed: {err}")))
    }
}

impl Exchange for OkxExchange {
    fn place_order(&self, order: &OrderRequest) -> Result<OrderAck> {
        let mut body = json!({
            "instId": order.symbol.clone(),
            "tdMode": "cash",
            "side": side_label(&order.side),
            "ordType": order_type_label(&order.order_type),
            "sz": order.quantity.to_string(),
            "clOrdId": order.client_order_id.clone(),
        });

        if let OrderType::Limit { price } = &order.order_type {
            if let Some(map) = body.as_object_mut() {
                map.insert("px".to_string(), Value::String(price.to_string()));
            }
        }

        let json = self.signed_request(Method::POST, "/api/v5/trade/order", Vec::new(), Some(body))?;
        ensure_okx_ok(&json)?;
        let exchange_order_id = json
            .get("data")
            .and_then(|value| value.as_array())
            .and_then(|array| array.first())
            .and_then(|value| value.get("ordId"))
            .and_then(|value| value.as_str())
            .map(|value| value.to_string());

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
        let body = json!({
            "instId": symbol,
            "clOrdId": order_id,
        });
        let json =
            self.signed_request(Method::POST, "/api/v5/trade/cancel-order", Vec::new(), Some(body))?;
        ensure_okx_ok(&json)?;
        Ok(())
    }

    fn fetch_balances(&self) -> Result<Vec<Balance>> {
        let json = self.signed_request(Method::GET, "/api/v5/account/balance", Vec::new(), None)?;
        ensure_okx_ok(&json)?;
        let data = json
            .get("data")
            .and_then(|value| value.as_array())
            .ok_or_else(|| Error::new("okx data missing"))?;

        let mut balances = Vec::new();
        for entry in data {
            if let Some(details) = entry.get("details").and_then(|value| value.as_array()) {
                for coin in details {
                    push_okx_balance(&mut balances, coin)?;
                }
            } else {
                push_okx_balance(&mut balances, entry)?;
            }
        }
        Ok(balances)
    }

    fn fetch_positions(&self) -> Result<Vec<Position>> {
        Ok(Vec::new())
    }

    fn fetch_open_orders(&self) -> Result<Vec<OrderAck>> {
        let mut params = vec![("instType".to_string(), "SPOT".to_string())];
        if let Some(symbol) = self.config.default_symbol.as_ref() {
            params.push(("instId".to_string(), symbol.clone()));
        }
        let json =
            self.signed_request(Method::GET, "/api/v5/trade/orders-pending", params, None)?;
        ensure_okx_ok(&json)?;
        let list = json
            .get("data")
            .and_then(|value| value.as_array())
            .ok_or_else(|| Error::new("okx data missing"))?;

        let mut orders = Vec::new();
        for item in list {
            let client_order_id = item
                .get("clOrdId")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            if client_order_id.is_empty() {
                continue;
            }
            let exchange_order_id = item
                .get("ordId")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
            let status = item
                .get("state")
                .and_then(|value| value.as_str())
                .map(parse_status)
                .unwrap_or(OrderStatus::New);

            orders.push(OrderAck {
                client_order_id,
                exchange_order_id,
                status,
            });
        }
        Ok(orders)
    }

    fn fetch_candles(&self, req: &CandleRequest) -> Result<Vec<Candle>> {
        let bar = map_okx_interval(&req.interval)?;
        let params = vec![
            ("instId".to_string(), req.symbol.clone()),
            ("bar".to_string(), bar),
            ("after".to_string(), req.end_time.to_string()),
            ("before".to_string(), req.start_time.to_string()),
        ];
        let json = self.public_request("/api/v5/market/candles", params)?;
        let parsed = parse_okx_candles(
            &serde_json::to_string(&json).map_err(|err| Error::new(format!("json encode failed: {err}")))?,
        )?;
        Ok(parsed.candles)
    }
}

fn side_label(side: &Side) -> &'static str {
    match side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}

fn order_type_label(order_type: &OrderType) -> &'static str {
    match order_type {
        OrderType::Market => "market",
        OrderType::Limit { .. } => "limit",
    }
}

fn parse_status(status: &str) -> OrderStatus {
    match status {
        "live" => OrderStatus::New,
        "partially_filled" => OrderStatus::PartiallyFilled,
        "filled" => OrderStatus::Filled,
        "canceled" => OrderStatus::Canceled,
        "rejected" => OrderStatus::Rejected,
        _ => OrderStatus::New,
    }
}

fn build_query_string(params: &[(String, String)]) -> String {
    params
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<String>>()
        .join("&")
}

fn ensure_okx_ok(json: &Value) -> Result<()> {
    let code = json
        .get("code")
        .and_then(|value| value.as_str())
        .unwrap_or("1");
    if code != "0" {
        let msg = json
            .get("msg")
            .and_then(|value| value.as_str())
            .unwrap_or("okx code is not 0");
        return Err(Error::new(format!("okx error: {msg}")));
    }
    Ok(())
}

fn push_okx_balance(balances: &mut Vec<Balance>, coin: &Value) -> Result<()> {
    let asset = coin
        .get("ccy")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();
    if asset.is_empty() {
        return Ok(());
    }
    let free = if let Some(value) = coin.get("availBal") {
        value_to_f64(value)?
    } else {
        value_to_f64(coin.get("bal").unwrap_or(&Value::String("0".to_string())))?
    };
    let locked = if let Some(value) = coin.get("frozenBal") {
        value_to_f64(value)?
    } else {
        0.0
    };
    balances.push(Balance { asset, free, locked });
    Ok(())
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
