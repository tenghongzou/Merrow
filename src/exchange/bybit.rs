use crate::data::exchange_loader::{map_bybit_interval, parse_bybit_klines};
use crate::exchange::{CandleRequest, Exchange};
use crate::models::{Balance, Candle, OrderAck, OrderRequest, OrderStatus, OrderType, Position, Side};
use crate::{Error, Result};
use hmac::{Hmac, Mac};
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::{json, Value};
use sha2::Sha256;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug)]
pub struct BybitConfig {
    pub base_url: String,
    pub api_key: String,
    pub api_secret: String,
    pub recv_window: u64,
    pub timeout_secs: u64,
    pub category: String,
    pub account_type: String,
    pub default_symbol: Option<String>,
}

pub struct BybitExchange {
    client: Client,
    config: BybitConfig,
}

impl BybitExchange {
    pub fn new(config: BybitConfig) -> Result<Self> {
        if config.base_url.trim().is_empty() {
            return Err(Error::new("base_url must be set"));
        }
        if config.api_key.trim().is_empty() {
            return Err(Error::new("api_key must be set"));
        }
        if config.api_secret.trim().is_empty() {
            return Err(Error::new("api_secret must be set"));
        }
        if config.category.trim().is_empty() {
            return Err(Error::new("category must be set"));
        }
        if config.account_type.trim().is_empty() {
            return Err(Error::new("account_type must be set"));
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs.max(1)))
            .build()
            .map_err(|err| Error::new(format!("http client build failed: {err}")))?;
        Ok(Self { client, config })
    }

    fn timestamp_ms() -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::new("system time before unix epoch"))?;
        Ok(now.as_millis().to_string())
    }

    fn hmac_sha256_hex(secret: &str, payload: &str) -> Result<String> {
        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| Error::new("invalid key"))?;
        mac.update(payload.as_bytes());
        let result = mac.finalize().into_bytes();
        Ok(bytes_to_hex(&result))
    }

    fn signed_request(
        &self,
        method: Method,
        path: &str,
        params: Vec<(String, String)>,
        body: Option<Value>,
    ) -> Result<Value> {
        let timestamp = Self::timestamp_ms()?;
        let recv_window = self.config.recv_window.to_string();
        let query = build_query_string(&params);
        let body_str = if let Some(body) = body {
            serde_json::to_string(&body)
                .map_err(|err| Error::new(format!("json encode failed: {err}")))?
        } else {
            String::new()
        };
        let sign_payload = if method == Method::GET {
            format!(
                "{}{}{}{}",
                timestamp, self.config.api_key, recv_window, query
            )
        } else {
            format!(
                "{}{}{}{}",
                timestamp, self.config.api_key, recv_window, body_str
            )
        };
        let signature = Self::hmac_sha256_hex(&self.config.api_secret, &sign_payload)?;

        let url = if query.is_empty() {
            format!("{}{}", self.config.base_url, path)
        } else {
            format!("{}{}?{}", self.config.base_url, path, query)
        };

        let mut request = self
            .client
            .request(method, url)
            .header("X-BAPI-API-KEY", self.config.api_key.as_str())
            .header("X-BAPI-SIGN", signature)
            .header("X-BAPI-TIMESTAMP", timestamp)
            .header("X-BAPI-RECV-WINDOW", recv_window)
            .header("Content-Type", "application/json");

        if !body_str.is_empty() {
            request = request.body(body_str);
        }

        let response = request
            .send()
            .map_err(|err| Error::new(format!("http request failed: {err}")))?;

        if !response.status().is_success() {
            return Err(Error::new(format!(
                "bybit response status: {}",
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
                "bybit response status: {}",
                response.status()
            )));
        }
        response
            .json::<Value>()
            .map_err(|err| Error::new(format!("json parse failed: {err}")))
    }
}

impl Exchange for BybitExchange {
    fn place_order(&self, order: &OrderRequest) -> Result<OrderAck> {
        let mut body = json!({
            "category": self.config.category,
            "symbol": order.symbol.clone(),
            "side": side_label(&order.side),
            "orderType": order_type_label(&order.order_type),
            "qty": order.quantity.to_string(),
            "orderLinkId": order.client_order_id.clone(),
        });

        if let OrderType::Limit { price } = &order.order_type {
            if let Some(map) = body.as_object_mut() {
                map.insert("price".to_string(), Value::String(price.to_string()));
                map.insert("timeInForce".to_string(), Value::String("GTC".to_string()));
            }
        } else if let Some(map) = body.as_object_mut() {
            map.insert(
                "marketUnit".to_string(),
                Value::String("baseCoin".to_string()),
            );
        }

        let json = self.signed_request(Method::POST, "/v5/order/create", Vec::new(), Some(body))?;
        ensure_bybit_ok(&json)?;
        let exchange_order_id = json
            .get("result")
            .and_then(|value| value.get("orderId"))
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
            "category": self.config.category,
            "symbol": symbol,
            "orderLinkId": order_id,
        });
        let json = self.signed_request(Method::POST, "/v5/order/cancel", Vec::new(), Some(body))?;
        ensure_bybit_ok(&json)?;
        Ok(())
    }

    fn fetch_balances(&self) -> Result<Vec<Balance>> {
        let params = vec![("accountType".to_string(), self.config.account_type.clone())];
        let json = self.signed_request(
            Method::GET,
            "/v5/account/wallet-balance",
            params,
            None,
        )?;
        ensure_bybit_ok(&json)?;
        let list = json
            .get("result")
            .and_then(|value| value.get("list"))
            .and_then(|value| value.as_array())
            .ok_or_else(|| Error::new("bybit result.list missing"))?;

        let mut balances = Vec::new();
        for entry in list {
            if let Some(coins) = entry.get("coin").and_then(|value| value.as_array()) {
                for coin in coins {
                    let asset = coin
                        .get("coin")
                        .and_then(|value| value.as_str())
                        .unwrap_or("")
                        .to_string();
                    if asset.is_empty() {
                        continue;
                    }
                    let wallet_balance =
                        value_to_f64(coin.get("walletBalance").unwrap_or(&Value::String("0".to_string())))?;
                    let locked = value_to_f64(coin.get("locked").unwrap_or(&Value::String("0".to_string())))?;
                    let free = if let Some(value) = coin.get("availableToWithdraw") {
                        value_to_f64(value)?
                    } else if let Some(value) = coin.get("availableBalance") {
                        value_to_f64(value)?
                    } else if let Some(value) = coin.get("free") {
                        value_to_f64(value)?
                    } else {
                        (wallet_balance - locked).max(0.0)
                    };
                    let locked_final = if let Some(value) = coin.get("locked") {
                        value_to_f64(value)?
                    } else {
                        (wallet_balance - free).max(0.0)
                    };

                    balances.push(Balance {
                        asset,
                        free,
                        locked: locked_final,
                    });
                }
            }
        }
        Ok(balances)
    }

    fn fetch_positions(&self) -> Result<Vec<Position>> {
        Ok(Vec::new())
    }

    fn fetch_open_orders(&self) -> Result<Vec<OrderAck>> {
        let mut params = vec![("category".to_string(), self.config.category.clone())];
        if let Some(symbol) = self.config.default_symbol.as_ref() {
            params.push(("symbol".to_string(), symbol.clone()));
        }
        let json = self.signed_request(Method::GET, "/v5/order/realtime", params, None)?;
        ensure_bybit_ok(&json)?;
        let list = json
            .get("result")
            .and_then(|value| value.get("list"))
            .and_then(|value| value.as_array())
            .ok_or_else(|| Error::new("bybit result.list missing"))?;

        let mut orders = Vec::new();
        for item in list {
            let client_order_id = item
                .get("orderLinkId")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            if client_order_id.is_empty() {
                continue;
            }
            let exchange_order_id = item
                .get("orderId")
                .and_then(|value| value.as_str())
                .map(|value| value.to_string());
            let status = item
                .get("orderStatus")
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
        let interval = map_bybit_interval(&req.interval)?;
        let params = vec![
            ("category".to_string(), self.config.category.clone()),
            ("symbol".to_string(), req.symbol.clone()),
            ("interval".to_string(), interval),
            ("start".to_string(), req.start_time.to_string()),
            ("end".to_string(), req.end_time.to_string()),
        ];
        let json = self.public_request("/v5/market/kline", params)?;
        let parsed = parse_bybit_klines(
            &serde_json::to_string(&json).map_err(|err| Error::new(format!("json encode failed: {err}")))?,
        )?;
        Ok(parsed.candles)
    }
}

fn side_label(side: &Side) -> &'static str {
    match side {
        Side::Buy => "Buy",
        Side::Sell => "Sell",
    }
}

fn order_type_label(order_type: &OrderType) -> &'static str {
    match order_type {
        OrderType::Market => "Market",
        OrderType::Limit { .. } => "Limit",
    }
}

fn parse_status(status: &str) -> OrderStatus {
    match status {
        "New" => OrderStatus::New,
        "PartiallyFilled" => OrderStatus::PartiallyFilled,
        "Filled" => OrderStatus::Filled,
        "Cancelled" | "Canceled" => OrderStatus::Canceled,
        "Rejected" => OrderStatus::Rejected,
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

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{:02x}", byte));
    }
    output
}

fn ensure_bybit_ok(json: &Value) -> Result<()> {
    let ret_code = json
        .get("retCode")
        .and_then(|value| value.as_i64())
        .unwrap_or(-1);
    if ret_code != 0 {
        let msg = json
            .get("retMsg")
            .and_then(|value| value.as_str())
            .unwrap_or("bybit retCode is not 0");
        return Err(Error::new(format!("bybit error: {msg}")));
    }
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
