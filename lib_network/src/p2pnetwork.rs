// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// P2PNetwork is a struct that implements a peer-to-peer network.
/// It is used to send and receive messages to/from neighbors.
/// It also automatically broadcasts messages. 
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./lib.rs to understand the expected behavior of the P2PNetwork.


use lib_chain::block::{BlockNode, Transaction, BlockId, TxId};
use crate::netchannel::*;
use std::collections::{HashMap, BTreeMap, HashSet};
use std::convert;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::sync::{mpsc, Arc, Mutex};

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
        Receiver<BlockNode>, 
        Receiver<Transaction>, 
        Sender<BlockNode>, 
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

        let network = P2PNetwork {
            send_msg_count: 0,
            recv_msg_count: 0,
            address,
            neighbors,
        };

        let (send_block, recv_block) = mpsc::channel::<BlockNode>(); //send & receive blocks from the network
        let (send_tx, recv_tx) = mpsc::channel::<Transaction>(); //send & receive transactions from the network
        let (send_block_request, recv_block_request) = mpsc::channel::<BlockId>(); //request block from the network

        let p2p_network_mutex = Arc::new(Mutex::new(network));

        let local_addr = format!("{}:{}", address.ip, address.port);
        let listener = TcpListener::bind(local_addr).unwrap();

        //step 3 here, dont really know what to implement, for loop accepts TCP connections automatically
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                    }
                    Err(e) => {
                        eprintln!("Error accepting incoming connection: {}", e);
                    }
                }
            }
        });

        for neighbour in neighbors.iter() {
            let neighbour_addr = format!("{}:{}", neighbour.ip, neighbour.port);
            let mut stream = TcpStream::connect(neighbour_addr);

        }

    }

    /// Get status information of the P2PNetwork for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the network. 
        // It should be displayed in the Client UI eventually.
        let mut statuses: BTreeMap<String, String> = BTreeMap::new();
        statuses.insert("address".to_string(), format!("{}:{}", self.address.ip, self.address.port));
        statuses.insert("send_msg_count".to_string(), self.send_msg_count.to_string());
        statuses.insert("recv_msg_count".to_string(), self.recv_msg_count.to_string());
        statuses.insert("neighbours".to_string(), self.neighbors
            .iter()
            .map(|n| format!("{}:{}", n.ip, n.port))
            .collect::<Vec<String>>()
            .join(", "));

        statuses
    }

}


