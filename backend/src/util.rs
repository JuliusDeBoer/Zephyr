use std::num::ParseIntError;

use actix_web::{
    ResponseError,
    http::{StatusCode, header::ToStrError},
};
use argon2::password_hash;
use hmac::digest::InvalidLength;
use rootcause::Report;

pub const fn status_error(code: StatusCode) -> EndpointError {
    EndpointError::StatusCode(code)
}

#[derive(thiserror::Error, Debug)]
pub enum EndpointError {
    #[error("{0}")]
    Report(Report),

    #[error("{0}")]
    StatusCode(StatusCode),

    #[error(transparent)]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error(transparent)]
    InvalidLength(#[from] InvalidLength),
    #[error(transparent)]
    JwtError(#[from] jwt::Error),
    #[error(transparent)]
    PasswordHashError(#[from] password_hash::Error),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    ToStrError(#[from] ToStrError),
}

impl From<Report<dyn std::any::Any>> for EndpointError {
    fn from(report: Report) -> Self {
        Self::Report(report)
    }
}

impl From<Report<Self>> for EndpointError {
    fn from(report: Report<Self>) -> Self {
        Self::Report(report.into())
    }
}

impl ResponseError for EndpointError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidLength(_) => StatusCode::BAD_REQUEST,
            Self::StatusCode(code) => *code,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
