
extern crate gtk;
extern crate glib;
extern crate hyper;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::io::Read;
use std::cell::RefCell;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::sync::{Arc, Mutex};

use hyper::Client;
use gtk::prelude::*;
use gtk::Builder;

/// A structure containing all the data needed to send a message to the server
pub struct MessageData {
	last_received: i64,
    username: String,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
	username: String,
	body: String,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
	messages: Vec<Message>,
	last_received: i64,
}

pub fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let data = MessageData {
        last_received: 0,
        username: "".to_string(),
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

    let sent_message_view = text_view.clone();
    let send_button_tx = tx.clone();
    // let chat_window = chat_view.clone();
    let send_button_data_mutex = data_mutex.clone();

    send_button.connect_clicked(move |_| {
        let current_message_buffer = sent_message_view.get_buffer().unwrap();
        let start = current_message_buffer.get_start_iter();
        let end = current_message_buffer.get_end_iter();
        let current_text = current_message_buffer.get_text(&start, &end, true).unwrap();
        // let send_button_data = send_button_data_mutex.lock().unwrap();
        // let chat_window_buffer = chat_window.get_buffer().unwrap();
        // Code to make the chat automatically append its own messages
        // let mut chat_window_end = chat_window_buffer.get_end_iter();
        // chat_window_buffer.insert(
        //     &mut chat_window_end, 
        //     &format!("{}: {}\n",  send_button_data.username, current_text)
        // );
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

    GLOBAL.with(move |global| {

        *global.borrow_mut() = Some((chat_view.get_buffer().unwrap(), rx))
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    log_in_button.connect_clicked(move |_| {

        make_log_in_window(tx.clone(), data_mutex.clone());
    });

    new_chat_button.connect_clicked(move |_| {

        // TODO
    });

    switch_chat_button.connect_clicked(move |_| {

        // TODO
    });


    window.show_all();
    gtk::main();
}

fn make_log_in_window (tx: std::sync::mpsc::Sender<String>,
                    data_mutex: Arc<Mutex<MessageData>>){

    let window = gtk::Window::new(gtk::WindowType::Toplevel);

    window.set_title("Log in");
    window.set_default_size(400, 200);

    let button = gtk::Button::new_with_label("Log in");
    let window_clone = window.clone();

    let username_entry = gtk::Entry::new();
    username_entry.set_tooltip_text("Username");
    username_entry.set_text("Username");
    let password_entry = gtk::Entry::new();
    password_entry.set_tooltip_text("Password");
    password_entry.set_text("Password");
    password_entry.set_visibility(false);

    let data_mutex_clone = data_mutex.clone();
    let username_entry_clone = username_entry.clone();

    button.connect_clicked(move |_| {

        let username_buffer = username_entry_clone.get_buffer();
        let username_text = username_buffer.get_text();

        let mut data = data_mutex_clone.lock().unwrap();
        data.username = username_text;
        window_clone.destroy();

        let tx_clone = tx.clone();
        let data_mutex_clone = data_mutex.clone();

        thread::spawn(move|| {
            poll_loop(tx_clone, data_mutex_clone);
        });
    });


    let gtkbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    gtkbox.add(&username_entry);
    gtkbox.add(&password_entry);
    gtkbox.add(&button);

    gtkbox.set_child_packing(&button, false, true, 0, gtk::PackType::Start);

    window.add(&gtkbox);

    window.show_all();
}


fn poll_loop(tx: std::sync::mpsc::Sender<std::string::String>,
            data_mutex: Arc<Mutex<MessageData>>) {
    loop {
        send_http_and_write_response("", &tx, &data_mutex.clone());
    }
}

fn send_http_and_write_response(text: &str,
                                tx: &std::sync::mpsc::Sender<std::string::String>,
                                data_mutex: &Arc<Mutex<MessageData>>) {
    let client = Client::new();
    let json = make_json(text, data_mutex.clone());
    let mut response = client.post("http://localhost:3000/").body(&json).send().unwrap();
    let mut body = String::new();
    response.read_to_string(&mut body).unwrap();
    if body.is_empty() {
        return;
    }
    let r: Response = serde_json::from_str(&body).expect("Something wrong with the JSON");
    let mut new_messages = "".to_owned();
    for message in r.messages {
        if !message.body.is_empty() {
            new_messages += &format!("{}: {}\n", message.username, message.body);
        }
    }
    let mut data = data_mutex.lock().unwrap();
    data.last_received = r.last_received;
    if !new_messages.is_empty() {
        tx.send(new_messages).unwrap();
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
                let mut chat_window_end = buf.get_end_iter();
                buf.insert(
                    &mut chat_window_end, 
                    &text
                );
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
