
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
    room: String,
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
        room: "".to_string(),
    };

    let data_mutex = Arc::new(Mutex::new(data));
    let (tx, rx) = channel();

    let glade_src = include_str!("rusty_chat.glade");
    let builder = Builder::new();
    builder.add_from_string(glade_src).unwrap();

    let window: gtk::Window = builder.get_object("window").unwrap();

    let log_in_button: gtk::ToolButton = builder.get_object("log_in_button").unwrap();
    let send_button: gtk::ToolButton = builder.get_object("send_button").unwrap();

    let chat_view: gtk::TextView = builder.get_object("chat_view").unwrap();
    let text_view: gtk::TextView = builder.get_object("text_view").unwrap();

    let sent_message_view = text_view.clone();
    let send_button_clone = send_button.clone();
    let send_button_tx = tx.clone();
    let send_button_data_mutex = data_mutex.clone();

    text_view.connect_key_release_event(move |_, key| {

        if key.get_keyval() == 65293 {
            if send_button_clone.get_sensitive(){

                let text_buffer = sent_message_view.get_buffer().unwrap();
                let mut end = text_buffer.get_end_iter();
                text_buffer.backspace(&mut end, true, true);

                send_message(sent_message_view.clone(), send_button_tx.clone(), send_button_data_mutex.clone());
            }
        }

        Inhibit(false)
    });

    let text_view_clone = text_view.clone();
    let tx_clone = tx.clone();
    let data_mutex_clone = data_mutex.clone();

    send_button.connect_clicked(move |_| {

        send_message(text_view_clone.clone(), tx_clone.clone(), data_mutex_clone.clone());
    });

    send_button.set_sensitive(false);
    text_view.set_editable(false);

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let window_clone = window.clone();
    let chat_view_clone = chat_view.clone();
    let tx_clone = tx.clone();
    let data_mutex_clone = data_mutex.clone();
    let log_in_button_clone = log_in_button.clone();

    log_in_button.connect_clicked(move |_| {

        make_log_in_window(tx_clone.clone(),
                            data_mutex_clone.clone(),
                            send_button.clone(),
                            text_view.clone(),
                            chat_view_clone.clone(),
                            window_clone.clone(),
                            log_in_button_clone.clone());
    });

    GLOBAL.with(move |global| {

        *global.borrow_mut() = Some((chat_view.get_buffer().unwrap(), rx))
    });

    let tx_clone = tx.clone();
    let data_mutex_clone = data_mutex.clone();

    window.connect_delete_event(move |_, _| {

        if get_data_username(data_mutex_clone.clone()) != ""{
            log_out(tx_clone.clone(), data_mutex_clone.clone());
        }

        Inhibit(false)
    });

    window.show_all();
    gtk::main();
}

fn log_out(tx: std::sync::mpsc::Sender<std::string::String>,
            data_mutex: Arc<Mutex<MessageData>>){

    println!("Logging out");

    //thread::spawn(move|| {
        set_data_room(data_mutex.clone(), "".to_string());
        send_http_and_write_response("", &tx, &data_mutex.clone());
    //});
}

fn send_message(sent_message_view: gtk::TextView,
                send_button_tx: std::sync::mpsc::Sender<String>,
                send_button_data_mutex: Arc<Mutex<MessageData>>){

    println!("Message sent from Username: {}, Room: {}", get_data_username(send_button_data_mutex.clone()), get_data_room(send_button_data_mutex.clone()));

    let current_message_buffer = sent_message_view.get_buffer().unwrap();
    let start = current_message_buffer.get_start_iter();
    let end = current_message_buffer.get_end_iter();
    let current_text = current_message_buffer.get_text(&start, &end, true).unwrap();
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
}

fn make_log_in_window (tx: std::sync::mpsc::Sender<String>,
                    data_mutex: Arc<Mutex<MessageData>>,
                    send_button: gtk::ToolButton,
                    text_view: gtk::TextView,
                    chat_view: gtk::TextView,
                    main_window: gtk::Window,
                    log_in_button: gtk::ToolButton){

    let log_in_window = gtk::Window::new(gtk::WindowType::Toplevel);

    log_in_window.set_title("Log in");
    log_in_window.set_default_size(400, 200);

    let button = gtk::Button::new_with_label("Log in");
    let window_clone = log_in_window.clone();

    let username_entry = gtk::Entry::new();
    username_entry.set_tooltip_text("Username");
    username_entry.set_text(get_data_username(data_mutex.clone()).as_str());
    let room_entry = gtk::Entry::new();
    room_entry.set_tooltip_text("Room Name");
    room_entry.set_text(get_data_room(data_mutex.clone()).as_str());
    let username_entry_clone = username_entry.clone();
    let room_entry_clone = room_entry.clone();

    if get_data_username(data_mutex.clone()) != ""{

        username_entry_clone.set_editable(false);
    }

    button.connect_clicked(move |_| {

        let username_buffer = username_entry_clone.get_buffer();
        let room_buffer = room_entry_clone.get_buffer();

        if username_buffer.get_text() != "" && room_buffer.get_text() != ""{

            let tx_clone = tx.clone();
            let data_mutex_clone = data_mutex.clone();

            if get_data_room(data_mutex_clone.clone()) != "" {

                log_out(tx_clone.clone(), data_mutex_clone.clone());
                chat_view.get_buffer().unwrap().set_text("");
            }

            set_data_username(data_mutex_clone.clone(), username_buffer.get_text());
            set_data_room(data_mutex_clone.clone(), room_buffer.get_text());

            window_clone.destroy();

            thread::spawn(move|| {

                poll_loop(tx_clone.clone(), data_mutex_clone.clone());
            });

            send_button.set_sensitive(true);
            text_view.set_editable(true);

            let title = format!("Rusty Chat : {}@{}", username_buffer.get_text(), room_buffer.get_text());
            main_window.set_title(title.as_str());
            log_in_button.set_label("Switch Rooms");
        }
    });


    let username_label = gtk::Label::new(Some("Username: "));
    let room_label = gtk::Label::new(Some("Room: "));

    let username_gtkbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    username_gtkbox.add(&username_label);
    username_gtkbox.add(&username_entry);

    let room_gtkbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    room_gtkbox.add(&room_label);
    room_gtkbox.add(&room_entry);

    let gtkbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    gtkbox.add(&username_gtkbox);
    gtkbox.add(&room_gtkbox);
    gtkbox.add(&button);

    gtkbox.set_child_packing(&button, false, true, 0, gtk::PackType::Start);

    log_in_window.add(&gtkbox);

    log_in_window.show_all();
}

fn poll_loop(tx: std::sync::mpsc::Sender<std::string::String>,
            data_mutex: Arc<Mutex<MessageData>>) {
    loop {
        //println!("polling");
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
            "room": data.room,
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

fn get_data_username(data_mutex: Arc<Mutex<MessageData>>)-> String{
    return data_mutex.lock().unwrap().username.clone();
}

fn set_data_username(data_mutex: Arc<Mutex<MessageData>>,
                    username: String){
    let mut data = data_mutex.lock().unwrap();
    data.username = username.clone();
}

fn get_data_room(data_mutex: Arc<Mutex<MessageData>>)-> String{
    return data_mutex.lock().unwrap().room.clone();
}

fn set_data_room(data_mutex: Arc<Mutex<MessageData>>,
                room: String){
    let mut data = data_mutex.lock().unwrap();
    data.room = room.clone();
}
