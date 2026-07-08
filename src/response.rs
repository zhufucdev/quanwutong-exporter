use rust_decimal::Decimal;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct QwtResponse {
    pub code: i32,
    pub msg: String,
    pub data: Vec<QwtDevice>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct QwtDevice {
    pub room_id: i32,
    pub device_id: i32,
    pub create_time: String,
    pub update_time: String,
    pub balance: Decimal,
    pub instant_value: Decimal,
    pub price: Decimal,
}
