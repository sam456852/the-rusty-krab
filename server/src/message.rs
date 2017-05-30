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

#[cfg(test)]
mod message_tests {
    use super::Message;

    #[test]
    fn is_poll_test() {
        let test_message = Message {
            username: "Stephen".to_string(),
            body: "".to_string(),
            last_received: 10,
            room: "test_room".to_string(),
        };

        assert_eq!(test_message.is_poll(), true);   
    }

    #[test]
    fn is_login_test() {
        let test_message = Message {
            username: "Kevin".to_string(),
            body: "".to_string(),
            last_received: 0,
            room: "test_room".to_string(),
        };

        assert_eq!(test_message.is_login(), true);   
    }

    #[test]
    fn is_logout_test() {
        let test_message = Message {
            username: "Draymond".to_string(),
            body: "".to_string(),
            last_received: 0,
            room: "".to_string(),
        };

        assert_eq!(test_message.is_logout(), true);   
    }
}