//! # Toolbar, Scrollable Text View and File Chooser
//!
//! A simple text file viewer

extern crate gtk;
extern crate glib;
extern crate hyper;

use std::io::Read;
use std::cell::RefCell;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

use hyper::Client;
use gtk::prelude::*;
use gtk::Builder;

const  POLL_SLEEP_TIME: u64 = 500;

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
        
        // TODO
    });

    new_chat_button.connect_clicked(move |_| {

        // TODO
    });

    switch_chat_button.connect_clicked(move |_| {

        // TODO
    });

    let chat_window = chat_view.clone();
    let sent_message_view = text_view.clone();
    /// Sends the message to the server and updates the chat view accordingly
    send_button.connect_clicked(move |_| {

        let current_message_buffer = sent_message_view.get_buffer().unwrap();
        let start = current_message_buffer.get_start_iter();
        let end = current_message_buffer.get_end_iter();
        let current_text = current_message_buffer.get_text(&start, &end, true).unwrap();
        let body = send_http_to_server(current_text.as_str());
        chat_window.get_buffer().unwrap().set_text(body.as_str());
        sent_message_view.get_buffer().unwrap().set_text("");
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    
    create_poll_thread(chat_view);


    window.show_all();
    gtk::main();
}

fn create_poll_thread(chat_view: gtk::TextView) {
    let (tx, rx) = channel();

    // put TextBuffer and receiver in thread local storage
    // This seems wonky, but it's how the gtk tutorial does it
    GLOBAL.with(move |global| {
        *global.borrow_mut() = Some((chat_view.get_buffer().unwrap(), rx))
    });

    thread::spawn(move|| {
        poll_loop(tx);
    });
}

/// Polls the server to see if new messages have been posted
/// Possibly should be switched to server pushes or 
/// [long polling](https://xmpp.org/extensions/xep-0124.html#technique)
fn poll_loop(tx: std::sync::mpsc::Sender<std::string::String>) {
    loop {
        thread::sleep(Duration::from_millis(POLL_SLEEP_TIME));
        let body = send_http_to_server("");
        tx.send(body).unwrap();

        // receive will be run on the main thread
        // TODO: only run this step if somehting has changed
        glib::idle_add(receive);
    }
}

// Writes the most recent chat transcript to the chat window
fn receive() -> glib::Continue {
    GLOBAL.with(|global| {
        if let Some((ref buf, ref rx)) = *global.borrow() {
            if let Ok(text) = rx.try_recv() {
                buf.set_text(&text);
            }
        }
    });
    glib::Continue(false)
}

// Sends a post request containing `text` to the chat server
fn send_http_to_server(text: &str) -> String {
    let client = Client::new();
    let mut response = client.post("http://localhost:3000/").body(text).send().unwrap();
    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();
    body
}

// declare a new thread local storage key (Again this is how the example did it)
thread_local!(
    static GLOBAL: RefCell<Option<(gtk::TextBuffer, Receiver<String>)>> 
        = RefCell::new(None)
);
