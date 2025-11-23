use std::num::ParseIntError;

use actix_web::{
    ResponseError,
    http::{StatusCode, header::ToStrError},
};
use argon2::password_hash;
use hmac::digest::InvalidLength;

pub const fn status_error(code: StatusCode) -> EndpointError {
    EndpointError::StatusCodeError(code)
}

#[derive(thiserror::Error, Debug)]
pub enum EndpointError {
    #[error(transparent)]
    UnexpectedError(#[from] eyre::Error),
    #[error("{0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("{0}")]
    InvalidLength(#[from] InvalidLength),
    #[error("{0}")]
    JwtError(#[from] jwt::Error),
    #[error("{0}")]
    PasswordHashError(#[from] password_hash::Error),
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("{0}")]
    ToStrError(#[from] ToStrError),
    #[error("{0}")]
    StatusCodeError(StatusCode),
}

impl ResponseError for EndpointError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidLength(_) => StatusCode::BAD_REQUEST,
            Self::StatusCodeError(code) => *code,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
