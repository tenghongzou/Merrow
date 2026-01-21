use merrow::models::{Account, Position};

#[test]
fn position_rejects_negative_quantity() {
    let position = Position::new("BTCUSDT", -1.0, 100.0);
    assert!(position.is_err());
}

#[test]
fn account_rejects_negative_cash() {
    let account = Account::new(-1.0, Vec::new());
    assert!(account.is_err());
}
