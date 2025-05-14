use crate::enums::user_type::UserType;
use actix_web::{post, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, DatabaseConnection, DbErr, EntityTrait, InsertResult, QueryFilter, Set};
use serde::{Deserialize};
use chrono::{Duration, Utc};
use sea_orm::ColumnTrait;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng, PasswordHash, PasswordVerifier};
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::http::response::ApiResponse;
use crate::http::middlewares::auth::Claims;
use std::env;
use actix_web::web::Data;
use crate::entities::{passwords, user_wallet, users};
use crate::entities::user_wallet::ActiveModel;
use crate::entities::users::Model;
use crate::web3::wallet_handler;

#[derive(Deserialize)]
pub struct RegisterUserRequest {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    pub phone: String,
    pub chain: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub confirm_password: String,
    pub password: String,
}

async fn generate_wallet(db: Data<DatabaseConnection>, user_id: i32) -> Result<InsertResult<ActiveModel>, DbErr> {
    let wallet_info = wallet_handler::WalletHandler::generate_wallet();
    // Store the encrypted private key and wallet info
    let mut new_wallet = user_wallet::Model {
        user_id: user_id as i64,
        public_key: wallet_info.public_key,
        address: wallet_info.address,
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
        ..Default::default()
    };

    new_wallet.set_mnemonic(&wallet_info.mnemonic).unwrap();
    new_wallet.set_private_key(&wallet_info.private_key).unwrap();
    let active_wallet_model: ActiveModel = new_wallet.into();

    let wallet_result = user_wallet::Entity::insert(active_wallet_model).exec(db.get_ref()).await;
    wallet_result
}

fn generate_user_response_with_token(user: Model) -> serde_json::Value {
    let expiration = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration.timestamp() as usize,
    };

    // Load the secret key from the .env file
    let secret = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(
            secret.as_ref()
        )
    ).unwrap();

    let result = serde_json::json!({
        "token": token,
        "user": user
    });

    result
}

#[post("/register/{type}")]
pub async fn register_user(
    db: Data<DatabaseConnection>,
    form: web::Json<RegisterUserRequest>,
    user_type: web::Path<String>,
) -> impl Responder {
    // Parse the user type from the path parameter
    let user_type = match user_type.into_inner().parse::<UserType>() {
        Ok(user_type) => user_type,
        Err(_) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                message: "Invalid user type. Must be either 'member' or 'proctor'.".to_string(),
                data: None,
            });
        }
    };

    // Check if the email already exists
    if let Ok(Some(_)) = users::Entity::find()
        .filter(users::Column::Email.eq(form.email.clone()))
        .one(db.get_ref())
        .await
    {
        return HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            message: "Email is already taken.".to_string(),
            data: None,
        });
    }

    // Create a new user
    let new_user = users::ActiveModel {
        r#type: Set(user_type.to_string()),
        chain: Set(form.chain.clone()),
        email: Set(form.email.clone()),
        first_name: Set(form.first_name.clone()),
        last_name: Set(form.last_name.clone()),
        phone: Set(form.phone.clone()),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };

    // Insert the new user into the database
    let result = users::Entity::insert(new_user).exec(db.get_ref()).await;

    let insert_result = match result {
        Ok(insert_result) => insert_result,
        Err(DbErr::Exec(err)) => {
            return HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                message: err.to_string(),
                data: None,
            });
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: "Error registering user.".to_string(),
                data: None,
            });
        }
    };

    // Retrieve the full user model using the last_insert_id
    let user_id = insert_result.last_insert_id;
    let user = match users::Entity::find_by_id(user_id).one(db.get_ref()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: "Error retrieving newly created user.".to_string(),
                data: None,
            });
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<()> {
                success: false,
                message: "Error retrieving user.".to_string(),
                data: None,
            });
        }
    };

    // Generate a salt
    let salt = SaltString::generate(&mut OsRng);

    // Hash the password using Argon2
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(form.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    // Insert the hashed password into the passwords table
    let new_password = passwords::ActiveModel {
        user_id: Set(user_id),
        password_hash: Set(password_hash),
        created_at: Set(Utc::now().into()),
        ..Default::default()
    };

    let password_result = passwords::Entity::insert(new_password)
        .exec(db.get_ref())
        .await;

    if password_result.is_err() {
        return HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Error saving password.".to_string(),
            data: None,
        });
    }

    // Generate the user wallet
    if let Err(_) = generate_wallet(db.clone(), user_id).await {
        return HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Error saving wallet information.".to_string(),
            data: None,
        });
    }

    let response_data = generate_user_response_with_token(user.clone());

    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message: "User registered successfully.".to_string(),
        data: Some(response_data),
    })
}

async fn generate_password(
    db: &Data<DatabaseConnection>,
    password: String,
    user_id: i32,
) -> Result<(), HttpResponse> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let new_password = passwords::ActiveModel {
        user_id: Set(user_id),
        password_hash: Set(password_hash),
        created_at: Set(Utc::now().into()),
        ..Default::default()
    };

    match passwords::Entity::insert(new_password).exec(db.get_ref()).await {
        Ok(_) => Ok(()),
        Err(_) => Err(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Error saving password.".to_string(),
            data: None,
        })),
    }
}

#[post("/login")]
pub async fn login_user(
    db: Data<DatabaseConnection>,
    form: web::Json<LoginRequest>,
) -> impl Responder {

    let user = users::Entity::find()
        .filter(users::Column::Email.eq(form.email.clone()))
        .one(db.get_ref())
        .await;

    match user {
        Ok(Some(user)) => {

            // Fetch the hashed password from the passwords table
            let stored_password = passwords::Entity::find()
                .filter(passwords::Column::UserId.eq(user.id))
                .one(db.get_ref())
                .await;

            match stored_password {
                Ok(Some(stored_password)) => {
                    let argon2 = Argon2::default();
                    let parsed_hash = PasswordHash::new(&stored_password.password_hash).unwrap();

                    if argon2.verify_password(form.password.as_bytes(), &parsed_hash).is_ok() {
                        let response_data = generate_user_response_with_token(user.clone());
                        HttpResponse::Ok().json(ApiResponse {
                            success: true,
                            message: "Login successful".to_string(),
                            data: Some(response_data),
                        })
                    } else {
                        HttpResponse::Unauthorized().json(ApiResponse::<()> {
                            success: false,
                            message: "Invalid username or password.".to_string(),
                            data: None,
                        })
                    }
                }
                _ => HttpResponse::Unauthorized().json(ApiResponse::<()> {
                    success: false,
                    message: "Invalid username or password.".to_string(),
                    data: None,
                }),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(ApiResponse::<()> {
            success: false,
            message: "Invalid username or password.".to_string(),
            data: None,
        }),
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            message: "Error logging in.".to_string(),
            data: None,
        }),
    }
}