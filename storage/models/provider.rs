use crate::storage::schema::provider;
use chrono::prelude::*;
use uuid::Uuid;

#[derive(
    Identifiable,
    Debug,
    Serialize,
    Deserialize,
    Associations,
    QueryableByName,
    Queryable,
    PartialEq,
    Clone,
)]
#[primary_key(uid)]
#[table_name = "provider"]
pub struct Provider {
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub enc_data_key: Option<Vec<u8>>,
    pub uid: Uuid,
    pub version: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Insertable)]
#[table_name = "provider"]
pub struct BaseProvider {
    pub enc_data_key: Option<Vec<u8>>,
    pub uid: Uuid,
    pub version: i64,
}
