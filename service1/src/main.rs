use crate::config::Config;
use crate::handlers::create_todo;
use crate::state::State;
use actix_web::middleware::{Compress, NormalizePath};
use actix_web::{web, App, HttpServer};
use clap::Parser;
use common::configure_tracing;
use lapin::options::ExchangeDeclareOptions;
use lapin::types::FieldTable;
use lapin::ExchangeKind;
use opentelemetry::global;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

mod config;
mod error;
mod handlers;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    configure_tracing("info".to_owned(), "service1".to_owned());

    let config: Config = Config::parse();
    let state = Arc::new(State::new(config).await?);

    let channel = state.amqp_connection.create_channel().await?;
    channel
        .exchange_declare(
            &*state.config.notification_exchange,
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(NormalizePath::trim())
            .wrap(Compress::default())
            .wrap(TracingLogger::default())
            .service(
                web::scope("/v1")
                    .service(web::resource("/todo").route(web::post().to(create_todo))),
            )
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await?;

    global::shutdown_tracer_provider();

    Ok(())
}
