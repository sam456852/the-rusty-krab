# the-rusty-krab
The Rusty Krab's Rust chat client

## Requirements
- The client uses gtk-rs for its UI. This crate expects GTK+ to be installed on your system. Here's a setup guide if you do not have it installed: http://gtk-rs.org/docs/requirements.html

## How to use
- Both the server and the client are setup to listen on `localhost:3000`.
1. Run an instance of the server using `cargo run` while in the `server` directory
2. Run an instance of the client using `cargo run` while in the `client` directory
3. When the client launches, press the "Log in" button at the top. Provide a username and room to join (neither can be blank.) Any room name suffices, the server will create the room if it does not already exist.
4. Type into the chatbox at the bottom and press your Enter key to send a message.
