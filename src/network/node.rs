use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::database::database::DatabaseType;
use crate::proto::proto_node::node_client::NodeClient;
use crate::proto::proto_node::node_server::{Node, NodeServer};
use crate::proto::proto_node::{Ack, Block, BlockHeight, None, PeerList, Transaction, Version};

use tonic::transport::Channel;
use tonic::{transport::Server, Request, Response, Status};

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
        request: tonic::Request<Version>,
    ) -> std::result::Result<tonic::Response<Version>, tonic::Status> {
        log::debug!("{:?}", &request);

        const VERSION: &str = env!("CARGO_PKG_VERSION");
        let block_height = self.database.lock().unwrap().block_height() as i32;

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

        let reply = Version {
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

    async fn add_block(
        &self,
        request: tonic::Request<Block>,
    ) -> std::result::Result<tonic::Response<Ack>, tonic::Status> {
        log::debug!("{:?}", request);
        todo!()
    }

    async fn get_block(
        &self,
        request: tonic::Request<BlockHeight>,
    ) -> std::result::Result<tonic::Response<Block>, tonic::Status> {
        log::debug!("{:?}", request);
        todo!()
    }

    async fn add_transaction(
        &self,
        request: tonic::Request<Transaction>,
    ) -> std::result::Result<tonic::Response<Ack>, tonic::Status> {
        todo!()
    }

    async fn get_peer_list(
        &self,
        _request: tonic::Request<None>,
    ) -> std::result::Result<tonic::Response<PeerList>, tonic::Status> {
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
