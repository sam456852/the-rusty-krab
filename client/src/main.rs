//! # Toolbar, Scrollable Text View and File Chooser
//!
//! A simple text file viewer

extern crate gtk;
extern crate hyper;

use std::io::Read;
use hyper::Client;
use gtk::prelude::*;
use gtk::Builder;

pub fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }


    let glade_src = include_str!("rusty_chat.glade");
    let builder = Builder::new();
    builder.add_from_string(glade_src).unwrap();

    let window: gtk::Window = builder.get_object("window").unwrap();

    let log_in_button: gtk::ToolButton = builder.get_object("log_in_button").unwrap();
    let new_chat_button: gtk::ToolButton = builder.get_object("new_chat_button").unwrap();
    let switch_chat_button: gtk::ToolButton = builder.get_object("switch_chat_button").unwrap();
    let send_button: gtk::ToolButton = builder.get_object("send_button").unwrap();

    let chat_view: gtk::TextView = builder.get_object("chat_view").unwrap();
    let text_view: gtk::TextView = builder.get_object("text_view").unwrap();


    log_in_button.connect_clicked(move |_| {
        
        // Not yet implemented
    });

    new_chat_button.connect_clicked(move |_| {

        // Not yet implemented
    });

    switch_chat_button.connect_clicked(move |_| {

        // Not yet implemented
    });

    let chat_window = chat_view.clone();
    let sent_message_view = text_view.clone();
    send_button.connect_clicked(move |_| {

        let current_message_buffer = sent_message_view.get_buffer().unwrap();
        let start = current_message_buffer.get_start_iter();
        let end = current_message_buffer.get_end_iter();
        let current_text = current_message_buffer.get_text(&start, &end, true).unwrap();

        let client = Client::new();
        let mut response = client.post("http://localhost:3000/").body(current_text.as_str()).send().unwrap();
        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();

        chat_window.get_buffer().unwrap().set_text(body.as_str());
        sent_message_view.get_buffer().unwrap().set_text("");
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window.show_all();
    gtk::main();
}
