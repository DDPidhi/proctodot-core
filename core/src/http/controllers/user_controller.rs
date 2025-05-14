use actix_web::{get, HttpRequest, HttpResponse, Responder, web, HttpMessage};
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
use crate::entities::users;
use crate::http::middlewares::auth::Claims;
use crate::http::response::ApiResponse;

#[get("/user/details")]
pub async fn user_details(
    req: HttpRequest,
    db: web::Data<sea_orm::DatabaseConnection>,
) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        let user_id: i32 = claims.sub.parse().unwrap();

        let user = users::Entity::find()
            .filter(users::Column::Id.eq(user_id))
            .one(db.get_ref())
            .await;

        match user {
            Ok(Some(user)) => HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "User details retrieved successfully".to_string(),
                data: Some(user),
            }),
            Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()> {
                success: false,
                message: "User not found".to_string(),
                data: None,
            }),
            Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: "Error retrieving user details".to_string(),
                data: None,
            }),
        }
    } else {
        HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            message: "Unauthorized".to_string(),
            data: None,
        })
    }
}
