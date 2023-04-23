// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// P2PNetwork is a struct that implements a peer-to-peer network.
/// It is used to send and receive messages to/from neighbors.
/// It also automatically broadcasts messages. 
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./lib.rs to understand the expected behavior of the P2PNetwork.


use lib_chain::block::{BlockNode, Transaction, BlockId};
use crate::netchannel::*;
use std::collections::BTreeMap;
use std::time::Duration;
use std::sync;
use std::net::TcpListener;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::sync::{Arc, Mutex};
use crate::p2pnetwork::NetMessage::*;

/// The struct to represent statistics of a peer-to-peer network.
pub struct P2PNetwork {
    /// The number of messages sent by this node.
    pub send_msg_count: u64,
    /// The number of messages received by this node.
    pub recv_msg_count: u64,
    /// The address of this node.
    pub address: NetAddress,
    /// The addresses of the neighbors.
    pub neighbors: Vec<NetAddress>,
}


impl P2PNetwork {
    /// Creates a new P2PNetwork instance and associated FIFO communication channels.
    /// There are 5 FIFO channels. 
    /// Those channels are used for communication within the process.
    /// They abstract away the network and neighbor nodes. 
    /// More specifically, they are for communicating between `bin_nakamoto` threads 
    /// and threads that are responsible for TCP network communication.
    /// The usage of those five channels can be guessed from the type:
    /// 1. Receiver<BlockNode>: read from this FIFO channel to receive blocks from the network.
    /// 2. Receiver<Transaction>: read from this FIFO channel to receive transactions from the network.
    /// 3. Sender<BlockNode>: write to this FIFO channel to broadcast a block to the network.
    /// 4. Sender<Transaction>: write to this FIFO channel to broadcast a transaction to the network.
    /// 5. Sender<BlockId>: write to this FIFO channel to request a block from the network.
    pub fn create(address: NetAddress, neighbors: Vec<NetAddress>) -> (
        Arc<Mutex<P2PNetwork>>,
        Receiver<BlockNode>, //when client send to you, put this inside Sender
        Receiver<Transaction>,
        Receiver<BlockId>,
        Sender<BlockNode>, //when program send to you, use receiver to get message and broadcast
        Sender<Transaction>,
        Sender<BlockId>
    ) {
        // Please fill in the blank
        // You might need to perform the following steps:
        // 1. create a P2PNetwork instance
        // 2. create mpsc channels for sending and receiving messages
        // 3. create a thread for accepting incoming TCP connections from neighbors
        // 4. create TCP connections to all neighbors
        // 5. create threads for each TCP connection to send messages
        // 6. create threads to listen to messages from neighbors
        // 7. create threads to distribute received messages (send to channels or broadcast to neighbors)
        // 8. return the created P2PNetwork instance and the mpsc channels

        // 1 P2P network interface
        let network = P2PNetwork {
            send_msg_count: 0,
            recv_msg_count: 0,
            address,
            neighbors,
        };

        let network_address = network.address.clone();
        let network_neighbors = network.neighbors.clone();
        let p2p_network_mutex = Arc::new(Mutex::new(network));

        // 2 mpsc channel
        let (process_block_sender, process_block_recv) = sync::mpsc::channel::<BlockNode>();
        let (process_tx_sender, process_tx_recv) = sync::mpsc::channel::<Transaction>();
        let (process_id_sender, process_id_recv) = sync::mpsc::channel::<BlockId>();
        let (relay_block_sender, relay_block_recv) = sync::mpsc::channel::<BlockNode>();
        let (relay_tx_sender, relay_tx_recv) = sync::mpsc::channel::<Transaction>();
        let (relay_id_sender, relay_id_recv) = sync::mpsc::channel::<BlockId>();

        // 3 incoming TCP connection
        let local_addr = format!("{}:{}", network_address.ip, network_address.port);
        let listener = TcpListener::bind(local_addr).unwrap();

        let lock_recv = Arc::clone(&p2p_network_mutex);
        let _listen_handler = thread::spawn(move || {
            for stream in listener.incoming() {
                let block_sender: Sender<BlockNode> = process_block_sender.clone();
                let tx_sender: Sender<Transaction> = process_tx_sender.clone();
                let id_sender: Sender<BlockId> = process_id_sender.clone();

                let lock_recv = Arc::clone(&lock_recv);

                match stream {
                    Ok(stream) => {
                        // 6. create threads to listen to messages from neighbors
                        let mut neighbor = NetChannelTCP::from_stream(stream);
                        thread::spawn(move || {
                            loop {
                                let msg;
                                let msg_result = neighbor.read_msg();
                                match msg_result {
                                    Some(m) => { msg = m; },
                                    None => { continue; },
                                }

                                match msg {
                                    BroadcastBlock(node) => { block_sender.send(node).unwrap(); },
                                    BroadcastTx(tx) => { tx_sender.send(tx).unwrap(); },
                                    RequestBlock(id) => { id_sender.send(id).unwrap(); },
                                    Unknown(_) => { }, //println!("{}", debug_msg);
                                }

                                lock_recv.lock().unwrap().recv_msg_count += 1;
                            }
                        });
                    },
                    Err(_) => {  } //TODO
                }
            }
        });

        // 4 Create TCP connection
        let outgoing_neighbors = Arc::new(Mutex::new(Vec::<NetChannelTCP>::new()));
        let outgoing_neighbors2 = Arc::new(Mutex::new(Vec::<NetChannelTCP>::new()));
        let outgoing_neighbors3 = Arc::new(Mutex::new(Vec::<NetChannelTCP>::new()));

        for neighbor in network_neighbors.into_iter() {
            let outgoing_neighbors_clone = Arc::clone(&outgoing_neighbors);
            let outgoing_neighbors2_clone = Arc::clone(&outgoing_neighbors2);
            let outgoing_neighbors3_clone = Arc::clone(&outgoing_neighbors3);
            thread::spawn(move || {
                loop {
                    let neighbor_tcp_result = NetChannelTCP::from_addr(&neighbor);
                    match neighbor_tcp_result {
                        Ok(mut neighbor_tcp) => {
                            println!("{{\"Notify\":\"Connected to {}:{}\"}}", neighbor.ip, neighbor.port);
                            outgoing_neighbors_clone.lock().unwrap().push(neighbor_tcp.clone_channel());
                            outgoing_neighbors2_clone.lock().unwrap().push(neighbor_tcp.clone_channel());
                            outgoing_neighbors3_clone.lock().unwrap().push(neighbor_tcp.clone_channel());
                            break;
                        },
                        Err(_) => {
                            eprintln!("Retrying connection to {}:{}...", neighbor.ip, neighbor.port);
                            thread::sleep(Duration::from_secs(1));
                            continue;
                        },
                    }
                }
            });
        }

        // 7 Distribute messages
        let lock = Arc::clone(&p2p_network_mutex);
        thread::spawn(move || {
            loop {
                // recv_timeout will get an error every 10 seconds when nothing is received REMOVED
                let msg_result = relay_block_recv.recv();
                let msg: BlockNode;
                match msg_result {
                    Ok(node) => { msg = node; },
                    Err(_) => { continue; },
                }

                // 5 create threads for each TCP connection to send messages
                let cloned_neighbors: Vec<NetChannelTCP> = outgoing_neighbors.lock().unwrap().iter_mut().map(|n| n.clone_channel()).collect();
                for mut n in cloned_neighbors.into_iter() {
                    let msg_clone = msg.clone();
                    let lock = Arc::clone(&lock);
                    thread::spawn(move || {
                        lock.lock().unwrap().send_msg_count += 1;
                        n.write_msg(NetMessage::BroadcastBlock(msg_clone));
                    });
                }
            }
        });

        let lock2 = Arc::clone(&p2p_network_mutex);
        thread::spawn(move || {
            loop {
                // recv_timeout will get an error every 10 seconds when nothing is received REMOVED
                let msg_result = relay_tx_recv.recv();
                let msg: Transaction;
                match msg_result {
                    Ok(tx) => { msg = tx; },
                    Err(_) => { continue; },
                }

                // 5 create threads for each TCP connection to send messages
                let cloned_neighbors: Vec<NetChannelTCP> = outgoing_neighbors2.lock().unwrap().iter_mut().map(|n| n.clone_channel()).collect();
                for mut n in cloned_neighbors.into_iter() {
                    let msg_clone = msg.clone();
                    let lock2 = Arc::clone(&lock2);
                    thread::spawn(move || {
                        lock2.lock().unwrap().send_msg_count += 1;
                        n.write_msg(NetMessage::BroadcastTx(msg_clone));
                    });
                }
            }
        });

        let lock3 = Arc::clone(&p2p_network_mutex);
        thread::spawn(move || {
            loop {
                let msg_result = relay_id_recv.recv();
                let msg: BlockId;
                match msg_result {
                    Ok(id) => { msg = id; },
                    Err(_) => { continue; },
                }

                // 5 create threads for each TCP connection to send messages
                let cloned_neighbors: Vec<NetChannelTCP> = outgoing_neighbors3.lock().unwrap().iter_mut().map(|n| n.clone_channel()).collect();
                for mut n in cloned_neighbors.into_iter() {
                    let msg_clone = msg.clone();
                    let lock3 = Arc::clone(&lock3);
                    thread::spawn(move || {
                        lock3.lock().unwrap().send_msg_count += 1;
                        n.write_msg(NetMessage::RequestBlock(msg_clone));
                    });
                }
            }
        });

        // 8 return
        (p2p_network_mutex, process_block_recv, process_tx_recv, process_id_recv, relay_block_sender, relay_tx_sender, relay_id_sender)

    }

    /// Get status information of the P2PNetwork for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the network. 
        // It should be displayed in the Client UI eventually.
        let mut statuses: BTreeMap<String, String> = BTreeMap::new();
        statuses.insert("#address".to_string(), format!("ip: {} port: {}", self.address.ip, self.address.port));
        statuses.insert("#send_msg".to_string(), self.send_msg_count.to_string());
        statuses.insert("#recv_msg".to_string(), self.recv_msg_count.to_string());

        statuses
    }

}

