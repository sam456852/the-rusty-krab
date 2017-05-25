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

fn main() {
    println!("Welcome to Rust chat!");

    let file = OpenOptions::new().write(true).truncate(true).open("messages.txt");

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
    if m.is_poll() {
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

fn long_poll(poll: Message) -> response::Response {
    let log_time = time::get_time().sec;
    let mut response = response::Response::new(log_time);
    let mut saw_self = false;
    while response.messages.is_empty() && !saw_self {
        let mut file = OpenOptions::new()
                        .read(true)
                        .open("messages.txt")
                        .unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let l = line.unwrap();
            let line_vec: Vec<&str> = l.split("\t").collect();
            let line_timestamp = i64::from_str(line_vec[0]).unwrap();
            if line_timestamp > poll.last_received {
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
    return response;
}

fn write_log(mut new_message: Message) -> response::Response {
    let mut file = OpenOptions::new()
                    .read(true)
                    .create(true)
                    .append(true)
                    .open("messages.txt")
                    .unwrap();
    if new_message.body.len() != 0 {
        new_message.body.push_str("\n");
    }
    let log_time = time::get_time().sec;
    let log_string = format!("{}\t{}\t{}", log_time, new_message.username, new_message.body);
    print!("{}", log_string);

    file.write_all(log_string.as_bytes()).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut response = response::Response::new(log_time);
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
