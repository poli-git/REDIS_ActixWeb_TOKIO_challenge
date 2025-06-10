use crate::schema::providers;
use chrono::prelude::Utc;
use chrono::prelude::*;
use diesel::Queryable;
use serde::Deserialize;
use std::convert::From;

use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct ListProvider {
    pub data: Vec<Provider>,
}

#[derive(Debug, Serialize, Deserialize, Identifiable, Queryable, PartialEq, Clone)]
#[diesel(table_name = providers)]
#[diesel(primary_key(providers_id))]
pub struct Provider {
    pub providers_id: Uuid,
    #[serde(rename = "name")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(rename = "description")]
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(rename = "url")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub url: String,
    #[serde(rename = "is_active")]
    pub is_active: bool,
    #[serde(rename = "created_at")]
    pub created_at: chrono::NaiveDateTime,
    #[serde(rename = "updated_at")]
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Insertable)]
#[diesel(table_name = providers)]
pub struct NewProvider {
    pub providers_id: Uuid,
    pub name: String,
    pub description: String,
    pub url: String,
    pub is_active: bool,
}

impl From<NewProvider> for Provider {
    fn from(new_provider: NewProvider) -> Self {
        let now: NaiveDateTime = Utc::now().naive_utc();

        Provider {
            providers_id: new_provider.providers_id,
            name: new_provider.name,
            description: new_provider.description,
            url: new_provider.url,
            is_active: new_provider.is_active,
            created_at: now,
            updated_at: now,
        }
    }
}
