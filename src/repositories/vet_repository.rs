use sqlx::SqlitePool;

use crate::models::{Specialty, Vet, VetDisplay};

pub async fn find_all(
    pool: &SqlitePool,
    page: i64,
    page_size: i64,
) -> Result<(Vec<VetDisplay>, i64), sqlx::Error> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vets")
        .fetch_one(pool)
        .await?;

    let offset = (page - 1) * page_size;
    let vets = sqlx::query_as::<_, Vet>(
        "SELECT id, first_name, last_name FROM vets ORDER BY last_name, first_name LIMIT ?1 OFFSET ?2",
    )
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut vet_displays = Vec::new();
    for vet in vets {
        let specialties = sqlx::query_as::<_, Specialty>(
            "SELECT s.id, s.name FROM specialties s \
             JOIN vet_specialties vs ON s.id = vs.specialty_id \
             WHERE vs.vet_id = ?1 ORDER BY s.name",
        )
        .bind(vet.id)
        .fetch_all(pool)
        .await?;

        let nr = specialties.len();
        vet_displays.push(VetDisplay {
            id: vet.id,
            first_name: vet.first_name,
            last_name: vet.last_name,
            specialties,
            nr_of_specialties: nr,
        });
    }

    Ok((vet_displays, total.0))
}

pub async fn find_all_json(pool: &SqlitePool) -> Result<Vec<VetDisplay>, sqlx::Error> {
    let vets = sqlx::query_as::<_, Vet>(
        "SELECT id, first_name, last_name FROM vets ORDER BY last_name, first_name",
    )
    .fetch_all(pool)
    .await?;

    let mut vet_displays = Vec::new();
    for vet in vets {
        let specialties = sqlx::query_as::<_, Specialty>(
            "SELECT s.id, s.name FROM specialties s \
             JOIN vet_specialties vs ON s.id = vs.specialty_id \
             WHERE vs.vet_id = ?1 ORDER BY s.name",
        )
        .bind(vet.id)
        .fetch_all(pool)
        .await?;

        let nr = specialties.len();
        vet_displays.push(VetDisplay {
            id: vet.id,
            first_name: vet.first_name,
            last_name: vet.last_name,
            specialties,
            nr_of_specialties: nr,
        });
    }

    Ok(vet_displays)
}
