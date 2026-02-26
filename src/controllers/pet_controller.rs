use actix_web::{web, HttpResponse};
use chrono::NaiveDate;
use sqlx::SqlitePool;
use tera::Tera;

use crate::models::{Pet, PetForm, ValidationErrors};
use crate::repositories::{owner_repository, pet_repository};

pub async fn init_creation_form(
    pool: web::Data<SqlitePool>,
    tmpl: web::Data<Tera>,
    path: web::Path<i64>,
) -> HttpResponse {
    let owner_id = path.into_inner();

    let owner = match owner_repository::find_by_id(&pool, owner_id).await {
        Ok(Some(o)) => o,
        Ok(None) => return HttpResponse::NotFound().body("Owner not found"),
        Err(e) => {
            log::error!("Database error: {e}");
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let types = pet_repository::find_pet_types(&pool)
        .await
        .unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert(
        "owner",
        &serde_json::json!({
            "id": owner.id,
            "firstName": owner.first_name,
            "lastName": owner.last_name,
        }),
    );
    ctx.insert(
        "pet",
        &serde_json::json!({
            "name": "", "birthDate": "", "type": ""
        }),
    );
    ctx.insert("is_new", &true);
    ctx.insert("types", &types);
    ctx.insert("errors", &serde_json::json!({}));
    let body = tmpl
        .render("pets/createOrUpdatePetForm.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn process_creation_form(
    pool: web::Data<SqlitePool>,
    tmpl: web::Data<Tera>,
    path: web::Path<i64>,
    form: web::Form<PetForm>,
) -> HttpResponse {
    let owner_id = path.into_inner();

    let owner = match owner_repository::find_by_id(&pool, owner_id).await {
        Ok(Some(o)) => o,
        Ok(None) => return HttpResponse::NotFound().body("Owner not found"),
        Err(e) => {
            log::error!("Database error: {e}");
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let types = pet_repository::find_pet_types(&pool)
        .await
        .unwrap_or_default();

    let mut errors = ValidationErrors::new();
    let name = form.name.as_deref().unwrap_or("").trim().to_string();
    let birth_date_str = form.birth_date.as_deref().unwrap_or("").trim().to_string();
    let type_name = form.pet_type.as_deref().unwrap_or("").trim().to_string();

    if name.is_empty() {
        errors.add("name", "is required");
    }
    if birth_date_str.is_empty() {
        errors.add("birthDate", "is required");
    }
    if type_name.is_empty() {
        errors.add("type", "is required");
    }

    let birth_date = if !birth_date_str.is_empty() {
        match NaiveDate::parse_from_str(&birth_date_str, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => {
                errors.add("birthDate", "invalid date");
                None
            }
        }
    } else {
        None
    };

    let pet_type = if !type_name.is_empty() {
        pet_repository::find_type_by_name(&pool, &type_name)
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    if errors.has_errors() {
        let mut ctx = tera::Context::new();
        ctx.insert(
            "owner",
            &serde_json::json!({
                "id": owner.id,
                "firstName": owner.first_name,
                "lastName": owner.last_name,
            }),
        );
        ctx.insert(
            "pet",
            &serde_json::json!({
                "name": name, "birthDate": birth_date_str, "type": type_name
            }),
        );
        ctx.insert("is_new", &true);
        ctx.insert("types", &types);
        let error_map: std::collections::HashMap<&str, &str> = errors
            .errors
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        ctx.insert("errors", &error_map);
        let body = tmpl
            .render("pets/createOrUpdatePetForm.html", &ctx)
            .unwrap_or_else(|e| format!("Template error: {e}"));
        return HttpResponse::Ok().content_type("text/html").body(body);
    }

    let pet = Pet {
        id: None,
        name,
        birth_date,
        type_id: pet_type.map(|t| t.id).unwrap_or(1),
        owner_id,
    };

    match pet_repository::insert(&pool, &pet).await {
        Ok(_) => HttpResponse::Found()
            .append_header(("Location", format!("/owners/{owner_id}")))
            .finish(),
        Err(e) => {
            log::error!("Failed to create pet: {e}");
            HttpResponse::InternalServerError().body("Failed to create pet")
        }
    }
}

pub async fn init_update_form(
    pool: web::Data<SqlitePool>,
    tmpl: web::Data<Tera>,
    path: web::Path<(i64, i64)>,
) -> HttpResponse {
    let (owner_id, pet_id) = path.into_inner();

    let owner = match owner_repository::find_by_id(&pool, owner_id).await {
        Ok(Some(o)) => o,
        Ok(None) => return HttpResponse::NotFound().body("Owner not found"),
        Err(e) => {
            log::error!("Database error: {e}");
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let pet = match pet_repository::find_by_id(&pool, pet_id).await {
        Ok(Some(p)) => p,
        Ok(None) => return HttpResponse::NotFound().body("Pet not found"),
        Err(e) => {
            log::error!("Database error: {e}");
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let types = pet_repository::find_pet_types(&pool)
        .await
        .unwrap_or_default();

    let pet_type = pet_repository::find_type_by_id(&pool, pet.type_id)
        .await
        .ok()
        .flatten()
        .map(|t| t.name)
        .unwrap_or_default();

    let mut ctx = tera::Context::new();
    ctx.insert(
        "owner",
        &serde_json::json!({
            "id": owner.id,
            "firstName": owner.first_name,
            "lastName": owner.last_name,
        }),
    );
    ctx.insert(
        "pet",
        &serde_json::json!({
            "name": pet.name,
            "birthDate": pet.birth_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
            "type": pet_type
        }),
    );
    ctx.insert("is_new", &false);
    ctx.insert("types", &types);
    ctx.insert("errors", &serde_json::json!({}));
    let body = tmpl
        .render("pets/createOrUpdatePetForm.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn process_update_form(
    pool: web::Data<SqlitePool>,
    tmpl: web::Data<Tera>,
    path: web::Path<(i64, i64)>,
    form: web::Form<PetForm>,
) -> HttpResponse {
    let (owner_id, pet_id) = path.into_inner();

    let owner = match owner_repository::find_by_id(&pool, owner_id).await {
        Ok(Some(o)) => o,
        Ok(None) => return HttpResponse::NotFound().body("Owner not found"),
        Err(e) => {
            log::error!("Database error: {e}");
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let types = pet_repository::find_pet_types(&pool)
        .await
        .unwrap_or_default();

    let mut errors = ValidationErrors::new();
    let name = form.name.as_deref().unwrap_or("").trim().to_string();
    let birth_date_str = form.birth_date.as_deref().unwrap_or("").trim().to_string();
    let type_name = form.pet_type.as_deref().unwrap_or("").trim().to_string();

    if name.is_empty() {
        errors.add("name", "is required");
    }
    if birth_date_str.is_empty() {
        errors.add("birthDate", "is required");
    }
    if type_name.is_empty() {
        errors.add("type", "is required");
    }

    let birth_date = if !birth_date_str.is_empty() {
        match NaiveDate::parse_from_str(&birth_date_str, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => {
                errors.add("birthDate", "invalid date");
                None
            }
        }
    } else {
        None
    };

    let pet_type = if !type_name.is_empty() {
        pet_repository::find_type_by_name(&pool, &type_name)
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    if errors.has_errors() {
        let mut ctx = tera::Context::new();
        ctx.insert(
            "owner",
            &serde_json::json!({
                "id": owner.id,
                "firstName": owner.first_name,
                "lastName": owner.last_name,
            }),
        );
        ctx.insert(
            "pet",
            &serde_json::json!({
                "name": name, "birthDate": birth_date_str, "type": type_name
            }),
        );
        ctx.insert("is_new", &false);
        ctx.insert("types", &types);
        let error_map: std::collections::HashMap<&str, &str> = errors
            .errors
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        ctx.insert("errors", &error_map);
        let body = tmpl
            .render("pets/createOrUpdatePetForm.html", &ctx)
            .unwrap_or_else(|e| format!("Template error: {e}"));
        return HttpResponse::Ok().content_type("text/html").body(body);
    }

    let pet = Pet {
        id: Some(pet_id),
        name,
        birth_date,
        type_id: pet_type.map(|t| t.id).unwrap_or(1),
        owner_id,
    };

    match pet_repository::update(&pool, pet_id, &pet).await {
        Ok(()) => HttpResponse::Found()
            .append_header(("Location", format!("/owners/{owner_id}")))
            .finish(),
        Err(e) => {
            log::error!("Failed to update pet: {e}");
            HttpResponse::InternalServerError().body("Failed to update pet")
        }
    }
}
