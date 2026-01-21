use merrow::exchange::binance::BinanceExchange;

#[test]
fn hmac_sha256_hex_matches_known_vector() {
    let secret = "key";
    let message = "The quick brown fox jumps over the lazy dog";
    let signature = BinanceExchange::hmac_sha256_hex(secret, message).expect("sign");
    assert_eq!(
        signature,
        "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
    );
}
