use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(author, about, version)]
pub struct Config {
    #[clap(long, env, default_value = "amqp://guest:guest@127.0.0.1:5672//")]
    pub amqp_uri: String,

    #[clap(long, env, default_value = "notification-exchange")]
    pub notification_exchange: String,

    #[clap(long, env, default_value = "notification-queue")]
    pub notification_queue: String,
}
