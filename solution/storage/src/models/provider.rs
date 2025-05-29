use chrono::prelude::*;
use diesel::Selectable;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name=crate::schema::providers)]
pub struct Provider {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub url: String,
    pub is_active: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct NewProvider {
    pub id: Uuid,
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
