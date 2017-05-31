//! Rustychat server implementation

extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate iron;
extern crate time;

mod message;
use message::Message;
mod response;

use iron::prelude::*;
use iron::status;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Write, Read, Seek, SeekFrom, BufRead, BufReader};
use std::str::FromStr;
use std::collections::HashMap;

static MESSAGES_PREFIX: &'static str = "messages_";
static TXT_SUFFIX: &'static str = ".txt";
static USERS_PREFIX: &'static str = "users";

fn main() {
    println!("Welcome to Rust chat!");
    // Reset logged in users
    let _ = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(USERS_PREFIX.to_owned() + TXT_SUFFIX);
    // Reset all message logs
    let paths = fs::read_dir("./").unwrap();
    for path in paths {
        let path_name = path.unwrap().path();
        let path_name_str = path_name.to_str().unwrap();
        let expected_path_prefix = "./".to_string() + MESSAGES_PREFIX;
        if path_name_str.len() >= expected_path_prefix.len()
           && &path_name_str[..expected_path_prefix.len()] == expected_path_prefix {
            let _ = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(path_name_str);
        }
    }
    // Iron will spawn a new thread per incoming request
    Iron::new(parse_request).http("localhost:3000").unwrap();
}

/// Parses an incoming HTTP request and returns a corresponding IronResult
/// containing the HTTP response to return
fn parse_request(request: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    request
        .body
        .read_to_string(&mut body)
        .map_err(|e| IronError::new(e, (status::InternalServerError, "Error reading request")))?;
    let m: Message = serde_json::from_str(body.as_str()).unwrap();
    // The incoming message is a logout request
    if m.is_logout() {
        logout(m);
        // Successful logout returns 200 Ok
        Ok(Response::with((status::Ok, "")))
    }
    // The incoming message is a login request
    else if m.is_login() {
        let response = login(m);
        // If username already taken, return 401 Unauthorized
        if response.messages.is_empty() && response.last_received == 0 {
            Ok(Response::with((status::Unauthorized, "")))
        }
        // If login successful, return 200 Ok
        else {
            Ok(Response::with((status::Ok, serde_json::to_string(&response).unwrap())))
        }
    }
    // The incoming message is a poll
    else if m.is_poll() {
        let response = long_poll(m);
        // If last message was sent by same user, return 204 No Content
        if response.messages.is_empty() {
            Ok(Response::with((status::NoContent, "")))
        }
        // If messages to return, return 200 Ok
        else {
            Ok(Response::with((status::Ok, serde_json::to_string(&response).unwrap())))
        }
    }
    // The incoming message is simply a message
    else {
        let response = write_log(m);
        // Sucessful message write responds 200 Ok
        Ok(Response::with((status::Ok, serde_json::to_string(&response).unwrap())))
    }
}

/// Given a logout request containing a username, removes the user from the users log file
fn logout(logout: Message) {
    let users_name = USERS_PREFIX.to_owned() + TXT_SUFFIX;
    let mut users_file = OpenOptions::new()
                        .read(true)
                        .open(users_name)
                        .unwrap();
    users_file.seek(SeekFrom::Start(0)).unwrap();
    let users_reader = BufReader::new(users_file);
    let mut users_to_keep = vec![];
    for line in users_reader.lines() {
        let username = line.unwrap();
        if username != logout.username {
            users_to_keep.push(username);
        }
    }
    let users_name_write = USERS_PREFIX.to_owned() + TXT_SUFFIX;
    let mut users_file_write = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(users_name_write)
                        .unwrap();
    users_file_write.seek(SeekFrom::Start(0)).unwrap();
    let mut user_log_entry = String::new();
    for username in users_to_keep {
        user_log_entry.push_str((username + "\n").as_str());
    }
    users_file_write.write_all(user_log_entry.as_bytes()).unwrap();
}

/// Given a login request with a username and room, adds the user
/// to the users log file and returns the messages of the specified
/// room log file in the returned Response. Returns empty Response
/// if user is already present in the log.
fn login(login: Message) -> response::Response {
    let mut last_received = time::get_time().sec;
    let mut response = response::Response::new();
    let users_name = USERS_PREFIX.to_owned() + TXT_SUFFIX;
    let mut users_file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .read(true)
                        .open(users_name)
                        .unwrap();
    users_file.seek(SeekFrom::Start(0)).unwrap();
    let users_reader = BufReader::new(users_file);
    for line in users_reader.lines() {
        let username = line.unwrap();
        if username == login.username {
            return response;
        }
    }
    let users_name_write = USERS_PREFIX.to_owned() + TXT_SUFFIX;
    let mut users_file_write = OpenOptions::new()
                        .append(true)
                        .open(users_name_write)
                        .unwrap();
    users_file_write.seek(SeekFrom::Start(0)).unwrap();
    let user_log_entry = login.username + "\n";
    users_file_write.write_all(user_log_entry.as_bytes()).unwrap();
    let messages_name = MESSAGES_PREFIX.to_owned() + login.room.as_str() + TXT_SUFFIX;
    let mut messages_file = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .read(true)
                            .open(messages_name)
                            .unwrap();
    messages_file.seek(SeekFrom::Start(0)).unwrap();
    let messages_reader = BufReader::new(messages_file);
    for line in messages_reader.lines() {
        let l = line.unwrap();
        let line_vec: Vec<&str> = l.split("\t").collect();
        let line_timestamp = i64::from_str(line_vec[0]).unwrap();
        if line_timestamp > login.last_received {
            last_received = line_timestamp;
            let mut message_map = HashMap::new();
            message_map.insert("username".to_string(), line_vec[1].to_string());
            message_map.insert("body".to_string(), line_vec[2].to_string());
            response.messages.push(message_map);
        }
    }
    response.last_received = last_received;
    return response;
}

/// Given a poll request, waits for a message newer than the last_received field to
/// be written to the room log and returns that message once it is written.
/// If a message by the same user is written, returns an empty Response.
fn long_poll(poll: Message) -> response::Response {
    let mut last_received = time::get_time().sec;
    let mut response = response::Response::new();
    let mut saw_self = false;
    while response.messages.is_empty() && !saw_self {
        // Long poll timeout is 5 seconds
        if time::get_time().sec - last_received > 5 {
            break;
        }
        let messages_name = MESSAGES_PREFIX.to_owned() + poll.room.as_str() + TXT_SUFFIX;
        let mut file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .read(true)
                        .open(messages_name)
                        .unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let l = line.unwrap();
            let line_vec: Vec<&str> = l.split("\t").collect();
            let line_timestamp = i64::from_str(line_vec[0]).unwrap();
            if line_timestamp > poll.last_received {
                last_received = line_timestamp;
                if line_vec[1] == poll.username {
                    saw_self = true;
                    break;
                }
                let mut message_map = HashMap::new();
                message_map.insert("username".to_string(), line_vec[1].to_string());
                message_map.insert("body".to_string(), line_vec[2].to_string());
                response.messages.push(message_map);
            }
        }
    }
    response.last_received = last_received;
    return response;
}

/// Writes a new message to the respective room log and returns the appropriate Response
fn write_log(mut new_message: Message) -> response::Response {
    let messages_name = MESSAGES_PREFIX.to_owned() + new_message.room.as_str() + TXT_SUFFIX;
    let mut file = OpenOptions::new()
                    .read(true)
                    .create(true)
                    .append(true)
                    .open(messages_name)
                    .unwrap();
    if new_message.body.len() != 0 {
        new_message.body.push_str("\n");
    }
    let log_time = time::get_time().sec;
    let log_string = format!("{}\t{}\t{}", log_time, new_message.username, new_message.body);
    print!("{}", log_string);

    file.write_all(log_string.as_bytes()).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut response = response::Response::new();
    response.last_received = log_time;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let l = line.unwrap();
        let line_vec: Vec<&str> = l.split("\t").collect();
        let line_timestamp = i64::from_str(line_vec[0]).unwrap();
        if line_timestamp > new_message.last_received {
            let mut message_map = HashMap::new();
            message_map.insert("username".to_string(), line_vec[1].to_string());
            message_map.insert("body".to_string(), line_vec[2].to_string());
            response.messages.push(message_map);
        }
    }
    return response;
}


#[cfg(test)]
mod server_tests {
    use std::fs::OpenOptions;
    use std::collections::HashMap;
    use std::io::{Write, Seek, SeekFrom};
    use super::message::Message;
    use super::response::Response;
    use super::login;
    use super::long_poll;
    use super::write_log;

    #[test]
    fn login_test() {
        let mut users_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open("users.txt")
                .unwrap();
        users_file.seek(SeekFrom::Start(0)).unwrap();
        let my_message = Message {
            username: "Klay".to_string(),
            body: "".to_string(),
            last_received: 0,
            room: "login_test".to_string(),
        };

        let mut test_file_message = String::new();
        test_file_message.push_str(("5\tAnotherUser\tHowdy".to_string()).as_str());
        let test_room_file_name = "messages_login_test.txt";
        let mut test_room_file = OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .create(true)
                            .open(test_room_file_name)
                            .unwrap();

        test_room_file.write_all(test_file_message.as_bytes()).unwrap();
        test_room_file.seek(SeekFrom::Start(0)).unwrap();

        let mut expected_response = Response::new();
        let mut message_map = HashMap::new();
        message_map.insert("username".to_string(), "AnotherUser".to_string());
        message_map.insert("body".to_string(), "Howdy".to_string());
        expected_response.messages.push(message_map);
        let actual_response = login(my_message);
        expected_response.last_received = actual_response.last_received;
        assert_eq!(actual_response, expected_response);
    }

    #[test]
    fn long_poll_test() {
        let mut users_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open("users.txt")
                .unwrap();
        users_file.seek(SeekFrom::Start(0)).unwrap();
        let my_message = Message {
            username: "Klay".to_string(),
            body: "".to_string(),
            last_received: 4,
            room: "long_poll_test".to_string(),
        };

        let mut test_file_message = String::new();
        test_file_message.push_str(("2\tAnotherUser\tHowdy\n".to_string()).as_str());
        test_file_message.push_str(("10\tAnotherUser\tyo".to_string()).as_str());
        let test_room_file_name = "messages_long_poll_test.txt";
        let mut test_room_file = OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .create(true)
                            .open(test_room_file_name)
                            .unwrap();
        test_room_file.write_all(test_file_message.as_bytes()).unwrap();
        test_room_file.seek(SeekFrom::Start(0)).unwrap();
        let mut expected_response = Response::new();
        let mut message_map = HashMap::new();
        message_map.insert("username".to_string(), "AnotherUser".to_string());
        message_map.insert("body".to_string(), "yo".to_string());
        expected_response.messages.push(message_map);
        let actual_response = long_poll(my_message);
        expected_response.last_received = actual_response.last_received;
        assert_eq!(actual_response, expected_response);
    }

    #[test]
    fn write_log_test() {
        let mut users_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open("users.txt")
                .unwrap();
        users_file.seek(SeekFrom::Start(0)).unwrap();
        let my_message = Message {
            username: "Klay".to_string(),
            body: "hi its klay".to_string(),
            last_received: 4,
            room: "write_log_test".to_string(),
        };

        let mut test_file_message = String::new();
        test_file_message.push_str(("2\tAnotherUser\tHowdy\n".to_string()).as_str());
        test_file_message.push_str(("10\tAnotherUser\tyo\n".to_string()).as_str());
        let test_room_file_name = "messages_write_log_test.txt";
        let mut test_room_file = OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .create(true)
                            .open(test_room_file_name)
                            .unwrap();
        test_room_file.write_all(test_file_message.as_bytes()).unwrap();
        test_room_file.seek(SeekFrom::Start(0)).unwrap();
        let mut expected_response = Response::new();
        let mut message_map = HashMap::new();
        let mut message_map2 = HashMap::new();
        message_map.insert("username".to_string(), "AnotherUser".to_string());
        message_map.insert("body".to_string(), "yo".to_string());
        expected_response.messages.push(message_map);
        message_map2.insert("username".to_string(), "Klay".to_string());
        message_map2.insert("body".to_string(), "hi its klay".to_string());
        expected_response.messages.push(message_map2);
        let actual_response = write_log(my_message);
        expected_response.last_received = actual_response.last_received;
        assert_eq!(actual_response, expected_response);
    }
}
