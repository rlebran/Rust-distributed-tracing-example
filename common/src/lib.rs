use lapin::types::{AMQPValue, ShortString};
use opentelemetry::propagation::{Extractor, Injector};
use std::collections::BTreeMap;
use tracing::{error, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub mod models;

pub fn configure_tracing(level: String, service_name: String) {
    opentelemetry::global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());

    let tracer = opentelemetry_zipkin::new_pipeline()
        .with_service_name(service_name)
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("unable to install zipkin tracer");

    let tracer = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = tracing_subscriber::fmt::layer().json();

    let level = EnvFilter::new(level);

    tracing_subscriber::registry()
        .with(subscriber)
        .with(level)
        .with(tracer)
        .init();
}

pub struct AmqpClientCarrier<'a> {
    properties: &'a mut BTreeMap<ShortString, AMQPValue>,
}

impl<'a> AmqpClientCarrier<'a> {
    pub fn new(properties: &'a mut BTreeMap<ShortString, AMQPValue>) -> Self {
        Self { properties }
    }
}

impl<'a> Injector for AmqpClientCarrier<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.properties
            .insert(key.into(), AMQPValue::LongString(value.into()));
    }
}

pub struct AmqpHeaderCarrier<'a> {
    headers: &'a BTreeMap<ShortString, AMQPValue>,
}

impl<'a> AmqpHeaderCarrier<'a> {
    pub fn new(headers: &'a BTreeMap<ShortString, AMQPValue>) -> Self {
        Self { headers }
    }
}

impl<'a> Extractor for AmqpHeaderCarrier<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|header_value| {
            if let AMQPValue::LongString(header_value) = header_value {
                std::str::from_utf8(header_value.as_bytes())
                    .map_err(|e| error!("Error decoding header value {:?}", e))
                    .ok()
            } else {
                warn!("Missing amqp tracing context propagation");
                None
            }
        })
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|header| header.as_str()).collect()
    }
}
