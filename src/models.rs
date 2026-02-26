use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Owner {
    pub id: Option<i64>,
    pub first_name: String,
    pub last_name: String,
    pub address: String,
    pub city: String,
    pub telephone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerListItem {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub address: String,
    pub city: String,
    pub telephone: String,
    pub pets: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Pet {
    pub id: Option<i64>,
    pub name: String,
    pub birth_date: Option<NaiveDate>,
    pub type_id: i64,
    pub owner_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetWithVisits {
    pub id: i64,
    pub name: String,
    pub birth_date: String,
    #[serde(rename = "type")]
    pub pet_type: String,
    pub visits: Vec<VisitDisplay>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PetType {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Visit {
    pub id: Option<i64>,
    pub pet_id: i64,
    pub visit_date: Option<NaiveDate>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisitDisplay {
    pub id: i64,
    pub date: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Vet {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VetDisplay {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub specialties: Vec<Specialty>,
    pub nr_of_specialties: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Specialty {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct OwnerForm {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub telephone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PetForm {
    pub name: Option<String>,
    pub birth_date: Option<String>,
    #[serde(rename = "type")]
    pub pet_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VisitForm {
    pub date: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FindOwnerQuery {
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    pub page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
}

pub struct ValidationErrors {
    pub errors: Vec<(String, String)>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, field: &str, message: &str) {
        self.errors.push((field.to_string(), message.to_string()));
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

}
