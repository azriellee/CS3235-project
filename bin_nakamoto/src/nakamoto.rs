// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file implements the Nakamoto struct, related data structs and methods.
// The Nakamoto leverages lib_chain, lib_miner, lib_tx_pool and lib_network to implement the Nakamoto consensus algorithm.
// You can see detailed instructions in the comments below.

use std::sync::mpsc::Sender;
use std::{thread, time::Duration};
use std::sync::{Arc, Mutex, RwLock};
use serde::{Deserialize, Serialize};
use lib_chain::block::{BlockTree, Puzzle, Transactions, MerkleTree, BlockNode, BlockNodeHeader, Transaction, BlockId};
use lib_miner::miner::{Miner, PuzzleSolution};
use lib_tx_pool::pool::TxPool;
use lib_network::p2pnetwork::{P2PNetwork};
use lib_network::netchannel::{NetAddress};
use std::collections::{HashMap, BTreeMap};
use sha2::{Sha256, Digest}; //might remove if they say donnid check the correctness of a solution found by another node

type UserId = String;

/// The struct to represent configuration of the Nakamoto instance.
/// The configuration does not contain any user information. The Nakamoto algorithm is user-independent.
/// The configuration sets information about neighboring nodes, miner, block creation, etc.
#[derive(Serialize, Deserialize, Debug, Clone)] 
pub struct Config {
    /// the list of addresses of neighboring nodes
    pub neighbors: Vec<NetAddress>,
    /// the address of this node
    pub addr: NetAddress,
    /// the number of threads used to mine a new block (for miner)
    pub miner_thread_count: u16,
    /// the length of the nonce string (for miner)
    pub nonce_len: u16,
    // difficulty to mine a new block (for miner)
    pub difficulty_leading_zero_len: u16,
    // difficulty to accept a new block (for verifying the block)
    pub difficulty_leading_zero_len_acc: u16,
    // the seed for the miner thread 0 (for miner)
    pub miner_thread_0_seed: u64,
    // the reward receiver (for mined blocks)
    pub mining_reward_receiver: UserId,
    // the max number of transactions in one block (for creating a new block)
    pub max_tx_in_one_block: u16
}


/// Create a puzzle for the miner given a chain and a tx pool (as smart pointers). 
/// It returns the puzzle (serialization of the Puzzle struct) and the corresponding incomplete block (nonce and block_id not filled)
fn create_puzzle(chain_p: Arc<Mutex<BlockTree>>, tx_pool_p: Arc<Mutex<TxPool>>, tx_count: u16, reward_receiver: UserId) -> (String, BlockNode) {
    // Please fill in the blank
    // Filter transactions from tx_pool and get the last node of the longest chain.
    // tx_count corresponds to max_count of filter_txs in TxPool

    // hi @chuawei i need help with the blocks stuff
    // 1. I think must delete the tx from tx_pool that is already in finalised blocks, but not sure how to get finalised blocks from chain_p.
    // after that can just use the "remove_txs_from_finalized_blocks" function in tx_pool
    // 2. Not very sure how to get the last node of the longest chain also

    let tx_pool = tx_pool_p.lock().unwrap();
    let block_chain = chain_p.lock().unwrap();
    let mut excluding_tx = Vec::new();
    let node_of_longest_chain: BlockId; //need help initialising this
    for tx_id in block_chain.finalized_tx_ids.iter() {
        let tx = tx_pool.pool_tx_map[tx_id];
        excluding_tx.push(tx);
    }
    for tx_id in tx_pool.removed_tx_ids.iter() {
        let tx = tx_pool.pool_tx_map[tx_id];
        excluding_tx.push(tx);
    }
    let unfinalised_tx = tx_pool.filter_tx(tx_count, &excluding_tx);
    let (merkle_root, merkle_tree) = MerkleTree::create_merkle_tree(unfinalised_tx);
    // build the puzzle
    let puzzle = Puzzle {
        // Please fill in the blank
        // Create a puzzle with the block_id of the parent node and the merkle root of the transactions.
        parent: node_of_longest_chain,
        merkle_root,
        reward_receiver,
    };
    let puzzle_str = serde_json::to_string(&puzzle).unwrap().to_owned();

    // Please fill in the blank
    // Create a block node with the transactions and the merkle root.
    // Leave the nonce and the block_id empty (to be filled after solving the puzzle).
    // The timestamp can be set to any positive interger.
    // In the end, it returns  (puzzle_str, pre_block);

    let pre_block_header: BlockNodeHeader; //need help initilaising this
    let pre_block_transactions: Transactions = Transactions { 
        merkle_tree, 
        transactions: excluding_tx, 
    };
    let pre_block: BlockNode = BlockNode {
        header: pre_block_header,
        transactions_block: pre_block_transactions,
    };

    (puzzle_str, pre_block)
}

/// The struct to represent the Nakamoto instance.
/// The Nakamoto instance contains the chain, the miner, the network and the tx pool as smart pointers.
/// It also contains a FIFO channel for sending transactions to the Blockchain
pub struct Nakamoto {
    /// the chain (BlockTree)
    pub chain_p: Arc<Mutex<BlockTree>>,
    /// the miner
    pub miner_p: Arc<Mutex<Miner>>,
    /// the p2pnetwork
    pub network_p: Arc<Mutex<P2PNetwork>>,
    /// the transaction pool
    pub tx_pool_p: Arc<Mutex<TxPool>>,
    /// the FIFO channel for sending transactions to the Blockchain
    trans_tx: Sender<Transaction>
}

impl Nakamoto {

    /// A function to send notification messages to stdout (For debugging purpose only)
    pub fn stdout_notify(msg: String) {
        let msg = HashMap::from([("Notify".to_string(), msg.clone())]);
        println!("{}", serde_json::to_string(&msg).unwrap());
    }

    /// Create a Nakamoto instance given the serialized chain, tx pool and config as three json strings.
    pub fn create_nakamoto (chain_str: String, tx_pool_str: String, config_str: String) -> Nakamoto {
        // Please fill in the blank
        // Deserialize the config from the given json string.
        // Deserialize the chain and the tx pool from the given json strings.
        // Create the miner and the network according to the config.
        // Start necessary threads that read from and write to FIFO channels provided by the network.
        // Start necessary thread(s) to control the miner.
        // Return the Nakamoto instance that holds pointers to the chain, the miner, the network and the tx pool.

        //verify solution that is found by another node
        fn verify_solution(block: BlockNode, puzzle: String, difficulty: u16) -> bool {
            let solution = block.header.nonce;
            let combined_string = format!({}{},solution, puzzle);
            let mut hasher = Sha256::new();
            hasher.update(combined_string.as_bytes());
            let result = hasher.finalize();
            let hex_string = result.iter().map(|b| format!("{:02x}", b)).collect::<String>();
            if(hex_string.starts_with(&"0".repeat(difficulty as usize))) {
                return true;
            }
            return false;
        }

        let user_config : Config = serde_json::from_str(&config_str.as_str()).unwrap(); 
        let user_chain : BlockTree = serde_json::from_str(&chain_str.as_str()).unwrap();
        let user_txPool : TxPool = serde_json::from_str(&tx_pool_str.as_str()).unwrap();
        let user_miner : Miner = Miner { 
            thread_count: user_config.miner_thread_count,
            leading_zero_len: user_config.difficulty_leading_zero_len,
            is_running: false 
        };
        let cancellation_token = Arc::new(RwLock::new(false));
        let (network_p, block_rx, trans_rx, block_tx, trans_tx, blockid_tx)
             = P2PNetwork::create(user_config.addr, user_config.neighbors);

        let chain_p = Arc::new(Mutex::new(user_chain));
        let miner_p = Arc::new(Mutex::new(user_miner)); 
        let tx_pool_p  = Arc::new(Mutex::new(user_txPool));

        let nakamoto = Nakamoto { chain_p, miner_p, network_p, tx_pool_p, trans_tx };
        let (puzzle, block) = create_puzzle(chain_p, tx_pool_p, user_config.max_tx_in_one_block, user_config.mining_reward_receiver);

        //when receive a block, check if the solution found by another node satisfies the solution?
        thread::spawn(move || {
            loop {
                let block_result = block_rx.recv();
                let cancellation_token_clone = cancellation_token.clone();
                match block_result {
                    Ok(block) => { 
                        chain_p.lock().unwrap().add_block(block, user_config.difficulty_leading_zero_len);
                        block_tx.send(block);
                        let pending_txs = chain_p.lock().unwrap().get_pending_finalization_txs();
                        tx_pool_p.lock().unwrap().filter_tx(user_config.max_tx_in_one_block, &pending_txs);
                        *cancellation_token_clone.write().unwrap() = verify_solution(block, puzzle, user_config.difficulty_leading_zero_len_acc);
                    },
                    Err(_) => { continue; },
                }
            }
        });

        thread::spawn(move || {
            loop {
                let tx_result = trans_rx.recv();
                match tx_result {
                    Ok(tx) => {
                        let is_added: bool = tx_pool_p.lock().unwrap().add_tx(tx);
                        if is_added {
                            Self::publish_tx(&mut nakamoto, tx);
                            trans_tx.send(tx);
                        }
                    }
                    Err(_) => { continue; }
                }
            }
        });

        // need thread to control miner

        let miner_thread = thread::spawn(move || {
            loop {
                let cancellation_token_clone = cancellation_token.clone();
                let solution = Miner::solve_puzzle(
                    miner_p, 
                    puzzle, 
                    user_config.nonce_len, 
                    user_config.difficulty_leading_zero_len, 
                    user_config.miner_thread_count, 
                    user_config.miner_thread_0_seed, 
                    cancellation_token.clone()
                );
                match solution {
                    Some(PuzzleSolution { puzzle, nonce, hash }) => {
                        println!("Solution Found! HASH: {}\nNONCE: {}\nPUZZLE: {}", hash, nonce, puzzle);
                    }, //solution found
                    None => {println!("Solution found by another miner");}
                };
                *cancellation_token_clone.write().unwrap() = false; //set cancellation token back to false to solve next puzzle
                (puzzle, block) = create_puzzle(chain_p, tx_pool_p, user_config.max_tx_in_one_block, user_config.mining_reward_receiver);
                //above is to create a new puzzle once a solution is found
            }

        });

        nakamoto
    }

    /// Get the status of the network as a dictionary of strings. For debugging purpose.
    pub fn get_network_status(&self) -> BTreeMap<String, String> {
        self.network_p.lock().unwrap().get_status()
    }

    /// Get the status of the chain as a dictionary of strings. For debugging purpose.
    pub fn get_chain_status(&self) -> BTreeMap<String, String> {
        self.chain_p.lock().unwrap().get_status()
    }

    /// Get the status of the transaction pool as a dictionary of strings. For debugging purpose.
    pub fn get_txpool_status(&self) -> BTreeMap<String, String> {
        self.tx_pool_p.lock().unwrap().get_status()
    }

    /// Get the status of the miner as a dictionary of strings. For debugging purpose.
    pub fn get_miner_status(&self) -> BTreeMap<String, String> {
        self.miner_p.lock().unwrap().get_status()
    }

    /// Publish a transaction to the Blockchain
    pub fn publish_tx(&mut self, transaction: Transaction) -> () {
        // Please fill in the blank
        // Add the transaction to the transaction pool and send it to the broadcast channel
        self.tx_pool_p.lock().unwrap().add_tx(transaction);
        self.trans_tx.send(transaction);
    }

    /// Get the serialized chain as a json string. 
    pub fn get_serialized_chain(&self) -> String {
        let chain = self.chain_p.lock().unwrap().clone();
        serde_json::to_string_pretty(&chain).unwrap()
    }

    /// Get the serialized transaction pool as a json string.
    pub fn get_serialized_txpool(&self) -> String {
        let tx_pool = self.tx_pool_p.lock().unwrap().clone();
        serde_json::to_string_pretty(&tx_pool).unwrap()
    }

    //Past this point are functions I made myself. Correctness now sits on the fence.

    pub fn get_block(&self, address : String) -> String {
        let the_block = self.chain_p.lock().unwrap().clone().get_block(address).unwrap();
        serde_json::to_string_pretty(&the_block).unwrap()
    }

    pub fn get_balance(&self, address : String) -> i64 {
        return self.chain_p.lock().unwrap().clone().get_balance(address);
    }
}

