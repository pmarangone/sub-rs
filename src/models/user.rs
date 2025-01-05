// use crate::error_handling::empty_string_as_none;
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneralParams {
    // #[serde(default, deserialize_with = "empty_string_as_none")]
    pub name: String,
    pub surname: String,
    pub description: String,
    pub age: f32,
}
