use actix_web::HttpMessage;
use actix_web::{
    HttpResponse,
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    error::InternalError,
    web,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use futures_util::future::LocalBoxFuture;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DerivePartialModel, EntityTrait, FromQueryResult, QueryFilter,
};
use std::{
    future::{Ready, ready},
    rc::Rc,
    sync::Arc,
};
use uuid::Uuid;

use crate::entity::prelude::User;
use crate::entity::user;
use crate::logic::jwt::validate_credentials;

#[derive(Clone, Debug)]
pub struct UserClaims {
    pub user_id: Uuid,
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "user::Entity")]
struct IdOnly {
    id: Uuid,
}

#[derive(Default)]
pub struct CalDavAuth {}

impl<S, B> Transform<S, ServiceRequest> for CalDavAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = CalDavAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CalDavAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct CalDavAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CalDavAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let err = Box::pin(async move {
            let response = HttpResponse::Unauthorized()
                .append_header(("WWW-Authenticate", "Basic realm=\"Zephyr\""))
                .finish();
            Err(InternalError::from_response("Invalid credentials", response).into())
        });

        let auth = req.headers().iter().find(|h| h.0 == "Authorization");

        if auth.is_none() {
            return err;
        }

        let auth_header = auth
            // NOTE(Julius): This error never happends since we already made
            //               sure the value is `Some()`
            .expect("Could not get auth header")
            .1
            .to_str()
            .unwrap_or("");

        let auth_value = auth_header
            .strip_prefix("Basic ")
            .map_or(auth_header, |stripped| stripped);

        let decoded: Vec<String> = match BASE64_STANDARD.decode(auth_value.as_bytes()) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(decoded_str) => decoded_str
                    .split(':')
                    .map(|v: &str| String::from(v))
                    .collect(),
                Err(_) => {
                    return err;
                }
            },
            Err(_) => {
                return err;
            }
        };

        if decoded.len() < 2 {
            return err;
        }

        let db = req
            .app_data::<web::Data<Arc<DatabaseConnection>>>()
            .expect("Could not get database connection")
            .clone();

        let service = self.service.clone();
        let email = decoded[0].clone();
        let password = decoded[1].clone();

        Box::pin(async move {
            let db_ref = db.as_ref().as_ref();
            if let Ok(valid) = validate_credentials(&email, &password, db_ref).await {
                if valid {
                    let user_result: Option<IdOnly> = User::find()
                        .filter(user::Column::Email.eq(email))
                        .into_partial_model()
                        .one(db_ref)
                        .await
                        .expect("Unexpected Database Error");

                    req.request().extensions_mut().insert(UserClaims {
                        user_id: user_result.expect("Could find user").id,
                    });
                    service.call(req).await
                } else {
                    let response = HttpResponse::Unauthorized()
                        .append_header(("WWW-Authenticate", "Basic realm=\"Zephyr\""))
                        .finish();
                    Err(InternalError::from_response("Unauthorized", response).into())
                }
            } else {
                let response = HttpResponse::InternalServerError().finish();
                Err(InternalError::from_response("Authentication error", response).into())
            }
        })
    }
}
