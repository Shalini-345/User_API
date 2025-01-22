use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web::post;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::RunQueryDsl;
mod models;
mod schema;
mod db;

use crate::models::{User, NewUser};
use crate::schema::users;
use crate::db::establish_connection_pool;

#[post("/users/register")]
async fn register_user(
    new_user: web::Json<NewUser>,
    pool: web::Data<db::Pool>,
) -> impl Responder {
    let conn = &mut pool.get().unwrap();

    match diesel::insert_into(users::table)
        .values(&*new_user)
        .execute(conn) // Correctly pass the mutable reference
    {
        Ok(_) => HttpResponse::Ok().body("User registered successfully"),
        Err(e) => {
            eprintln!("Error registering user: {}", e);
            HttpResponse::InternalServerError().body("Failed to register user")
        }
    }
}

#[post("/users/login")]
async fn login_user(
    login_data: web::Json<NewUser>,
    pool: web::Data<db::Pool>,
) -> impl Responder {
    let conn = &mut pool.get().unwrap();

    match users::table
        .filter(users::username.eq(&login_data.username))
        .filter(users::password.eq(&login_data.password))
        .first::<User>(conn)
    {
        Ok(_) => HttpResponse::Ok().body("Login successful"),
        Err(DieselError::NotFound) => HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(e) => {
            eprintln!("Error during login: {}", e);
            HttpResponse::InternalServerError().body("Login failed")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = establish_connection_pool();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(register_user)
            .service(login_user)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
