use actix_web::web;
use crate::http::middlewares::auth::AuthMiddleware;
use crate::http::controllers::auth_controller::{login_user, logout_user, register_user};
use crate::http::controllers::user_controller::user_details;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(register_user);
    cfg.service(login_user);
    cfg.service(
        web::scope("/api")
            .wrap(AuthMiddleware {})

            .service(logout_user)
            // User Controller apis
            .service(user_details)
    );
}
