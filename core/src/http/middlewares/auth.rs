use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, web,
};
use futures::future::{ok, Ready};
use futures::Future;
use jsonwebtoken::{decode, Validation, DecodingKey};
use serde::{Deserialize, Serialize};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::rc::Rc;
use sea_orm::EntityTrait; // Import sea_orm for database queries
use crate::entities::users; // Import the users entity
use sea_orm::DatabaseConnection; // Import DatabaseConnection
use crate::http::response::ApiResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct AuthMiddleware;

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static, // Ensure B implements MessageBody
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
        })
    }
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static, // Ensure B implements MessageBody
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        // Clone the database connection from app_data
        let db_conn = req.app_data::<web::Data<DatabaseConnection>>().cloned();

        Box::pin(async move {
            // Check for token in Authorization header or query parameter
            let token = if let Some(auth_header) = req.headers().get("Authorization") {
                Some(auth_header.to_str().unwrap_or("").replace("Bearer ", ""))
            } else {
                // Check for token in query parameters
                let query = req.query_string();
                if let Some(token_value) = query.split('&')
                    .find(|pair| pair.starts_with("token=") || pair.starts_with("authorization="))
                    .and_then(|pair| pair.split('=').nth(1)) {
                    Some(token_value.to_string())
                } else {
                    None
                }
            };

            if let Some(token) = token {
                let secret = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set");
                let key = DecodingKey::from_secret(secret.as_ref());

                let token_data = decode::<Claims>(&token, &key, &Validation::default());

                match token_data {
                    Ok(token) => {
                        // Extract user_id from the token
                        let user_id = token.claims.sub.parse::<i32>().unwrap();

                        // Check if we have a database connection available
                        if let Some(db) = db_conn {
                            // Query the database to check if the user exists
                            let user_exists = users::Entity::find_by_id(user_id)
                                .one(db.get_ref())
                                .await;

                            match user_exists {
                                Ok(Some(_)) => {
                                    // User exists, proceed with the request
                                    req.extensions_mut().insert(token.claims);
                                    let res = service.call(req).await?;
                                    let res = res.map_into_boxed_body();
                                    Ok(res)
                                }
                                Ok(None) => {
                                    // User does not exist, return Unauthorized
                                    let res = HttpResponse::Unauthorized()
                                        .json(ApiResponse::<()> {
                                            success: false,
                                            message: "User does not exist".to_string(),
                                            data: None,
                                        });
                                    Ok(req.into_response(res.map_into_boxed_body()))
                                }
                                Err(_) => {
                                    // Database query error
                                    let res = HttpResponse::InternalServerError()
                                        .json(ApiResponse::<()> {
                                            success: false,
                                            message: "Error checking user in the database".to_string(),
                                            data: None,
                                        });
                                    Ok(req.into_response(res.map_into_boxed_body()))
                                }
                            }
                        } else {
                            // No database connection available
                            let res = HttpResponse::InternalServerError()
                                .json(ApiResponse::<()> {
                                    success: false,
                                    message: "Database connection not available".to_string(),
                                    data: None,
                                });
                            Ok(req.into_response(res.map_into_boxed_body()))
                        }
                    }
                    Err(_) => {
                        // Invalid token
                        let res = HttpResponse::Unauthorized()
                            .json(ApiResponse::<()> {
                                success: false,
                                message: "Invalid token".to_string(),
                                data: None,
                            });
                        Ok(req.into_response(res.map_into_boxed_body()))
                    }
                }
            } else {
                // No token provided in header or query params
                let res = HttpResponse::Unauthorized()
                    .json(ApiResponse::<()> {
                        success: false,
                        message: "No token provided".to_string(),
                        data: None,
                    });
                Ok(req.into_response(res.map_into_boxed_body()))
            }
        })
    }
}