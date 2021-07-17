use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, source::Source, Sink};
use std::time::Duration;
use websocket::{OwnedMessage};
use websocket::ClientBuilder;
use std::thread;
use std::collections::HashMap;


const ID: &str = "AN_ID";

pub fn main() -> Result<(),String> {
    println!("about to play music");
    //open websocket channel
    let name  = "ws://127.0.0.1:8081";
    let mut client = ClientBuilder::new(name)
        .unwrap()
        .connect_insecure()
        .unwrap();
    //websocket connection
    let (mut receiver, mut sender) = client.split().unwrap();

    let (_stream, handle) = OutputStream::try_default().unwrap();

    let mut sinks:HashMap<String,Sink> = HashMap::new();
    // let sink = Sink::try_new(&handle).unwrap();
    // let sink2 = Sink::try_new(&handle).unwrap();

    let receive_loop = thread::spawn(move || {
        // Receive loop
        for message in receiver.incoming_messages() {
            let message = match message {
                Ok(m) => m,
                Err(e) => {
                    println!("Receive Loop error: {:?}", e);
                    // let _ = tx_1.send(OwnedMessage::Close(None));
                    return;
                }
            };
            match message {
                OwnedMessage::Close(_) => {
                    // Got a close message, so send a close message and return
                    return;
                }
                OwnedMessage::Text(str) => {
                    println!("got message {:?}",str);
                    let parts:Vec<&str> = str.split(":").collect();
                    if parts[0].eq("load") {
                        let file = BufReader::new(File::open(parts[1]).unwrap());
                        let source = Decoder::new(file).unwrap();
                        let sink = Sink::try_new(&handle).unwrap();
                        sink.append(source);
                        sink.pause();

                        sinks.insert(ID.to_string(), sink);
                        let msg = format!("loaded:{:}",ID);
                        sender.send_message(&OwnedMessage::Text(msg));
                    }
                    if parts[0].eq("play") {
                        if let Some(sink) = sinks.get(parts[1]) {
                            sink.play();
                            let msg = format!("played:{:}",ID);
                            sender.send_message(&OwnedMessage::Text(msg));
                        }
                    }
                    if parts[0].eq("pause") {
                        if let Some(sink) = sinks.get(parts[1]) {
                            sink.pause();
                            let msg = format!("paused:{:}",ID);
                            sender.send_message(&OwnedMessage::Text(msg));
                        }
                    }
                    if parts[0].eq("exit") {
                        break;
                    }
                }
                // Say what we received
                _ => println!("Receive Loop: {:?}", message),
            }
        }
    });
    let _ = receive_loop.join();

    Ok(())

}
