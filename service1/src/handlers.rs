use crate::error::Error;
use crate::{FieldTable, State};
use actix_web::{web, HttpResponse};
use common::models::TodoResponse;
use common::AmqpClientCarrier;
use lapin::options::BasicPublishOptions;
use lapin::{BasicProperties, Channel};
use serde::Deserialize;
use sqlx::{Postgres, Transaction};
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::{instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[derive(Debug, Deserialize)]
pub struct TodoRequest {
    pub content: String,
    pub owner: String,
}

#[instrument]
pub async fn create_todo(
    state: web::Data<Arc<State>>,
    payload: web::Json<TodoRequest>,
) -> Result<HttpResponse, Error> {
    let mut transaction = state.pg_pool.begin().await?;
    let amqp_channel = state.amqp_connection.create_channel().await?;

    let res = insert_to_db(&mut transaction, payload.into_inner()).await?;
    notify(&amqp_channel, &res, &*state.config.notification_exchange).await?;

    transaction.commit().await?;

    Ok(HttpResponse::Created().json(res))
}

#[instrument]
async fn notify(channel: &Channel, todo: &TodoResponse, exchange: &str) -> anyhow::Result<()> {
    let mut amqp_headers = BTreeMap::new();

    let span = Span::current();
    let cx = span.context();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut AmqpClientCarrier::new(&mut amqp_headers))
    });

    channel
        .basic_publish(
            exchange,
            "",
            BasicPublishOptions::default(),
            &*serde_json::to_vec(todo)?,
            BasicProperties::default().with_headers(FieldTable::from(amqp_headers)),
        )
        .await?;

    Ok(())
}

#[instrument]
async fn insert_to_db(
    transaction: &mut Transaction<'_, Postgres>,
    req: TodoRequest,
) -> anyhow::Result<TodoResponse> {
    sqlx::query_as!(
        TodoResponse,
        r#"
            insert into todo (owner, content)
            values ($1, $2)
            returning owner, content, created_at
        "#,
        req.owner.clone(),
        req.content.clone()
    )
    .fetch_one(transaction)
    .await
    .map_err(Into::into)
}
