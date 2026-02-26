use actix_web::{web, HttpResponse};
use sqlx::SqlitePool;
use tera::Tera;

use crate::models::PageQuery;
use crate::repositories::vet_repository;

const PAGE_SIZE: i64 = 5;

pub async fn show_vet_list(
    pool: web::Data<SqlitePool>,
    tmpl: web::Data<Tera>,
    query: web::Query<PageQuery>,
) -> HttpResponse {
    let page = query.page.unwrap_or(1).max(1);

    let (vets, total) = match vet_repository::find_all(&pool, page, PAGE_SIZE).await {
        Ok(result) => result,
        Err(e) => {
            log::error!("Database error: {e}");
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let total_pages = (total + PAGE_SIZE - 1) / PAGE_SIZE;

    let mut ctx = tera::Context::new();
    ctx.insert("listVets", &vets);
    ctx.insert("currentPage", &page);
    ctx.insert("totalPages", &total_pages);
    let body = tmpl
        .render("vets/vetList.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn show_vets_json(pool: web::Data<SqlitePool>) -> HttpResponse {
    let vets = match vet_repository::find_all_json(&pool).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("Database error: {e}");
            return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Database error"}));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({ "vetList": vets }))
}
