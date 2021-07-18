use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, source::Source, Sink};
use std::time::Duration;
use websocket::{OwnedMessage};
use websocket::ClientBuilder;
use std::thread;
use std::collections::HashMap;
use serde_json::{json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct AudioMessage {
    id:String,
    #[serde(rename = "type")]
    pub type_:String,
    command:String,
    resource:String,
}

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

    sender.send_message(&OwnedMessage::Text(json!(AudioMessage {
                            id: "".to_string(),
                            type_: "AUDIO".to_string(),
                            command: "connected".to_string(),
                            resource: "audio".to_string()
                        }).to_string()));

    let receive_loop = thread::spawn(move || {
        // Receive loop
        println!("waiting for messages");
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
                    // let v: Value = serde_json::from_str(str.as_str())?;
                    let msg:AudioMessage = serde_json::from_str(str.as_str()).unwrap();
                    println!("incoming message {:?}",msg);
                    if msg.command.eq("load") {
                        let file = BufReader::new(File::open(msg.resource).unwrap());
                        let source = Decoder::new(file).unwrap();
                        let sink = Sink::try_new(&handle).unwrap();
                        sink.append(source);
                        sink.pause();
                        sinks.insert(ID.to_string(), sink);
                        sender.send_message(&OwnedMessage::Text(json!(AudioMessage {
                            id: "".to_string(),
                            type_: "AUDIO".to_string(),
                            command: "played".to_string(),
                            resource: ID.to_string()
                        }).to_string()));
                        continue
                    }
                    if msg.command.eq("play") {
                        if let Some(sink) = sinks.get(msg.resource.as_str()) {
                            sink.play();
                            let msg:AudioMessage = AudioMessage {
                                id: "".to_string(),
                                type_: "AUDIO".to_string(),
                                command: "played".to_string(),
                                resource: ID.to_string()
                            };
                            // let msg = format!("played:{:}",ID);
                            sender.send_message(&OwnedMessage::Text(json!(msg).to_string()));
                            continue
                        }
                    }
                    if msg.command.eq("pause") {
                        if let Some(sink) = sinks.get(msg.resource.as_str()) {
                            sink.pause();
                            let msg:AudioMessage = AudioMessage {
                                id: "".to_string(),
                                type_: "AUDIO".to_string(),
                                command: "paused".to_string(),
                                resource: ID.to_string()
                            };
                            sender.send_message(&OwnedMessage::Text(json!(msg).to_string()));
                            continue
                        }
                    }
                    if msg.command.eq("exit") {
                        break;
                    }
                }
                // Say what we received
                _ => println!("Receive Loop: {:?}", message),
            }
        }
    });
    let _ = receive_loop.join();

    println!("audio service ending");
    Ok(())

}
