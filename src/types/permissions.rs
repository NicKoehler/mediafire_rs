use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Permissions {
    pub value: String,
    pub explicit: String,
    pub read: String,
    pub write: String,
}
