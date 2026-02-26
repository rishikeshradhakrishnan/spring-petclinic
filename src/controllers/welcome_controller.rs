use actix_web::{web, HttpResponse};
use tera::Tera;

pub async fn welcome(tmpl: web::Data<Tera>) -> HttpResponse {
    let ctx = tera::Context::new();
    let body = tmpl
        .render("welcome.html", &ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok().content_type("text/html").body(body)
}

pub async fn trigger_error() -> HttpResponse {
    let mut ctx = tera::Context::new();
    ctx.insert("status", &500);
    ctx.insert(
        "message",
        "Expected: controller used to showcase what happens when an exception is thrown",
    );
    HttpResponse::InternalServerError()
        .content_type("text/html")
        .body("Error triggered intentionally")
}
