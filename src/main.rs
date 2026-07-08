use std::sync::{Arc, atomic::AtomicU32};

use axum::{
    body::Body,
    extract::State,
    http::HeaderName,
    response::{IntoResponse, Response},
    routing::get,
};
use prometheus_client::{
    encoding::EncodeLabelSet,
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};
use reqwest::StatusCode;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::{
    decimal::{AtomicDecimal, DecimalWrapper},
    response::QwtResponse,
};

mod decimal;
mod environ;
mod response;

struct AppState {
    client: reqwest::Client,
    token: String,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let token = environ::get_env_var("TOKEN")
        .await
        .expect("Unable to get token");
    let state = AppState {
        client: reqwest::Client::new(),
        token,
    };

    let bind = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("[::]:8080".into());
    println!("Listening on {bind}");

    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .expect("Bind address failed");
    let app = axum::Router::new()
        .route("/", get(root_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(Arc::new(state));
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> &'static str {
    concat!("quanwutong-exporter ", std::env!("CARGO_PKG_VERSION"))
}

async fn metrics_handler(State(state): State<Arc<AppState>>) -> Result<QwtMetrics, ScrapError> {
    const USER_AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 MicroMessenger/7.0.20.1781(0x6700143B) NetType/WIFI MiniProgramEnv/Mac MacWechat/WMPF MacWechat/3.8.7(0x13080712) UnifiedPCMacWechat(0xf2641a50) XWEB/19978";

    let response = state
        .client
        .get("https://qft.quanfangtongvip.com/api/wechat/smart/device/detail?deviceType=2")
        .header("Token", state.token.clone())
        .header("User-Agent", USER_AGENT)
        .header("Platform", "MiniApp")
        .header("Model", "iPhone18,4")
        .send()
        .await?
        .error_for_status()?;
    let body = response.bytes().await?;
    let json: QwtResponse = serde_json::from_slice(&body)?;

    Ok(qwt_to_metrics(&json))
}

fn qwt_to_metrics(qwt: &QwtResponse) -> QwtMetrics {
    #[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelSet)]
    struct Labels {
        device_id: i32,
    }
    let mut registry = <Registry>::default();
    let balance = Family::<Labels, Gauge<DecimalWrapper, AtomicDecimal>>::default();
    let instant_value = Family::<Labels, Gauge<DecimalWrapper, AtomicDecimal>>::default();
    let price = Family::<Labels, Gauge<DecimalWrapper, AtomicDecimal>>::default();
    for device in qwt.data.iter() {
        let labels = Labels {
            device_id: device.device_id,
        };
        balance.get_or_create(&labels).set(device.balance.into());
        instant_value
            .get_or_create(&labels)
            .set(device.instant_value.into());
        price.get_or_create(&labels).set(device.price.into());
    }
    registry.register("balance_cny", "Balance of the room", balance);
    registry.register("instant_value_kwh", "Current meter read", instant_value);
    registry.register("price_cny", "Price per kWh", price);

    QwtMetrics { registry }
}

struct QwtMetrics {
    registry: Registry,
}

#[derive(Debug, Error)]
enum ScrapError {
    #[error("io: {0}")]
    IO(#[from] reqwest::Error),
    #[error("JSON parsing: {0}")]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for ScrapError {
    fn into_response(self) -> Response<Body> {
        match self {
            ScrapError::IO(error) => (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("Unstream service failed: {error}"),
            )
                .into_response(),
            ScrapError::Json(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("JSON parsing failed: {error}"),
            )
                .into_response(),
        }
    }
}

impl IntoResponse for QwtMetrics {
    fn into_response(self) -> Response<Body> {
        let mut buffer = String::new();
        match prometheus_client::encoding::text::encode(&mut buffer, &self.registry) {
            Ok(spec) => Response::builder()
                .header(
                    "Content-Type",
                    "application/openmetrics-text; version=1.0.0",
                )
                .body(buffer.into())
                .unwrap(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }
}
