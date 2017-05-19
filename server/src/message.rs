use serde_json;
use serde_json::{Value, Error};


pub struct Message {
	username: String,
	body: String,
	last_received: i64,
}


impl Message {
	// let v: Value = serde_json::from_str(json_string)?; //helps make a json
	pub fn new(v: Value) -> Self {
		Message {
			username: v["username"].as_str().unwrap().to_string(),
			body: v["body"].as_str().unwrap().to_string(),
			last_received: v["last_received"].as_i64().unwrap(),
		}
	}

	pub fn to_json(&self) -> Value {
		json!({
			"username": self.username,
			"body": self.body,
			"last_received": self.last_received,
		})
	}

	pub fn is_poll(&self) -> bool {
		return self.body == "".to_string();
	}
}
