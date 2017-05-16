//! # Toolbar, Scrollable Text View and File Chooser
//!
//! A simple text file viewer

extern crate gtk;
extern crate iron;
extern crate router;
extern crate rustc_serialize;

use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Write, Read, Seek, SeekFrom, BufReader};

use iron::prelude::*;
use iron::status;

use router::Router;
use rustc_serialize::json;

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


    let window1 = window.clone();
    let chat_view1 = chat_view.clone();
    let text_view1 = text_view.clone();
    log_in_button.connect_clicked(move |_| {

        let file_chooser = gtk::FileChooserDialog::new(
            Some("Open File"), Some(&window1), gtk::FileChooserAction::Open);

        file_chooser.add_buttons(&[
            ("Open", gtk::ResponseType::Ok.into()),
            ("Cancel", gtk::ResponseType::Cancel.into()),
        ]);

        if file_chooser.run() == gtk::ResponseType::Ok.into() {
            let filename = file_chooser.get_filename().unwrap();
            let file = File::open(&filename).unwrap();

            let mut reader = BufReader::new(file);
            let mut contents = String::new();
            let _ = reader.read_to_string(&mut contents);

            chat_view1.get_buffer().unwrap().set_text(&contents);
        }

        file_chooser.destroy();
    });

    let window2 = window.clone();
    let chat_view2 = chat_view.clone();
    let text_view2 = text_view.clone();
    new_chat_button.connect_clicked(move |_| {

        // Not yet implemented
    });

    let window3 = window.clone();
    let chat_view3 = chat_view.clone();
    let text_view3 = text_view.clone();
    switch_chat_button.connect_clicked(move |_| {

        // Not yet implemented
    });

    let window4 = window.clone();
    let chat_view4 = chat_view.clone();
    let text_view4 = text_view.clone();
    send_button.connect_clicked(move |_| {


        let mut current_log_buffer = chat_view4.get_buffer().unwrap();
        let mut start = current_log_buffer.get_start_iter();
        let mut end = current_log_buffer.get_end_iter();
        let mut currentLog = current_log_buffer.get_text(&start, &end, true).unwrap();

        let mut current_message_buffer = text_view4.get_buffer().unwrap();
        start = current_message_buffer.get_start_iter();
        end = current_message_buffer.get_end_iter();
        let mut currentText = current_message_buffer.get_text(&start, &end, true).unwrap();

        let new_log = currentLog.as_str().to_owned() + currentText.as_str() + "\n";

        chat_view4.get_buffer().unwrap().set_text(new_log.as_str());

        text_view4.get_buffer().unwrap().set_text("");
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    window.show_all();
    gtk::main();
}
