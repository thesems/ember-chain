use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tonic::{Request, Response, Status, transport::Server};
use tonic::transport::Channel;

use crate::crypto::hash_utils::Address;
use crate::database::database::DatabaseType;
use crate::proto::proto_node::{
    Balance, Block, BlockReq, HandshakeMessage, None, PeerList, PublicKey, Transaction,
    TransactionReq, UnspentOutput, UnspentOutputs,
};
use crate::proto::proto_node::node_client::NodeClient;
use crate::proto::proto_node::node_server::{Node, NodeServer};
use crate::transaction::input::Input;
use crate::transaction::output::Output;

struct Network {
    peers: Arc<Mutex<HashMap<String, NodeClient<Channel>>>>,
    database: Arc<Mutex<DatabaseType>>,
}
impl Network {
    fn new(database: Arc<Mutex<DatabaseType>>) -> Self {
        Network {
            peers: Arc::new(Mutex::new(HashMap::new())),
            database,
        }
    }
}

#[tonic::async_trait]
impl Node for Network {
    async fn handshake(
        &self,
        request: Request<HandshakeMessage>,
    ) -> Result<Response<HandshakeMessage>, Status> {
        log::debug!("{:?}", &request);

        const VERSION: &str = env!("CARGO_PKG_VERSION");
        let block_height = self.database.lock().unwrap().block_height() as u32;

        let our_version = semver::Version::parse(VERSION).unwrap();
        let their_version = semver::Version::parse(&request.get_ref().version);

        if let Ok(their_version) = their_version {
            if our_version.lt(&their_version) {
                log::warn!(
                    "Peer client is running a higher version ({}).",
                    their_version
                )
            }
        } else {
            log::warn!(
                "Peer client is running an invalid version ({}).",
                &request.get_ref().version,
            )
        }

        let reply = HandshakeMessage {
            version: VERSION.to_string(),
            block_height,
        };

        if let Some(addr) = request.remote_addr() {
            let peer_address = addr.to_string();
            if !self.peers.lock().unwrap().contains_key(&peer_address) {
                if let Ok(client) = NodeClient::connect(peer_address.clone()).await {
                    log::debug!("Added a connection to a new peer {}.", addr);
                    self.peers.lock().unwrap().insert(peer_address, client);
                } else {
                    log::warn!("Failed to connect to peer {}.", addr);
                }
            }
        }

        Ok(Response::new(reply))
    }

    async fn get_peer_list(&self, _request: Request<None>) -> Result<Response<PeerList>, Status> {
        let peers: Vec<String> = self
            .peers
            .lock()
            .unwrap()
            .keys()
            .map(|x| x.to_string())
            .collect();

        let reply = PeerList { peers };
        Ok(Response::new(reply))
    }

    async fn add_block(&self, request: Request<Block>) -> Result<Response<None>, Status> {
        log::debug!("{:?}", request);
        todo!()
    }

    async fn get_block(&self, request: Request<BlockReq>) -> Result<Response<Block>, Status> {
        log::debug!("{:?}", request);
        todo!()
    }

    async fn get_balance(&self, request: Request<PublicKey>) -> Result<Response<Balance>, Status> {
        let address = &request.get_ref().key;
        let balance = self.database.lock().unwrap().get_balance(address);
        Ok(Response::new(Balance { balance }))
    }

    async fn add_transaction(
        &self,
        request: Request<Transaction>,
    ) -> Result<Response<None>, Status> {
        if let Ok(tx) = serde_json::from_str::<crate::transaction::Transaction>(
            request.get_ref().tx_json.as_str()) {
            self.database.lock().unwrap().add_pending_transaction(tx);
            return Ok(Response::new(None {}));
        }
        Err(Status::invalid_argument("Failed to decode transaction."))
    }

    async fn get_transaction(
        &self,
        request: Request<TransactionReq>,
    ) -> Result<Response<Transaction>, Status> {
        log::debug!("{:?}", request);
        todo!()
    }

    async fn get_utxo(
        &self,
        request: Request<PublicKey>,
    ) -> Result<Response<UnspentOutputs>, Status> {
        let public_key = request.into_inner().key as Address;
        let utxos = self.database.lock().unwrap().get_utxo(&public_key);
        let mut unspent_outputs = vec![];
        for utxo in &utxos {
            unspent_outputs.push(UnspentOutput {
                previous_transaction_hash: utxo.0.to_vec(),
                previous_transaction_output_index: utxo.1,
                amount: utxo.2,
            });
        }
        Ok(Response::new(UnspentOutputs { unspent_outputs }))
    }
}

pub async fn start_network_node(
    port: u16,
    database: Arc<Mutex<DatabaseType>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("[::1]:{}", port).parse().unwrap();
    let network_node = Network::new(database);

    log::info!("Node gRPC server started on {}", addr);
    Server::builder()
        .add_service(NodeServer::new(network_node))
        .serve(addr)
        .await?;

    Ok(())
}
