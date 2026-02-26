use sqlx::SqlitePool;

use crate::models::{Pet, PetType};

pub async fn find_pet_types(pool: &SqlitePool) -> Result<Vec<PetType>, sqlx::Error> {
    sqlx::query_as::<_, PetType>("SELECT id, name FROM types ORDER BY name")
        .fetch_all(pool)
        .await
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Pet>, sqlx::Error> {
    sqlx::query_as::<_, Pet>(
        "SELECT id, name, birth_date, type_id, owner_id FROM pets WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_owner_id(pool: &SqlitePool, owner_id: i64) -> Result<Vec<Pet>, sqlx::Error> {
    sqlx::query_as::<_, Pet>(
        "SELECT id, name, birth_date, type_id, owner_id FROM pets \
         WHERE owner_id = ?1 ORDER BY name",
    )
    .bind(owner_id)
    .fetch_all(pool)
    .await
}

pub async fn find_type_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<PetType>, sqlx::Error> {
    sqlx::query_as::<_, PetType>("SELECT id, name FROM types WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn find_type_by_name(
    pool: &SqlitePool,
    name: &str,
) -> Result<Option<PetType>, sqlx::Error> {
    sqlx::query_as::<_, PetType>("SELECT id, name FROM types WHERE name = ?1")
        .bind(name)
        .fetch_optional(pool)
        .await
}

pub async fn insert(pool: &SqlitePool, pet: &Pet) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO pets (name, birth_date, type_id, owner_id) VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(&pet.name)
    .bind(pet.birth_date)
    .bind(pet.type_id)
    .bind(pet.owner_id)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn update(pool: &SqlitePool, id: i64, pet: &Pet) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE pets SET name = ?1, birth_date = ?2, type_id = ?3, owner_id = ?4 WHERE id = ?5",
    )
    .bind(&pet.name)
    .bind(pet.birth_date)
    .bind(pet.type_id)
    .bind(pet.owner_id)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn pet_names_for_owner(
    pool: &SqlitePool,
    owner_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM pets WHERE owner_id = ?1 ORDER BY name")
            .bind(owner_id)
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(n,)| n).collect())
}
