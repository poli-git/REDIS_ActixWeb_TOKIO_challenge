use crate::schema::providers;
use chrono::prelude::Utc;
use chrono::prelude::*;
use serde::Serialize;
use std::convert::From;
use uuid::Uuid;
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Associations,
    Identifiable,
    Insertable,
    Queryable,
    PartialEq,
    Clone,
)]
#[table_name = "providers"]
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
    fn from(new_provider: NewProvider) -> Self {
        let now: NaiveDateTime = Utc::now().naive_utc();

        Provider {
            id: new_provider.id,
            name: new_provider.name,
            description: new_provider.description,
            url: new_provider.url,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}
