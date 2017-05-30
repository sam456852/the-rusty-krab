//! Serde JSON implementation of HTTP Response format

use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Response {
    pub messages: Vec<HashMap<String, String>>,
    pub last_received: i64,
    pub room: String,
}


impl Response {
    /// Constructs a new, empty response
    pub fn new(room: String) -> Self {
        Response{
            messages: vec![],
            last_received: 0,
            room: room,
        }
    }

}
