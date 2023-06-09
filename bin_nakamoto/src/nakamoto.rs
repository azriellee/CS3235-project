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
use lib_chain::block::{BlockTree, Puzzle, Transactions, MerkleTree, BlockNode, BlockNodeHeader, Transaction};
use lib_miner::miner::{Miner, PuzzleSolution};
use lib_tx_pool::pool::TxPool;
use lib_network::p2pnetwork::{P2PNetwork};
use lib_network::netchannel::{NetAddress};
use std::collections::{HashMap, BTreeMap};
use std::time::{SystemTime, UNIX_EPOCH};

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
    if chain_p.lock().unwrap().finalized_tx_ids.len() > 0 {
        // get the last finalized block for removing finalized transactions
        // technically i can just call the function on each new finalized block?
        let genesis_id = chain_p.lock().unwrap().root_id.clone();
        let finalized_blocks = chain_p.lock().unwrap().get_finalized_blocks_since(genesis_id);
        tx_pool_p.lock().unwrap().remove_txs_from_finalized_blocks(&finalized_blocks);
    }
    let mut unfinalised_tx: Vec<Transaction>;

    // Loop for every 0.5secs to see if there is any new transaction
    loop {
        let mut excluding_tx: Vec<Transaction> = Vec::new();
        // Get transactions that are part of blocks 
        for block_node in chain_p.lock().unwrap().all_blocks.values() {
            let block_transactions = block_node.transactions_block.transactions.clone();
            excluding_tx.extend(block_transactions);
        }
        /*
        // Get blocks that are already removed from the pool
        for tx_id in tx_pool_p.lock().unwrap().removed_tx_ids.iter() {
            let tx = tx_pool_p.lock().unwrap().pool_tx_map[tx_id].clone();
            excluding_tx.push(tx);
        } */
        // Get the transactions that are not included in finalised blocks or are already removed in tx_pool
        unfinalised_tx = tx_pool_p.lock().unwrap().filter_tx(tx_count, &excluding_tx);

        if unfinalised_tx.len() > 0 {
            break;
        }
        thread::sleep(Duration::from_millis(500)); //wait 0.5s if no new transactions
    }

    // Create the blocknode
    let (merkle_root, merkle_tree) = MerkleTree::create_merkle_tree(unfinalised_tx.clone());

    let new_transactions = Transactions {
        merkle_tree,
        transactions: unfinalised_tx.clone(),
    };

    let new_blocknode_header = BlockNodeHeader {
        parent: chain_p.lock().unwrap().working_block_id.to_string(),
        merkle_root: merkle_root.to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        block_id: String::new(),
        nonce: String::new(),
        reward_receiver: reward_receiver.to_string(),
    };

    let new_blocknode = BlockNode {
        header: new_blocknode_header,
        transactions_block: new_transactions,
    };

    // Create the puzzle
    let new_puzzle = Puzzle {
        parent: chain_p.lock().unwrap().working_block_id.to_string(),
        merkle_root: merkle_root.to_string(),
        reward_receiver: reward_receiver.to_string(),
    };
    let new_puzzle_json = serde_json::to_string(&new_puzzle).unwrap();

    (new_puzzle_json, new_blocknode)



}

/// The struct to represent the Nakamoto instance.
/// The Nakamoto instance contains the chain, the miner, the network and the tx pool as smart pointers.
/// It also contains a FIFO channel for sending transactions to the Blockchain
#[derive(Clone)] 
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

        let user_config: Config = serde_json::from_str(&config_str.as_str()).unwrap(); 
        let user_chain: BlockTree = serde_json::from_str(&chain_str.as_str()).unwrap();
        let user_txpool: TxPool = serde_json::from_str(&tx_pool_str.as_str()).unwrap();
        let user_miner: Miner = Miner::new();
 
        let cancellation_token = Arc::new(RwLock::new(false));
        
        let (network_p, 
            block_rx, 
            trans_rx, 
            blockid_rx,
            block_tx, 
            trans_tx, 
            blockid_tx)
             = P2PNetwork::create(user_config.addr, user_config.neighbors);

        let chain_p = Arc::new(Mutex::new(user_chain));
        let miner_p = Arc::new(Mutex::new(user_miner)); 
        let tx_pool_p  = Arc::new(Mutex::new(user_txpool));
        
        let nakamoto = Nakamoto { chain_p, miner_p, network_p, tx_pool_p, trans_tx };

        let chain_p_block = Arc::clone(&nakamoto.chain_p);
        // let tx_pool_p_block = Arc::clone(&nakamoto.tx_pool_p);
        let block_tx_block = block_tx.clone();
        let chain_p = nakamoto.chain_p.clone();

        // when receiving a block from neighbor
        let cancellation_token_clone = cancellation_token.clone();
        thread::spawn(move || {
            loop {
                // recv_timeout will get an error every 10 seconds when nothing is received REMOVED
                let block_result = block_rx.recv();

                match block_result {
                    Ok(block) => { 
                        // if the block already exists in our block tree, do not broadcast?
                        let incoming_block_id = block.header.block_id.to_string();
                        let has_existing_block_result = chain_p_block.lock().unwrap().all_blocks.contains_key(&incoming_block_id);
                        if has_existing_block_result {
                            continue;
                        }
                        // add block to the blocktree, broadcasts block
                        let initial_working_block = chain_p.lock().unwrap().working_block_id.clone();
                        chain_p.lock().unwrap().add_block(block.clone(), user_config.difficulty_leading_zero_len_acc);
                        block_tx_block.send(block.clone()).unwrap(); //possible err?

                        // check if client has parent for that block, if no request the parent block
                        let incoming_block_parent_id = block.header.parent.to_string();
                        let has_parent_block_result = chain_p_block.lock().unwrap().all_blocks.contains_key(&incoming_block_parent_id);
                        if !has_parent_block_result {
                            blockid_tx.send(incoming_block_parent_id.clone()).unwrap(); //possible err?
                        }

                        // if the working block is updated while miner is solving (a new block), restart the miner
                        let cur_working_block = chain_p.lock().unwrap().working_block_id.clone();
                        if initial_working_block != cur_working_block {
                            *cancellation_token_clone.write().unwrap() = true;
                        }
                        
                    },
                    Err(_) => { continue; },
                }
            }
        });

        let mut nakamoto_clone = nakamoto.clone();
        // when receiving a transaction from neighbor
        thread::spawn(move || {
            loop {
                // recv_timeout will get an error every 10 seconds when nothing is received REMOVED
                let tx_result = trans_rx.recv();
                match tx_result {
                    Ok(tx) => {
                        // if the transaction already exists in our tx_pool, do not broadcast?
                        let has_existing_tx = nakamoto_clone.tx_pool_p.lock().unwrap().pool_tx_ids.contains(&tx.gen_hash());
                        if has_existing_tx {
                            continue;
                        }

                        // Nakamoto::stdout_notify(format!("Received a new transaction: {}", tx.gen_hash()));
                        nakamoto_clone.publish_tx(tx);
                        
                    }
                    Err(_) => { continue; }
                }
            }
        });

        let chain_p_blockid = Arc::clone(&nakamoto.chain_p);
        let blockid_tx_block = block_tx.clone();

        // when receiving a request block id from neighbor
        thread::spawn(move || {
            loop {
                // recv_timeout will get an error every 10 seconds when nothing is received REMOVED
                let id_result = blockid_rx.recv();
                match id_result {
                    Ok(id) => {
                        // get block
                        let block_result = chain_p_blockid.lock().unwrap().get_block(id);
                        match block_result {
                            Some(block) => { blockid_tx_block.send(block).unwrap(); },
                            None => { continue; }
                        }
                    },
                    Err(_) => { continue; },
                }
            }
        });

        let miner_p_puzzle = Arc::clone(&nakamoto.miner_p);
        let chain_p_puzzle = Arc::clone(&nakamoto.chain_p);
        let tx_pool_p_puzzle = Arc::clone(&nakamoto.tx_pool_p);
        let block_tx_puzzle = block_tx.clone();

        // create a miner to solve puzzle
        let _miner_thread = thread::spawn(move || {
            loop {
                // constantly creates a puzzle from tx pool
                let (puzzle_json, mut block) = create_puzzle(
                    chain_p_puzzle.clone(), 
                    tx_pool_p_puzzle.clone(), 
                    user_config.max_tx_in_one_block, 
                    user_config.mining_reward_receiver.to_string()
                );

                let cancellation_token_clone = cancellation_token.clone();
                
                // Nakamoto::stdout_notify(format!("Mining on a block: {}", block.header.block_id.to_string()));
                let solution = Miner::solve_puzzle(
                    miner_p_puzzle.clone(), 
                    puzzle_json.to_string(), 
                    user_config.nonce_len, 
                    user_config.difficulty_leading_zero_len, 
                    user_config.miner_thread_count, 
                    user_config.miner_thread_0_seed, 
                    cancellation_token.clone()
                );
                match solution {
                    Some(PuzzleSolution {puzzle: _, nonce, hash }) => {
                        //solution found, update the block and broadcast
                        Nakamoto::stdout_notify(format!("Solution found for block: {}", hash.to_string()));

                        block.header.block_id = hash;
                        block.header.nonce = nonce;
                        block_tx_puzzle.send(block.clone()).unwrap();
                        chain_p_puzzle.lock().unwrap().add_block(block.clone(),user_config.difficulty_leading_zero_len);
                    },
                    None => {
                        Nakamoto::stdout_notify(format!("Block found by another miner."));
                }
                };
                *cancellation_token_clone.write().unwrap() = false; //set cancellation token back to false to solve next puzzle
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
        let is_added = self.tx_pool_p.lock().unwrap().add_tx(transaction.clone());
        if is_added {
            self.trans_tx.send(transaction).unwrap(); //possible err?
        }
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
}
