use crate::block::transaction::Transaction;
use crossbeam::channel::Sender;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

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

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
    sender_tx: Sender<Transaction>,
}

impl Server {
    pub fn new(sender_tx: Sender<Transaction>) -> Self {
        let listener = TcpListener::bind("localhost:1559").unwrap();
        Server {
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
        match (tokens[0], tokens[1]) {
            ("POST", "/transaction") => {
                // { pubkey: key, outputs: [{ receiver: rcr, amount: 0 }]}
                if let Ok(req) =
                    serde_json::from_str::<AddTransactionRequest>(tokens.last().unwrap())
                {
                    log::debug!("POST /transaction {:?}", req);
                    // let tx = Transaction::new();
                    // self.sender_tx.send(tx).unwrap();
                }
            }
            ("GET", "/") => {
                log::info!("GET /");
            }
            _ => {}
        }

        let response = "HTTP/1.1 200 OK\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
    }
}
