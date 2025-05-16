use actix_web::{get, HttpRequest, Responder, web};
use crate::http::controllers::base_controller::{BaseController, Controller};
use crate::entities::{users, user_wallet};
use crate::enums::user_type::UserType;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter};
use std::str::FromStr;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UserWithWallet {
    pub user: users::Model,
    pub wallet_address: Option<String>,
}

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

#[get("/user/{id}/details")]
pub async fn get_user_details_by_id(
    req: HttpRequest,
    path: web::Path<i32>,
    db: web::Data<sea_orm::DatabaseConnection>,
) -> impl Responder {
    let claims = Controller::get_claims(&req);
    if claims.is_none() {
        return Controller::unauthorized("Unauthorized");
    }

    // Get the requesting user's ID and check if they're a proctor
    let requester_id: i32 = claims.unwrap().sub.parse().unwrap();
    let requester = match Controller::get_user_by_id(requester_id, db.get_ref()).await {
        Ok(user) => user,
        Err(err_response) => return err_response,
    };

    // Check if the requester is a proctor
    let user_type = match UserType::from_str(&requester.r#type) {
        Ok(user_type) => user_type,
        Err(_) => return Controller::bad_request("Invalid user type"),
    };

    if user_type != UserType::Proctor && user_type != UserType::Admin {
        return Controller::unauthorized("Only proctors and admins can access user details");
    }

    // Get the requested user's details
    let requested_user_id = path.into_inner();
    let user = match Controller::get_user_by_id(requested_user_id, db.get_ref()).await {
        Ok(user) => user,
        Err(err_response) => return err_response,
    };

    // Get the user's wallet address
    let wallet_address = match user_wallet::Entity::find()
        .filter(user_wallet::Column::UserId.eq(user.id as i64))
        .one(db.get_ref())
        .await {
        Ok(Some(wallet)) => Some(wallet.address),
        Ok(None) => None,
        Err(_) => return Controller::internal_server_error("Error retrieving wallet details"),
    };

    let user_with_wallet = UserWithWallet {
        user,
        wallet_address,
    };

    Controller::ok_with_data(
        "User details retrieved successfully",
        Some(user_with_wallet),
    )
}
