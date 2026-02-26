mod controllers;
mod db;
mod models;
mod repositories;

use actix_files as fs;
use actix_web::{web, App, HttpServer};
use tera::Tera;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Initializing database...");
    let pool = db::init_pool().await;

    log::info!("Starting PetClinic server on http://localhost:8080");

    let tera = Tera::new("templates/**/*").expect("Failed to initialize Tera templates");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(tera.clone()))
            // Static files
            .service(fs::Files::new("/resources", "static").show_files_listing())
            // Welcome
            .route("/", web::get().to(controllers::welcome_controller::welcome))
            // Error trigger
            .route(
                "/oups",
                web::get().to(controllers::welcome_controller::trigger_error),
            )
            // Owner routes
            .route(
                "/owners/new",
                web::get().to(controllers::owner_controller::init_creation_form),
            )
            .route(
                "/owners/new",
                web::post().to(controllers::owner_controller::process_creation_form),
            )
            .route(
                "/owners/find",
                web::get().to(controllers::owner_controller::init_find_form),
            )
            .route(
                "/owners",
                web::get().to(controllers::owner_controller::process_find_form),
            )
            .route(
                "/owners/{ownerId}",
                web::get().to(controllers::owner_controller::show_owner),
            )
            .route(
                "/owners/{ownerId}/edit",
                web::get().to(controllers::owner_controller::init_update_form),
            )
            .route(
                "/owners/{ownerId}/edit",
                web::post().to(controllers::owner_controller::process_update_form),
            )
            // Pet routes
            .route(
                "/owners/{ownerId}/pets/new",
                web::get().to(controllers::pet_controller::init_creation_form),
            )
            .route(
                "/owners/{ownerId}/pets/new",
                web::post().to(controllers::pet_controller::process_creation_form),
            )
            .route(
                "/owners/{ownerId}/pets/{petId}/edit",
                web::get().to(controllers::pet_controller::init_update_form),
            )
            .route(
                "/owners/{ownerId}/pets/{petId}/edit",
                web::post().to(controllers::pet_controller::process_update_form),
            )
            // Visit routes
            .route(
                "/owners/{ownerId}/pets/{petId}/visits/new",
                web::get().to(controllers::visit_controller::init_creation_form),
            )
            .route(
                "/owners/{ownerId}/pets/{petId}/visits/new",
                web::post().to(controllers::visit_controller::process_creation_form),
            )
            // Vet routes
            .route(
                "/vets.html",
                web::get().to(controllers::vet_controller::show_vet_list),
            )
            .route(
                "/vets",
                web::get().to(controllers::vet_controller::show_vets_json),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
