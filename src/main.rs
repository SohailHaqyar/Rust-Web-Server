#[macro_use]
extern crate diesel;

mod models;
mod schema;

use std::collections::HashMap;

use self::models::*;
use actix_files::Files;
use actix_web::{http, web, App, Error, HttpResponse, HttpServer};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use self::schema::memes::dsl::*;
use handlebars::Handlebars;
use serde::Serialize;
type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Serialize)]
struct IndexTemplateData {
    project_name: String,
    cats: Vec<self::models::Meme>,
}

async fn index(
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("couldn't get db connection from pool");
    let memes_data = web::block(move || memes.limit(100).load::<Meme>(&connection))
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;

    let data = IndexTemplateData {
        project_name: "Memeadex".to_string(),
        cats: memes_data,
    };
    let body = hb.render("index", &data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}
async fn add_cat_form(
    pool: web::Data<DbPool>,
    mut parts: awmp::Parts,
) -> Result<HttpResponse, Error> {
    let file_path = parts
        .files
        .take("image")
        .pop()
        .and_then(|f| f.persist_in("./static/image").ok())
        .unwrap_or_default();

    let text_fields: HashMap<_, _> = parts.texts.as_pairs().into_iter().collect();
    let connection = pool.get().expect("couldn't get db connection from pool");

    let new_meme = NewMeme {
        name: text_fields.get("name").unwrap().to_string(),
        image_path: file_path.to_string_lossy().to_string(),
    };
    web::block(move || {
        diesel::insert_into(memes)
            .values(&new_meme)
            .execute(&connection)
    })
    .await
    .map_err(|_| HttpResponse::InternalServerError().finish())?;
    Ok(HttpResponse::SeeOther()
        .header(http::header::LOCATION, "/")
        .finish())
}

async fn add(hb: web::Data<Handlebars<'_>>) -> Result<HttpResponse, Error> {
    let body = hb.render("add", &{}).unwrap();
    Ok(HttpResponse::Ok().body(body))
}

async fn meme(
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<DbPool>,
    meme_id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    // get db pool
    let connection = pool.get().expect("couldn't get db connection from pool");
    // get meme by id
    let meme_data = web::block(move || {
        memes
            .filter(id.eq(meme_id.into_inner()))
            .first::<Meme>(&connection)
    })
    .await
    .map_err(|_| HttpResponse::InternalServerError().finish())?;

    let body = hb.render("meme", &meme_data).unwrap();
    Ok(HttpResponse::Ok().body(body))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./static/")
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);

    // Setting up database
    let database_url = "postgres://actix:actix@localhost:5432";
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    println!("Listening on port 8080");
    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .data(pool.clone())
            .service(Files::new("/static", "static").show_files_listing())
            .route("/", web::get().to(index))
            .route("/add", web::get().to(add))
            .route("/add_cat_form", web::post().to(add_cat_form))
            .route("/meme/{id}", web::get().to(meme))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
