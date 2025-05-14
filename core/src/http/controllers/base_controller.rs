use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use serde::Serialize;
use crate::entities::{users};
use crate::http::middlewares::auth::Claims;
use crate::http::response::ApiResponse;
/// Trait providing common controller utilities with standardized responses
pub trait BaseController {

    /// Extract claims from the request if available
    fn get_claims(req: &HttpRequest) -> Option<Claims> {
        req.extensions().get::<Claims>().cloned()
    }

    /// Generate a 200 OK response with data
    fn ok_with_data<T: Serialize>(message: &str, data: T) -> HttpResponse {
        HttpResponse::Ok().json(ApiResponse {
            success: true,
            message: message.to_string(),
            data: Some(data),
        })
    }

    /// Generate a 200 OK response with no data
    fn ok_empty(message: &str) -> HttpResponse {
        HttpResponse::Ok().json(ApiResponse::<()> {
            success: true,
            message: message.to_string(),
            data: None,
        })
    }

    /// Generate a 401 Unauthorized response
    fn unauthorized(message: &str) -> HttpResponse {
        HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            message: message.to_string(),
            data: None,
        })
    }

    /// Generate a 404 Not Found response
    fn not_found(message: &str) -> HttpResponse {
        HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            message: message.to_string(),
            data: None,
        })
    }

    /// Generate a 400 Bad Request response
    fn bad_request(message: &str) -> HttpResponse {
        HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            message: message.to_string(),
            data: None,
        })
    }

    /// Generate a 500 Internal Server Error response
    fn internal_server_error(message: &str) -> HttpResponse {
        HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: message.to_string(),
            data: None,
        })
    }

    /// Generate a 201 Created response
    fn created<T: Serialize>(message: &str, data: T) -> HttpResponse {
        HttpResponse::Created().json(ApiResponse {
            success: true,
            message: message.to_string(),
            data: Some(data),
        })
    }

    async fn get_user_by_id(
        user_id: i32,
        db: &DatabaseConnection
    ) -> Result<users::Model, HttpResponse>{
        // Retrieve the user to be updated
        match users::Entity::find()
            .filter(users::Column::Id.eq(user_id))
            .one(db)
            .await
        {
            Ok(Some(user_model)) => Ok(user_model),
            Ok(None) => Err(Self::not_found("User not found")),
            Err(_e) => Err(Self::internal_server_error("Error retrieving user"))
        }
    }

}

// Define the controller struct
pub struct Controller;

impl BaseController for Controller {}