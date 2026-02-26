use sqlx::SqlitePool;

use crate::models::Owner;

pub async fn find_by_last_name(
    pool: &SqlitePool,
    last_name: &str,
    page: i64,
    page_size: i64,
) -> Result<(Vec<Owner>, i64), sqlx::Error> {
    let pattern = format!("{last_name}%");
    let offset = (page - 1) * page_size;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM owners WHERE last_name LIKE ?1 COLLATE NOCASE",
    )
    .bind(&pattern)
    .fetch_one(pool)
    .await?;

    let owners = sqlx::query_as::<_, Owner>(
        "SELECT id, first_name, last_name, address, city, telephone \
         FROM owners WHERE last_name LIKE ?1 COLLATE NOCASE \
         ORDER BY last_name LIMIT ?2 OFFSET ?3",
    )
    .bind(&pattern)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((owners, total.0))
}

pub async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Owner>, sqlx::Error> {
    sqlx::query_as::<_, Owner>(
        "SELECT id, first_name, last_name, address, city, telephone FROM owners WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn insert(pool: &SqlitePool, owner: &Owner) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO owners (first_name, last_name, address, city, telephone) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&owner.first_name)
    .bind(&owner.last_name)
    .bind(&owner.address)
    .bind(&owner.city)
    .bind(&owner.telephone)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn update(pool: &SqlitePool, id: i64, owner: &Owner) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE owners SET first_name = ?1, last_name = ?2, address = ?3, \
         city = ?4, telephone = ?5 WHERE id = ?6",
    )
    .bind(&owner.first_name)
    .bind(&owner.last_name)
    .bind(&owner.address)
    .bind(&owner.city)
    .bind(&owner.telephone)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}
