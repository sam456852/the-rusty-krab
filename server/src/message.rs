//! Serde JSON implementation of received messages

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub username: String,
    pub body: String,
    pub last_received: i64,
    pub room: String,
}

impl Message {
    /// Returns whether or not the message is a poll request
    pub fn is_poll(&self) -> bool {
        return self.body == "".to_string() && self.last_received != 0 && self.room != "".to_string();
    }

    /// Returns whether or not the message is a login request
    pub fn is_login(&self) -> bool {
        return self.body == "".to_string() && self.last_received == 0 && self.room != "".to_string();
    }

    /// Returns whether or not the message is a logout request
    pub fn is_logout(&self) -> bool {
        return self.body == "".to_string() && self.last_received == 0 && self.room == "".to_string()
    }
}
