use crate::models::Candle;
use crate::Result;

pub trait MarketDataProvider {
    fn load_candles(&self, symbol: &str, interval: &str) -> Result<Vec<Candle>>;
}
