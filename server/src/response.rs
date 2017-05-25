use serde_json;
use serde_json::{Value, Error};

use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub messages: Vec<HashMap<String, String>>,
    pub last_received: i64,
}


impl Response {

    pub fn new() -> Self {
        Response{
            messages: vec![],
            last_received: 0,
        }
    }

}
