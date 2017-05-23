
extern crate gtk;
extern crate glib;
extern crate hyper;
#[macro_use]
extern crate serde_json;

use std::io::Read;
use std::cell::RefCell;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use hyper::Client;
use gtk::prelude::*;
use gtk::Builder;

/// A structure containing all the data needed to send a message to the server
pub struct MessageData {
	last_received: i64,
    username: String,
}

pub fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }
    
    let data = MessageData {
        last_received: 0,
        username: "Chris".to_owned(),
    };

    let data_mutex = Arc::new(Mutex::new(data));

    let (tx, rx) = channel();

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

    let sent_message_view = text_view.clone();
    let send_button_tx = tx.clone();
    let chat_window = chat_view.clone();
    let send_button_data_mutex = data_mutex.clone();

    send_button.connect_clicked(move |_| {
        let current_message_buffer = sent_message_view.get_buffer().unwrap();
        let start = current_message_buffer.get_start_iter();
        let end = current_message_buffer.get_end_iter();
        let current_text = current_message_buffer.get_text(&start, &end, true).unwrap();
        let chat_window_buffer = chat_window.get_buffer().unwrap();
        let mut chat_window_end = chat_window_buffer.get_end_iter();
        let send_button_data = send_button_data_mutex.lock().unwrap();
        chat_window_buffer.insert(
            &mut chat_window_end, 
            &format!("{}: {}",  current_text, send_button_data.username)
        );
        sent_message_view.get_buffer().unwrap().set_text("");
        let message_thread_tx = send_button_tx.clone();
        let message_thread_data = send_button_data_mutex.clone();
        thread::spawn(move || {
            send_http_and_write_response(
                &current_text, 
                &message_thread_tx, 
                &message_thread_data
            );
        });
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    create_poll_thread(chat_view, tx, rx, data_mutex);


    window.show_all();
    gtk::main();
}

fn create_poll_thread(chat_view: gtk::TextView, 
                    tx: std::sync::mpsc::Sender<String>, 
                    rx: std::sync::mpsc::Receiver<String>, 
                    data_mutex: Arc<Mutex<MessageData>>) {

    // put TextBuffer and receiver in thread local storage
    // This seems wonky, but it's how the gtk tutorial does it
    GLOBAL.with(move |global| {
        *global.borrow_mut() = Some((chat_view.get_buffer().unwrap(), rx))
    });

    thread::spawn(move|| {
        poll_loop(tx, data_mutex.clone());
    });
}

/// Polls the server to see if new messages have been posted
/// Possibly should be switched to server pushes or 
/// [long polling](https://xmpp.org/extensions/xep-0124.html#technique)
fn poll_loop(tx: std::sync::mpsc::Sender<std::string::String>, 
            data_mutex: Arc<Mutex<MessageData>>) {
    loop {
        thread::sleep(Duration::from_millis(500));
        send_http_and_write_response("", &tx, &data_mutex);
    }
}

fn send_http_and_write_response(text: &str, 
                                tx: &std::sync::mpsc::Sender<std::string::String>, 
                                data_mutex: &Arc<Mutex<MessageData>>) {
    let client = Client::new();
    let json = make_json(text, data_mutex.clone());
    let mut response = client.post("http://localhost:3000/").body(&json).send().unwrap();
    let mut data = data_mutex.lock().unwrap();
    data.last_received = data.last_received + 1;
    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();
    if !body.is_empty() {
        tx.send(body).unwrap();
        glib::idle_add(receive);
    }
}

fn make_json(text: &str, data_mutex: Arc<Mutex<MessageData>>) -> String {
    let data = data_mutex.lock().unwrap();
    json!({
			"username": data.username,
			"body": text,
			"last_received": data.last_received,
		}).to_string()
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

// declare a new thread local storage key (Again this is how the example did it)
thread_local!(
    static GLOBAL: RefCell<Option<(gtk::TextBuffer, Receiver<String>)>> 
        = RefCell::new(None)
);
