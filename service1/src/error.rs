use actix_web::http::StatusCode;
use actix_web::ResponseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Amqp(#[from] lapin::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::Amqp(_) | Self::Sqlx(_) | Self::Unexpected(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}
