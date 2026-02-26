use actix_web::{web, HttpResponse};
use chrono::NaiveDate;
use sqlx::SqlitePool;
use tera::Tera;

use crate::models::{ValidationErrors, Visit, VisitDisplay};
use crate::repositories::{owner_repository, pet_repository, visit_repository};

pub async fn init_creation_form(
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

    let pet_type = pet_repository::find_type_by_id(&pool, pet.type_id)
        .await
        .ok()
        .flatten()
        .map(|t| t.name)
        .unwrap_or_default();

    let visits = visit_repository::find_by_pet_id(&pool, pet_id)
        .await
        .unwrap_or_default();

    let visit_displays: Vec<VisitDisplay> = visits
        .into_iter()
        .map(|v| VisitDisplay {
            id: v.id.unwrap_or(0),
            date: v
                .visit_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            description: v.description,
        })
        .collect();

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

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
            "id": pet.id,
            "name": pet.name,
            "birthDate": pet.birth_date.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default(),
            "type": pet_type,
            "visits": visit_displays,
        }),
    );
    ctx.insert(
        "visit",
        &serde_json::json!({
            "date": today,
            "description": ""
        }),
    );
    ctx.insert("is_new", &true);
    ctx.insert("errors", &serde_json::json!({}));
    let body = tmpl
        .render("pets/createOrUpdateVisitForm.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn process_creation_form(
    pool: web::Data<SqlitePool>,
    path: web::Path<(i64, i64)>,
    form: web::Form<crate::models::VisitForm>,
) -> HttpResponse {
    let (owner_id, pet_id) = path.into_inner();

    let mut errors = ValidationErrors::new();
    let description = form.description.as_deref().unwrap_or("").trim().to_string();
    let date_str = form.date.as_deref().unwrap_or("").trim().to_string();

    if description.is_empty() {
        errors.add("description", "is required");
    }

    let visit_date = if !date_str.is_empty() {
        match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(d) => Some(d),
            Err(_) => {
                errors.add("date", "invalid date");
                None
            }
        }
    } else {
        Some(chrono::Local::now().date_naive())
    };

    if errors.has_errors() {
        // Re-render form with errors - redirect back for simplicity
        return HttpResponse::Found()
            .append_header((
                "Location",
                format!("/owners/{owner_id}/pets/{pet_id}/visits/new"),
            ))
            .finish();
    }

    let visit = Visit {
        id: None,
        pet_id,
        visit_date,
        description,
    };

    match visit_repository::insert(&pool, &visit).await {
        Ok(_) => HttpResponse::Found()
            .append_header(("Location", format!("/owners/{owner_id}")))
            .finish(),
        Err(e) => {
            log::error!("Failed to create visit: {e}");
            HttpResponse::InternalServerError().body("Failed to create visit")
        }
    }
}
