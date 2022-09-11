use crate::config::Config;
use clap::Parser;
use common::models::TodoResponse;
use common::{configure_tracing, AmqpHeaderCarrier};
use futures_util::StreamExt;
use lapin::message::Delivery;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions,
    QueueDeclareOptions,
};
use lapin::types::FieldTable;
use lapin::{Channel, Connection, ConnectionProperties, ExchangeKind};
use opentelemetry::global;
use tracing::{info, instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    configure_tracing("info".to_owned(), "service2".to_owned());

    let config: Config = Config::parse();

    let connection_properties = ConnectionProperties::default();
    let amqp_connection = Connection::connect(&*config.amqp_uri, connection_properties).await?;
    let channel = amqp_connection.create_channel().await?;

    channel
        .exchange_declare(
            &*config.notification_exchange,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_declare(
            &*config.notification_queue,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            &*config.notification_queue,
            &*config.notification_exchange,
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    consume(&channel, config).await?;

    global::shutdown_tracer_provider();

    Ok(())
}

#[instrument]
async fn consume(channel: &Channel, config: Config) -> anyhow::Result<()> {
    let mut consumer = channel
        .basic_consume(
            &*config.notification_queue,
            "consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        let delivery = correlate_trace_from_delivery(delivery);
        process_delivery(&delivery).await?;
        delivery.ack(BasicAckOptions::default()).await?
    }

    Ok(())
}

fn correlate_trace_from_delivery(delivery: Delivery) -> Delivery {
    let span = Span::current();

    let headers = &delivery
        .properties
        .headers()
        .clone()
        .unwrap_or_default()
        .inner()
        .clone();
    let parent_cx = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&AmqpHeaderCarrier::new(headers))
    });

    span.set_parent(parent_cx);

    delivery
}

#[instrument]
async fn process_delivery(delivery: &Delivery) -> anyhow::Result<()> {
    let todo: TodoResponse = serde_json::from_slice(&delivery.data)?;
    info!("Received todo {:?}", todo);
    Ok(())
}
