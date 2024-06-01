use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use tonic::transport::Channel;
use tonic::{transport::Server, Request, Response, Status};

use crate::crypto::hash_utils::Address;
use crate::database::database::DatabaseType;
use crate::proto::proto_node::node_client::NodeClient;
use crate::proto::proto_node::node_server::{Node, NodeServer};
use crate::proto::proto_node::{
    self, Block, BlockReq, Chain, HandshakeMessage, PeerList, PublicKey, Transaction,
    TransactionReq, UnspentOutput, UnspentOutputs,
};

pub struct Network {
    peers: Arc<Mutex<HashMap<String, NodeClient<Channel>>>>,
    blocked_peers: Arc<Vec<String>>,
    seed_list: Vec<String>,
    database: Arc<Mutex<DatabaseType>>,
    address: String,
}
impl Network {
    pub fn new(port: u16, seed_list: Vec<String>, database: Arc<Mutex<DatabaseType>>) -> Self {
        let peers = Arc::new(Mutex::new(HashMap::new()));
        let address = format!("[::1]:{}", port);
        let blocked_peers = Arc::new(vec![address.clone()]);
        Network {
            address,
            blocked_peers,
            seed_list,
            peers,
            database,
        }
    }

    pub async fn start_network_node(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Node gRPC server started on {}", &self.address);
        let server = NetworkServer::new(
            self.peers.clone(),
            self.blocked_peers.clone(),
            self.database.clone(),
            self.address.clone(),
        );
        Server::builder()
            .add_service(NodeServer::new(server))
            .serve(self.address.parse().unwrap())
            .await?;

        Ok(())
    }

    pub async fn start_sync(&self) -> Result<(), String> {
        // explore peers
        let mut longest_chain = ("".to_string(), 0);
        let peers = self.peers.clone();
        let blocked_peers = self.blocked_peers.clone();

        let mut neighbours: Vec<String>;
        {
            let peers = peers.lock().unwrap();
            neighbours = self
                .seed_list
                .clone()
                .into_iter()
                .filter(|p| !self.blocked_peers.contains(&p) && !peers.contains_key(p))
                .collect();
        }

        while let Some(next_peer) = neighbours.pop() {
            let (new_peers, block_height) = self
                .connect_and_get_info(next_peer.to_string(), peers.clone())
                .await;
            if block_height > longest_chain.1 {
                longest_chain.0 = next_peer;
                longest_chain.1 = block_height;
            }

            let peers = peers.lock().unwrap();
            for new_peer in new_peers {
                if blocked_peers.contains(&new_peer) || peers.contains_key(&new_peer) {
                    continue;
                }
            }
        }

        if longest_chain.0 == "" || longest_chain.1 == 0 {
            return Err("Failed to find a peer for synchronization!".to_string());
        }

        // select the peer with highest block height and download chain
        if let Some(client) = self.peers.lock().unwrap().get_mut(&longest_chain.0) {
            if let Ok(chain_json) = client.get_chain(Request::new(proto_node::None {})).await {
                if let Ok(blocks) =
                    serde_json::from_str::<Vec<crate::block::Block>>(&chain_json.get_ref().blocks)
                {
                    log::debug!(
                        "Downloaded {} blocks from peer {}",
                        blocks.len(),
                        &longest_chain.0
                    );

                    for block in blocks {
                        if block.verify(&self.database) {
                            self.database.lock().unwrap().insert_block(block);
                        } else {
                            log::warn!("Block verification failed!");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn connect_and_get_info(
        &self,
        peer_addr: String,
        peers: Arc<Mutex<HashMap<String, NodeClient<Channel>>>>,
    ) -> Pin<Box<dyn Future<Output = (Vec<String>, u32)> + Send>> {
        let version = self.database.lock().unwrap().get_version();
        let block_height = self.database.lock().unwrap().block_height() as u32;
        let server_address = self.address.clone();

        Box::pin(async move {
            let mut new_peers = vec![];
            let mut peer_block_height = 0;

            if let Ok(mut client) = NodeClient::connect(peer_addr.clone()).await {
                log::debug!("Connected to {}.", peer_addr);

                if let Ok(resp) = client
                    .handshake(Request::new(HandshakeMessage {
                        version,
                        block_height,
                        server_address,
                    }))
                    .await
                {
                    peers.lock().unwrap().insert(peer_addr, client.clone());
                    peer_block_height = resp.get_ref().block_height;

                    if let Ok(peer_list) = client.get_peer_list(proto_node::None {}).await {
                        for p in peer_list.get_ref().peers.clone() {
                            new_peers.push(p.to_string());
                        }
                    }
                }
            } else {
                log::warn!("Failed to connect to {}.", &peer_addr);
            }

            return (new_peers, peer_block_height);
        })
    }
}

struct NetworkServer {
    peers: Arc<Mutex<HashMap<String, NodeClient<Channel>>>>,
    blocked_peers: Arc<Vec<String>>,
    database: Arc<Mutex<DatabaseType>>,
    address: String,
}
impl NetworkServer {
    fn new(
        peers: Arc<Mutex<HashMap<String, NodeClient<Channel>>>>,
        blocked_peers: Arc<Vec<String>>,
        database: Arc<Mutex<DatabaseType>>,
        address: String,
    ) -> Self {
        NetworkServer {
            peers,
            blocked_peers,
            database,
            address,
        }
    }
}

#[tonic::async_trait]
impl Node for NetworkServer {
    async fn handshake(
        &self,
        request: Request<HandshakeMessage>,
    ) -> Result<Response<HandshakeMessage>, Status> {
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
            server_address: self.address.clone(),
        };

        let peer_address = request.get_ref().server_address.to_string();
        let contained = self.peers.lock().unwrap().contains_key(&peer_address);
        if !contained && !self.blocked_peers.contains(&peer_address) {
            if let Ok(client) =
                NodeClient::connect(format!("http://{}", peer_address.clone())).await
            {
                log::debug!("Added a connection to a new peer {}.", peer_address);
                self.peers
                    .lock()
                    .unwrap()
                    .insert(peer_address, client.clone());

                // let peers = self.peers.clone();
                // let blocked_peers = self.blocked_peers.clone();
                // tokio::spawn(
                //     async move { Network::explore_peers(peers, blocked_peers, client).await },
                // );
            } else {
                log::warn!("Failed to connect to peer {}.", peer_address);
            }
        }

        Ok(Response::new(reply))
    }

    async fn get_peer_list(
        &self,
        _request: Request<proto_node::None>,
    ) -> Result<Response<PeerList>, Status> {
        let peers: Vec<String> = self
            .peers
            .lock()
            .unwrap()
            .keys()
            .map(|x| x.to_string())
            .collect();

        log::debug!("Responded with peer list: {:?}", peers);
        Ok(Response::new(PeerList { peers }))
    }

    async fn add_block(
        &self,
        request: Request<Block>,
    ) -> Result<Response<proto_node::None>, Status> {
        log::debug!("add_block: {:?}", request);
        todo!()
    }

    async fn get_block(&self, request: Request<BlockReq>) -> Result<Response<Block>, Status> {
        log::debug!("get_block: {:?}", request);
        todo!()
    }

    async fn get_chain(&self, _: Request<proto_node::None>) -> Result<Response<Chain>, Status> {
        let db = self.database.lock().unwrap();
        let blocks = db.get_blocks();
        if let Ok(json) = serde_json::to_string(&blocks) {
            Ok(Response::new(Chain { blocks: json }))
        } else {
            Err(Status::new(
                tonic::Code::Internal,
                "Failed to encode blocks to JSON.",
            ))
        }
    }

    async fn add_transaction(
        &self,
        request: Request<Transaction>,
    ) -> Result<Response<proto_node::None>, Status> {
        if let Ok(tx) = serde_json::from_str::<crate::transaction::Transaction>(
            request.get_ref().tx_json.as_str(),
        ) {
            log::debug!("tx_hash={:?}", hex::encode(tx.hash()));
            self.database.lock().unwrap().add_pending_transaction(tx);
            return Ok(Response::new(proto_node::None {}));
        }
        Err(Status::invalid_argument("Failed to decode transaction."))
    }

    async fn get_transaction(
        &self,
        request: Request<TransactionReq>,
    ) -> Result<Response<Transaction>, Status> {
        log::debug!("get_tranction: {:?}", request);
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
