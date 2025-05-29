use crate::schema::providers;
use chrono::prelude::Utc;
use std::convert::From;

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct Provider {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub url: String,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NewProvider {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub url: String,
}

impl From<NewProvider> for Provider {
    fn from(provider: NewProvider) -> Self {
        let now = Utc::now().naive_utc();

        Provider {
            id: provider.id,
            name: provider.name,
            description: provider.description,
            url: provider.url,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}
