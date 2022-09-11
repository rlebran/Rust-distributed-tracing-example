use crate::Config;
use lapin::{Connection, ConnectionProperties};
use sqlx::PgPool;

#[derive(Debug)]
pub struct State {
    pub amqp_connection: Connection,
    pub pg_pool: PgPool,
    pub config: Config,
}

impl State {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let connection_properties = ConnectionProperties::default();
        let amqp_connection = Connection::connect(&*config.amqp_uri, connection_properties).await?;

        let pg_pool = PgPool::connect(&*config.postgres_uri).await?;

        Ok(Self {
            amqp_connection,
            pg_pool,
            config,
        })
    }
}
