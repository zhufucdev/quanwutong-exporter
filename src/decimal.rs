use std::{
    fmt::Debug, ops::DerefMut, sync::atomic::{AtomicPtr, Ordering}
};

use axum::routing::get;
use prometheus_client::{
    encoding::{self, EncodeGaugeValue, EncodeMetric},
    metrics::{MetricType, gauge::Atomic},
};
use rust_decimal::{Decimal, dec};

pub struct DecimalWrapper(Decimal);

pub struct AtomicDecimal {
    value: AtomicPtr<Decimal>,
}

impl Default for AtomicDecimal {
    fn default() -> Self {
        Self {
            value: AtomicPtr::new(Box::leak(Box::new(Default::default()))),
        }
    }
}

impl Debug for AtomicDecimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtomicDecimal")
            .field(
                "value",
                &prometheus_client::metrics::gauge::Atomic::<Decimal>::get(self),
            )
            .finish()
    }
}

impl Debug for DecimalWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl EncodeGaugeValue for DecimalWrapper {
    fn encode(&self, encoder: &mut encoding::GaugeValueEncoder) -> Result<(), std::fmt::Error> {
        encoding::helper::gauge_encode_str(encoder, self.0.to_string())
    }
}

impl EncodeMetric for DecimalWrapper {
    fn encode(
        &self,
        mut encoder: prometheus_client::encoding::MetricEncoder,
    ) -> Result<(), std::fmt::Error> {
        encoder.encode_info(&[("", Option::<String>::None)])
    }

    fn metric_type(&self) -> MetricType {
        MetricType::Gauge
    }
}

impl prometheus_client::metrics::gauge::Atomic<DecimalWrapper> for AtomicDecimal {
    fn inc(&self) -> DecimalWrapper {
        DecimalWrapper(self.inc_by(dec!(1)))
    }

    fn inc_by(&self, v: DecimalWrapper) -> DecimalWrapper {
        DecimalWrapper(self.inc_by(v.0))
    }

    fn dec(&self) -> DecimalWrapper {
        DecimalWrapper(self.dec())
    }

    fn dec_by(&self, v: DecimalWrapper) -> DecimalWrapper {
        DecimalWrapper(self.dec_by(v.0))
    }

    fn set(&self, v: DecimalWrapper) -> DecimalWrapper {
        DecimalWrapper(self.set(v.0))
    }

    fn get(&self) -> DecimalWrapper {
        DecimalWrapper(self.get())
    }
}

impl prometheus_client::metrics::gauge::Atomic<Decimal> for AtomicDecimal {
    fn inc(&self) -> Decimal {
        self.inc_by(dec!(1))
    }

    fn inc_by(&self, v: Decimal) -> Decimal {
        let value = unsafe { Box::from_raw(self.value.load(Ordering::Acquire)) };
        let new_value = value.as_ref() + v;
        self.value.store(
            Box::leak(Box::new(new_value)) as *mut Decimal,
            Ordering::Release,
        );
        new_value
    }

    fn dec(&self) -> Decimal {
        self.inc_by(dec!(-1))
    }

    fn dec_by(&self, v: Decimal) -> Decimal {
        self.inc_by(-v)
    }

    fn set(&self, v: Decimal) -> Decimal {
        drop(unsafe { Box::from_raw(self.value.load(Ordering::Acquire)) });
        self.value.store(Box::leak(Box::new(v)), Ordering::Relaxed);
        v
    }

    fn get(&self) -> Decimal {
        unsafe { *self.value.load(Ordering::Relaxed) }
    }
}

impl From<Decimal> for DecimalWrapper {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}
