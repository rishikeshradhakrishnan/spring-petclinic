use actix_web::{web, HttpResponse};
use sqlx::SqlitePool;
use tera::Tera;

use crate::models::{FindOwnerQuery, Owner, OwnerForm, OwnerListItem, PetWithVisits, ValidationErrors, VisitDisplay};
use crate::repositories::{owner_repository, pet_repository, visit_repository};

const PAGE_SIZE: i64 = 5;

pub async fn init_find_form(tmpl: web::Data<Tera>) -> HttpResponse {
    let mut ctx = tera::Context::new();
    ctx.insert("lastName", "");
    ctx.insert("errors", &serde_json::json!({}));
    let body = tmpl
        .render("owners/findOwners.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn process_find_form(
    pool: web::Data<SqlitePool>,
    tmpl: web::Data<Tera>,
    query: web::Query<FindOwnerQuery>,
) -> HttpResponse {
    let last_name = query.last_name.as_deref().unwrap_or("");
    let page = query.page.unwrap_or(1).max(1);

    let (owners, total) = match owner_repository::find_by_last_name(&pool, last_name, page, PAGE_SIZE).await {
        Ok(result) => result,
        Err(e) => {
            log::error!("Database error: {e}");
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    if total == 0 {
        let mut ctx = tera::Context::new();
        ctx.insert("lastName", last_name);
        let mut errors = std::collections::HashMap::new();
        errors.insert("lastName", "has not been found");
        ctx.insert("errors", &errors);
        let body = tmpl
            .render("owners/findOwners.html", &ctx)
            .unwrap_or_else(|e| format!("Template error: {e}"));
        return HttpResponse::Ok().content_type("text/html").body(body);
    }

    if total == 1 {
        let owner = &owners[0];
        return HttpResponse::Found()
            .append_header(("Location", format!("/owners/{}", owner.id.unwrap())))
            .finish();
    }

    // Build owner list items with pet names
    let mut list_items = Vec::new();
    for owner in &owners {
        let pet_names = pet_repository::pet_names_for_owner(&pool, owner.id.unwrap())
            .await
            .unwrap_or_default();
        list_items.push(OwnerListItem {
            id: owner.id.unwrap(),
            first_name: owner.first_name.clone(),
            last_name: owner.last_name.clone(),
            address: owner.address.clone(),
            city: owner.city.clone(),
            telephone: owner.telephone.clone(),
            pets: pet_names.join(", "),
        });
    }

    let total_pages = (total + PAGE_SIZE - 1) / PAGE_SIZE;

    let mut ctx = tera::Context::new();
    ctx.insert("listOwners", &list_items);
    ctx.insert("currentPage", &page);
    ctx.insert("totalPages", &total_pages);
    let body = tmpl
        .render("owners/ownersList.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn init_creation_form(tmpl: web::Data<Tera>) -> HttpResponse {
    let mut ctx = tera::Context::new();
    ctx.insert("owner", &serde_json::json!({
        "firstName": "", "lastName": "", "address": "", "city": "", "telephone": ""
    }));
    ctx.insert("is_new", &true);
    ctx.insert("errors", &serde_json::json!({}));
    let body = tmpl
        .render("owners/createOrUpdateOwnerForm.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn process_creation_form(
    pool: web::Data<SqlitePool>,
    form: web::Form<OwnerForm>,
) -> HttpResponse {
    let mut errors = ValidationErrors::new();
    let first_name = form.first_name.as_deref().unwrap_or("").trim().to_string();
    let last_name = form.last_name.as_deref().unwrap_or("").trim().to_string();
    let address = form.address.as_deref().unwrap_or("").trim().to_string();
    let city = form.city.as_deref().unwrap_or("").trim().to_string();
    let telephone = form.telephone.as_deref().unwrap_or("").trim().to_string();

    if first_name.is_empty() {
        errors.add("firstName", "is required");
    }
    if last_name.is_empty() {
        errors.add("lastName", "is required");
    }
    if address.is_empty() {
        errors.add("address", "is required");
    }
    if city.is_empty() {
        errors.add("city", "is required");
    }
    if telephone.is_empty() {
        errors.add("telephone", "is required");
    } else if !telephone.chars().all(|c| c.is_ascii_digit()) || telephone.len() != 10 {
        errors.add("telephone", "Telephone must be a 10-digit number");
    }

    if errors.has_errors() {
        let tmpl = actix_web::web::Data::new(
            tera::Tera::new("templates/**/*").expect("Tera init failed"),
        );
        let mut ctx = tera::Context::new();
        ctx.insert("owner", &serde_json::json!({
            "firstName": first_name, "lastName": last_name,
            "address": address, "city": city, "telephone": telephone
        }));
        ctx.insert("is_new", &true);
        let error_map: std::collections::HashMap<&str, &str> =
            errors.errors.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        ctx.insert("errors", &error_map);
        let body = tmpl
            .render("owners/createOrUpdateOwnerForm.html", &ctx)
            .unwrap_or_else(|e| format!("Template error: {e}"));
        return HttpResponse::Ok().content_type("text/html").body(body);
    }

    let owner = Owner {
        id: None,
        first_name,
        last_name,
        address,
        city,
        telephone,
    };

    match owner_repository::insert(&pool, &owner).await {
        Ok(id) => HttpResponse::Found()
            .append_header(("Location", format!("/owners/{id}")))
            .finish(),
        Err(e) => {
            log::error!("Failed to create owner: {e}");
            HttpResponse::InternalServerError().body("Failed to create owner")
        }
    }
}

pub async fn show_owner(
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

    let pets = pet_repository::find_by_owner_id(&pool, owner_id)
        .await
        .unwrap_or_default();

    let mut pets_with_visits = Vec::new();
    for pet in &pets {
        let pet_type = pet_repository::find_type_by_id(&pool, pet.type_id)
            .await
            .ok()
            .flatten()
            .map(|t| t.name)
            .unwrap_or_default();

        let visits = visit_repository::find_by_pet_id(&pool, pet.id.unwrap())
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

        pets_with_visits.push(PetWithVisits {
            id: pet.id.unwrap(),
            name: pet.name.clone(),
            birth_date: pet
                .birth_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            pet_type,
            visits: visit_displays,
        });
    }

    let mut ctx = tera::Context::new();
    ctx.insert(
        "owner",
        &serde_json::json!({
            "id": owner.id,
            "firstName": owner.first_name,
            "lastName": owner.last_name,
            "address": owner.address,
            "city": owner.city,
            "telephone": owner.telephone,
            "pets": pets_with_visits,
        }),
    );
    let body = tmpl
        .render("owners/ownerDetails.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn init_update_form(
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

    let mut ctx = tera::Context::new();
    ctx.insert("owner", &serde_json::json!({
        "firstName": owner.first_name, "lastName": owner.last_name,
        "address": owner.address, "city": owner.city, "telephone": owner.telephone
    }));
    ctx.insert("is_new", &false);
    ctx.insert("errors", &serde_json::json!({}));
    let body = tmpl
        .render("owners/createOrUpdateOwnerForm.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn process_update_form(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    form: web::Form<OwnerForm>,
) -> HttpResponse {
    let owner_id = path.into_inner();

    let mut errors = ValidationErrors::new();
    let first_name = form.first_name.as_deref().unwrap_or("").trim().to_string();
    let last_name = form.last_name.as_deref().unwrap_or("").trim().to_string();
    let address = form.address.as_deref().unwrap_or("").trim().to_string();
    let city = form.city.as_deref().unwrap_or("").trim().to_string();
    let telephone = form.telephone.as_deref().unwrap_or("").trim().to_string();

    if first_name.is_empty() {
        errors.add("firstName", "is required");
    }
    if last_name.is_empty() {
        errors.add("lastName", "is required");
    }
    if address.is_empty() {
        errors.add("address", "is required");
    }
    if city.is_empty() {
        errors.add("city", "is required");
    }
    if telephone.is_empty() {
        errors.add("telephone", "is required");
    } else if !telephone.chars().all(|c| c.is_ascii_digit()) || telephone.len() != 10 {
        errors.add("telephone", "Telephone must be a 10-digit number");
    }

    if errors.has_errors() {
        let tmpl = tera::Tera::new("templates/**/*").expect("Tera init failed");
        let mut ctx = tera::Context::new();
        ctx.insert("owner", &serde_json::json!({
            "firstName": first_name, "lastName": last_name,
            "address": address, "city": city, "telephone": telephone
        }));
        ctx.insert("is_new", &false);
        let error_map: std::collections::HashMap<&str, &str> =
            errors.errors.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        ctx.insert("errors", &error_map);
        let body = tmpl
            .render("owners/createOrUpdateOwnerForm.html", &ctx)
            .unwrap_or_else(|e| format!("Template error: {e}"));
        return HttpResponse::Ok().content_type("text/html").body(body);
    }

    let owner = Owner {
        id: Some(owner_id),
        first_name,
        last_name,
        address,
        city,
        telephone,
    };

    match owner_repository::update(&pool, owner_id, &owner).await {
        Ok(()) => HttpResponse::Found()
            .append_header(("Location", format!("/owners/{owner_id}")))
            .finish(),
        Err(e) => {
            log::error!("Failed to update owner: {e}");
            HttpResponse::InternalServerError().body("Failed to update owner")
        }
    }
}
