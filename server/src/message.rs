use serde_json;
use serde_json::{Value, Error};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub username: String,
    pub body: String,
    pub last_received: i64,
}


impl Message {
    // let v: Value = serde_json::from_str(json_string)?; //helps make a json

    pub fn is_poll(&self) -> bool {
        return self.body == "".to_string();
    }
}
