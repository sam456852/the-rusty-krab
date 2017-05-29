#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate iron;
extern crate time;

use iron::prelude::*;
use iron::status;
use std::fs::{OpenOptions, File};
use std::io::{Write, Read, Seek, SeekFrom, BufRead, BufReader};
use serde_json::{Value, Error};
use std::str::FromStr;
use std::collections::HashMap;
use std::thread;


mod message;
use message::Message;
mod response;

static MESSAGES_PREFIX: &'static str = "messages_";
static TXT_SUFFIX: &'static str = ".txt";
static USERS_PREFIX: &'static str = "users";


fn main() {
    println!("Welcome to Rust chat!");
    // Iron will already spawn a new thread per incoming request
    Iron::new(parse_request).http("localhost:3000").unwrap();
}

fn parse_request(request: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    request
        .body
        .read_to_string(&mut body)
        .map_err(|e| IronError::new(e, (status::InternalServerError, "Error reading request")))?;
    let m: Message = serde_json::from_str(body.as_str()).unwrap();
    if m.is_logout() {
        logout(m);
        Ok(Response::with((status::Ok, "")))
    }
    else if m.is_login() {
        let response = login(m);
        if response.messages.is_empty() && response.last_received == 0 {
            Ok(Response::with((status::Unauthorized, "")))
        }
        else {
            Ok(Response::with((status::Ok, serde_json::to_string(&response).unwrap())))
        }
    }
    else if m.is_poll() {
        let response = long_poll(m);
        if response.messages.is_empty() {
            Ok(Response::with((status::NoContent, "")))
        }
        else {
            Ok(Response::with((status::Ok, serde_json::to_string(&response).unwrap())))
        }

    }
    else {
        let response = write_log(m);
        Ok(Response::with((status::Ok, serde_json::to_string(&response).unwrap())))
    }
}

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

fn long_poll(poll: Message) -> response::Response {
    let mut last_received = time::get_time().sec;
    let mut response = response::Response::new();
    let mut saw_self = false;
    while response.messages.is_empty() && !saw_self {
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
