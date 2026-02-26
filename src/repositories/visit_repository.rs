use sqlx::SqlitePool;

use crate::models::Visit;

pub async fn find_by_pet_id(pool: &SqlitePool, pet_id: i64) -> Result<Vec<Visit>, sqlx::Error> {
    sqlx::query_as::<_, Visit>(
        "SELECT id, pet_id, visit_date, description FROM visits \
         WHERE pet_id = ?1 ORDER BY visit_date DESC",
    )
    .bind(pet_id)
    .fetch_all(pool)
    .await
}

pub async fn insert(pool: &SqlitePool, visit: &Visit) -> Result<i64, sqlx::Error> {
    let result =
        sqlx::query("INSERT INTO visits (pet_id, visit_date, description) VALUES (?1, ?2, ?3)")
            .bind(visit.pet_id)
            .bind(visit.visit_date)
            .bind(&visit.description)
            .execute(pool)
            .await?;

    Ok(result.last_insert_rowid())
}
