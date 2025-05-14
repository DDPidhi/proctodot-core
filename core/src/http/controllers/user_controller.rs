use actix_web::{get, HttpRequest, Responder, web};
use crate::http::controllers::base_controller::{BaseController, Controller};

#[get("/user/details")]
pub async fn user_details(
    req: HttpRequest,
    db: web::Data<sea_orm::DatabaseConnection>,
) -> impl Responder {

    let claims = Controller::get_claims(&req);
    if claims.is_none() {
        return Controller::unauthorized("Unauthorized");
    }

    let user_id: i32 = claims.unwrap().sub.parse().unwrap();
    match Controller::get_user_by_id(user_id, db.get_ref()).await {
        Ok(user) => Controller::ok_with_data(
            "User details retrieved successfully",
            Some(user),
        ),
        Err(err_response) => err_response,
    }
}
