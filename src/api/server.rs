use crossbeam::channel::Sender;
use serde::{Deserialize, Serialize};
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use crate::{database::database::DatabaseType, transaction::Transaction};

#[derive(Deserialize, Serialize, Debug)]
struct ReceiverAmount {
    receiver: String,
    amount: u32,
}

#[derive(Deserialize, Serialize, Debug)]
struct AddTransactionRequest {
    sender: String,
    receivers: Vec<ReceiverAmount>,
}

pub struct Server {
    // dependecies
    database: Arc<Mutex<DatabaseType>>,
    // other
    listener: TcpListener,
    sender_tx: Sender<Transaction>,
}

impl Server {
    pub fn new(sender_tx: Sender<Transaction>, database: Arc<Mutex<DatabaseType>>) -> Self {
        let listener = TcpListener::bind("localhost:1559").unwrap();
        Server {
            database,
            listener,
            sender_tx,
        }
    }
    pub fn listen(&self) {
        log::info!("HTTP server started on port 1559.");
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            self.handle_connection(stream);
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let buf_reader = BufReader::new(&mut stream);
        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let tokens: Vec<&str> = http_request.first().unwrap().split(' ').collect();
        let path_tokens: Vec<&str> = tokens[1].split('/').collect();
        let base_path = tokens[1].split('/').nth(1).unwrap();

        log::debug!("HTTP Request: {:?}", tokens);

        let response = match (tokens[0], base_path) {
            ("GET", "") => create_response(200).to_string(),
            ("GET", "account") => {
                if path_tokens.len() == 3 {
                    let mut pub_key = [0u8; 32];
                    if hex::decode_to_slice(path_tokens[2], &mut pub_key).is_err() {
                        create_response(404).to_string()
                    } else {
                        let tx_hashes: Vec<String> = self
                            .database
                            .lock()
                            .unwrap()
                            .get_transaction_hashes(&pub_key)
                            .iter()
                            .map(hex::encode)
                            .collect();

                        let json = serde_json::to_string(tx_hashes.as_slice()).unwrap();
                        create_response_json(json)
                    }
                } else {
                    create_response(404).to_string()
                }
            }
            ("GET", "transaction") => {
                let mut tx_hash = [0u8; 32];
                if hex::decode_to_slice(path_tokens[2], &mut tx_hash).is_err() {
                    create_response(400).to_string()
                } else if let Some(tx) = self.database.lock().unwrap().get_transaction(&tx_hash) {
                    let json = serde_json::to_string(tx).unwrap();
                    create_response_json(json)
                } else {
                    create_response(404).to_string()
                }
            }
            ("POST", "transaction") => {
                if let Ok(tx) = serde_json::from_str::<Transaction>(tokens.last().unwrap()) {
                    log::debug!("{:?}", tx);
                    self.sender_tx.send(tx).unwrap();
                }
                create_response(200).to_string()
            }
            _ => create_response(200).to_string(),
        };

        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn create_response_json(body: String) -> String {
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/json\r\n\r\n{}", body)
}

fn create_response(status: u32) -> &'static str {
    match status {
        200 => "HTTP/1.1 200 OK\r\n\r\n",
        404 => "HTTP/1.1 404 Not Found\r\n\r\n",
        _ => {
            panic!("not implented {}", status)
        }
    }
}
