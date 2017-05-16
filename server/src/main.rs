extern crate iron;

use iron::prelude::*;
use iron::status;
use std::fs::OpenOptions;
use std::io::{Write, Read, Seek, SeekFrom};

fn main() {
    println!("Welcome to Rust chat!");
    Iron::new(parse_request).http("localhost:3000").unwrap();
}

fn parse_request(request: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    request
        .body
        .read_to_string(&mut body)
        .map_err(|e| IronError::new(e, (status::InternalServerError, "Error reading request")))?;
    let response_body = write_log(body);
    Ok(Response::with((status::Ok, response_body)))
}

fn write_log(mut new_message: String) -> String {
    let mut file = OpenOptions::new()
                    .read(true)
                    .create(true)
                    .append(true)
                    .open("messages.txt")
                    .unwrap();
    print!("{}",new_message);
    // Ignore the message if it's just a poll from the client
    if new_message.len() != 0 {
        new_message.push_str("\n");
    }
    file.write_all(new_message.as_bytes()).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut messages = String::new();
    file.read_to_string(&mut messages).unwrap();
    return messages;
}
