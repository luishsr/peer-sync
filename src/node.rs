use colored::*;
use jsonrpc_http_server::{ServerBuilder, jsonrpc_core::*};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::blockchain::{Blockchain, Transaction, Block};

#[derive(Debug)]
struct Peer {
    stream: Arc<Mutex<TcpStream>>,
    address: String,
}

#[derive(Clone)]
pub struct Node {
    address: String,
    peers: Arc<Mutex<Vec<Peer>>>,
    pub blockchain: Arc<Mutex<Blockchain>>,
}

impl Node {
    pub fn new(address: &str) -> Self {
        Node {
            address: address.to_string(),
            peers: Arc::new(Mutex::new(Vec::new())),
            blockchain: Arc::new(Mutex::new(Blockchain::new())),
        }
    }

    pub fn mine(&self) {
        let mut blockchain = self.blockchain.lock().unwrap();
        let block = blockchain.mine_block();
        self.broadcast_block(block);
    }

    pub fn start(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.address)?;
        println!("Server listening on {}", self.address);

        // Start the RPC server in a separate thread
        let rpc_node = self.clone();
        thread::spawn(move || {
            Node::start_rpc_server(rpc_node.into());
        });

        // Accept incoming connections
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let node = self.clone();
                    thread::spawn(move || {
                        if let Err(e) = Arc::new(node.clone()).handle_client(stream) {
                            eprintln!("{} - Connection failed: {}", node.address, e);
                        }
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }

        Ok(())
    }

    fn handle_client(self: Arc<Self>, stream: TcpStream) -> std::io::Result<()> {
        let address = stream.peer_addr()?.to_string();
        let stream = Arc::new(Mutex::new(stream));
        self.peers.lock().unwrap().push(Peer {
            stream: stream.clone(),
            address: address.clone(),
        });
        println!("{} - New peer connected: {}", self.address, address);

        let node_clone = self.clone();
        let address_clone = self.address.clone();
        let stream_clone = stream.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stream_clone.lock().unwrap().try_clone().unwrap());
            for line in reader.lines() {
                match line {
                    Ok(message) => {
                        println!(
                            "{} - {} {}: {}",
                            address_clone,
                            "Received message from".green(),
                            address,
                            message
                        );
                        // Process received message
                        if let Ok(block) = serde_json::from_str::<Block>(&message) {
                            if node_clone.blockchain.lock().unwrap().add_block(block.clone()) {
                                node_clone.broadcast_block(block);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from {}: {}", address, e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub fn broadcast_block(&self, block: Block) {
        let block_message = serde_json::to_string(&block).expect("Failed to serialize block");
        let peers = self.peers.lock().unwrap();
        for peer in peers.iter() {
            let mut stream = peer.stream.lock().unwrap();
            if let Err(e) = stream.write_all(block_message.as_bytes()) {
                eprintln!("Failed to send block to {}: {}", peer.address, e);
            }
            if let Err(e) = stream.flush() {
                eprintln!("Failed to flush stream to {}: {}", peer.address, e);
            } else {
                println!(
                    "{} - {} {}",
                    self.address,
                    "Broadcasted block to".blue(),
                    peer.address
                );
            }
        }
    }

    pub fn add_peer(&self, address: String) {
        for attempt in 1..=3 {
            if self.is_connected(&address) {
                break;
            }

            println!(
                "{} - Attempt {} to connect to {}",
                self.address, attempt, &address
            );
            match TcpStream::connect(&address) {
                Ok(stream) => {
                    let address = stream.peer_addr().unwrap().to_string();
                    println!("{} - Connected to peer at {}", self.address, address);
                    if let Err(e) = Arc::new(self.clone()).handle_client(stream) {
                        eprintln!("{} - Error handling client: {}", self.address, e);
                    }
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "{} - Failed to connect to peer {}: {} (attempt {})",
                        self.address, address, e, attempt
                    );
                    if attempt < 3 {
                        thread::sleep(Duration::from_secs(5));
                    }
                }
            }
        }
    }

    fn is_connected(&self, peer_address: &str) -> bool {
        let peers = self.peers.lock().unwrap();
        for peer in peers.iter() {
            if peer.address == peer_address {
                return true;
            }
        }
        false
    }

    // Start the RPC server with the necessary methods
    fn start_rpc_server(node: Arc<Node>) {
        let mut io = IoHandler::new();

        // Clone the node for the send_message method
        let send_message_node = node.clone();
        io.add_method("send_message", move |params: Params| {
            let message: String = params.parse()?;
            let node_clone = send_message_node.clone();
            thread::spawn(move || {
                println!("Received RPC message: {}", message);
                node_clone.broadcast_message(&message);
            });
            Ok(Value::String("Message broadcasted".into()))
        });

        // Clone the node for the create_account method
        let create_account_node = node.clone();
        io.add_method("create_account", move |_| {
            let node_clone = create_account_node.clone();
            thread::spawn(move || {
                let account_id = node_clone.blockchain.lock().unwrap().create_account();
                println!("Created account: {}", account_id);
                node_clone.broadcast_message(&account_id);
            });
            Ok(Value::String("Account created".into()))
        });

        // Clone the node for the send_transaction method
        let send_transaction_node = node.clone();
        io.add_method("send_transaction", move |params: Params| {
            let (from, to, amount): (String, String, u64) = params.parse()?;
            let node_clone = send_transaction_node.clone();
            thread::spawn(move || {
                let success = node_clone.blockchain.lock().unwrap().add_transaction(from, to, amount);
                println!("Transaction success: {}", success);
            });
            Ok(Value::String("Transaction processed".into()))
        });

        // Clone the node for the mine_block method
        let mine_block_node = node.clone();
        io.add_method("mine_block", move |_| {
            let node_clone = mine_block_node.clone();
            thread::spawn(move || {
                node_clone.mine();
            });
            Ok(Value::String("Block mined".into()))
        });

        let server = ServerBuilder::new(io)
            .threads(3)
            .start_http(&node.address.parse().unwrap())
            .unwrap();

        println!("RPC server started on {}", node.address);
        server.wait();
    }


    fn broadcast_message(&self, message: &str) {
        let peers = self.peers.lock().unwrap();
        for peer in peers.iter() {
            let mut stream = peer.stream.lock().unwrap();
            if let Err(e) = stream.write_all(message.as_bytes()) {
                eprintln!("Failed to send message to {}: {}", peer.address, e);
            }
            if let Err(e) = stream.flush() {
                eprintln!("Failed to flush stream to {}: {}", peer.address, e);
            } else {
                println!(
                    "{} - {} {}",
                    self.address,
                    "Broadcasted message to".blue(),
                    peer.address
                );
            }
        }
    }
}
