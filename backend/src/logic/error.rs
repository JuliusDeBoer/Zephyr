//! Generic error handeling for Actix endpoints
//!
//! The [`ApiError`] is an error type that can be transformed into a response
//! through Actix. Plus it provides the [`WithStatusCode`] trait. Allowing for
//! adding a custom status code to the error.

use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError, body::BoxBody, http::StatusCode};
use rootcause::{Report, report};

/// An error that contains a [`Report`] and a [`StatusCode`]. Allowing for
/// easily returning an error to the top-level endpoint error.
#[derive(Debug)]
pub struct ApiError {
    pub report: Report,
    pub status_code: StatusCode,
}

impl ApiError {
    pub fn new(reason: &str, status_code: StatusCode) -> Self {
        Self {
            report: report!("{}", reason),
            status_code,
        }
    }
}

pub trait WithStatusCode<T> {
    fn with_status(self, code: StatusCode) -> Result<T, ApiError>;
}

impl<T, E> WithStatusCode<T> for Result<T, E>
where
    E: Into<Report>,
{
    fn with_status(self, code: StatusCode) -> Result<T, ApiError> {
        self.map_err(|e| ApiError {
            report: e.into(),
            status_code: code,
        })
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.status_code.as_u16(), self.report)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code).body(format!("{}", self.report))
    }
}

impl<E: Into<Report>> From<E> for ApiError {
    fn from(error: E) -> Self {
        Self {
            report: error.into(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
