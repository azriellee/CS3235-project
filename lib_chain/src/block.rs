// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// This file contains the definition of the BlockTree
/// The BlockTree is a data structure that stores all the blocks that have been mined by this node or received from other nodes.
/// The longest path in the BlockTree is the main chain. It is the chain from the root to the working_block_id.

use core::panic;
use std::{collections::{BTreeMap, HashMap, HashSet}, convert, str::FromStr};
use base64ct::{Base64, Encoding};
use rsa::{pkcs1::DecodeRsaPublicKey, pkcs1v15::VerifyingKey};
use serde::{Serialize, Deserialize};
use serde_json::json;
use sha2::{Sha256, Digest, digest::block_buffer::Block};
use rsa::signature::{Verifier};

const PUBLIC_KEY_BEGIN: &str = "-----BEGIN RSA PUBLIC KEY-----\n";
const PUBLIC_KEY_END: &str = "-----END RSA PUBLIC KEY-----\n";

pub type UserId = String;
pub type BlockId = String;
pub type Signature = String;
pub type TxId = String;

/// Merkle tree is used to verify the integrity of transactions in a block.
/// It is generated from a list of transactions. It will be stored inside `Transactions` struct.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)] 
pub struct MerkleTree {
    /// A list of lists of hashes, where the first list is the list of hashes of the transactions,
    /// the second list is the list of hashes of the first list, and so on. 
    /// See the `create_merkle_tree` function for more details.
    pub hashes: Vec<Vec<String>>
}

impl MerkleTree {    
    /// Create a merkle tree from a list of transactions.
    /// The merkle tree is a list of lists of hashes, 
    /// where the first list is the list of hashes of the transactions.
    /// The last list is the list with only one hash, called the Merkle root.
    /// - `txs`: a list of transactions
    /// - The return value is the root hash of the merkle tree
    pub fn create_merkle_tree (txs: Vec<Transaction>) -> (String, MerkleTree) {
        if txs.len() == 0 {
            panic!("create_merkel_tree get empty Transaction Vector.");
        }

        // In MerkleTree.hashes, each level contains a list of transaction hashes
        // E.g. 4 transactions, level 1:4, level 2:2, level 3:1

        let mut hashes: Vec<Vec<String>> = Vec::new();
        
        // In level 1, convert each transaction into hashes
        let mut init_hashes = Vec::new();
        
        for tx in txs.iter() {
            init_hashes.push(tx.gen_hash());
        }

        hashes.push(init_hashes);

        // Subsequent levels, hash for every 2 hashes
        loop {
            let mut curr_level: Vec<String> = Vec::new();
            let last_level = hashes.len() - 1;
            let no_of_leaf = hashes[last_level].len();
            let no_of_iterations = ((no_of_leaf / 2) as f32).ceil() as i32;

            for i in 0..no_of_iterations {
                let leaf1 = hashes[last_level].get((i * 2) as usize).unwrap();
                let leaf2 = hashes[last_level].get((i * 2 + 1) as usize).unwrap_or_else(|| leaf1);

                let hash = Self::gen_hash_strings(leaf1, leaf2);
                curr_level.push(hash);
            }

            hashes.push(curr_level);
            
            // do-while loop
            if no_of_iterations == 1 {
                break;
            }
        }

        // Finally, initalize merkle tree
        let merkle_tree = MerkleTree{hashes};
        let root = merkle_tree.hashes[merkle_tree.hashes.len() - 1][0].clone();
        
        (root, merkle_tree)
        
    }

    // Please fill in the blank
    // Depending on your implementation, you may need additional functions here.

    fn gen_hash_strings(s1: &String, s2: &String) -> String {
        let s = format!("{}{}", s1, s2);
        let mut hasher = Sha256::new();
        hasher.update(s);

        format!("{:x}", hasher.finalize())
    }
    
}

/// The struct containing a list of transactions and the merkle tree of the transactions. 
/// Each block will contain one `Transactions` struct.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)] 
pub struct Transactions {
    /// The merkle tree of the transactions
    pub merkle_tree: MerkleTree,
    /// A list of transactions
    pub transactions: Vec<Transaction>
}


/// The struct is used to store the information of one transaction.
/// The transaction id is not stored explicitly, but can be generated from the transaction using the `gen_hash` function.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)] 
pub struct Transaction {
    /// The user_id of the sender
    pub sender: UserId,
    /// The user_id of the receiver
    pub receiver: UserId,
    /// The message of the transaction. 
    /// The expected format is `SEND $300   // By Alice   // 1678173972743`, 
    /// where `300` is the amount of money to be sent,
    /// and the part after the first `//` is the comment: `Alice` is the friendly name of the sender, and `1678173972743` is the timestamp of the transaction.
    /// The comment part does not affect the validity of the transaction nor the computation of the balance.
    pub message: String,
    /// The signature of the transaction in base64 format
    pub sig: Signature
}

impl Transaction {
    /// Create a new transaction struct given the sender, receiver, message, and signature.
    pub fn new(sender: UserId, receiver: UserId, message: String, sig: Signature) -> Transaction {
        Transaction { 
            sender, 
            receiver, 
            message,
            sig
        }
    }

    /// Compute the transaction id from the transaction. The transaction id is the sha256 hash of the serialized transaction struct in hex format.
    pub fn gen_hash(&self) -> TxId {
        let mut hasher = Sha256::new();
        let hasher_str = serde_json::to_string(&self).unwrap();
        hasher.update(hasher_str);
        let result = hasher.finalize();
        let tx_hash: TxId = format!("{:x}", result);
        tx_hash
    }

    /// Verify the signature of the transaction. Return true if the signature is valid, and false otherwise.
    pub fn verify_sig(&self) -> bool {
        // Please fill in the blank
        // verify the signature using the sender_id as the public key (you might need to change the format into PEM)
        // You can look at the `verify` function in `bin_wallet` for reference. They should have the same functionality.
        let msg: [&String; 3] = [&self.sender, &self.receiver, &self.message];
        let msg_json: String = json!(msg).to_string();

        // Make sender public key string into RSA public key
        let public_key = Self::generate_pub_key_pem(&self.sender.as_str());
        let verifying_key = VerifyingKey::<Sha256>::new(public_key);

        // Retrieve signature into RSA
        let signature = Base64::decode_vec(&self.sig).unwrap();
        let verify_signature = rsa::signature::Signature::from_bytes(&signature).unwrap();

        // Verify result
        let verify_result = verifying_key.verify(msg_json.as_bytes(), &verify_signature);

        return match verify_result {
            Ok(_) => true,
            Err(_) => false
        }
    }

    // Converts public key string into RSA Public Key with PEM format
    fn generate_pub_key_pem(public_key: &str) -> rsa::RsaPublicKey {
        let key = public_key.chars()
            .collect::<Vec<char>>()
            .chunks(64)
            .map(|c| c.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n") + "\n";

        rsa::RsaPublicKey::from_pkcs1_pem(format!("{}{}{}", PUBLIC_KEY_BEGIN, key, PUBLIC_KEY_END).as_str()).unwrap()
    }
}


/// The struct representing a whole block tree.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockTree {
    /// A map from block id to the block node
    pub all_blocks: HashMap<BlockId, BlockNode>,
    /// A map from block id to the list of its children (as block ids)
    pub children_map: HashMap<BlockId, Vec<BlockId>>,
    /// A map from block id to the depth of the block. The genesis block has depth 0.
    pub block_depth: HashMap<BlockId, u64>, 
    /// The id of the root block (the genesis block)
    pub root_id: BlockId,
    /// The id of the working block (the block at the end of the longest chain)
    pub working_block_id: BlockId,
    /// A map to bookkeep the orphan blocks. 
    /// Orphan blocks are blocks whose parent are not in the block tree yet.
    /// They should be added to the block tree once they can be connected to the block tree.
    pub orphans: HashMap<BlockId, BlockNode>,
    /// The id of the latest finalized block
    pub finalized_block_id: BlockId,
    /// A map from the user id to its balance
    pub finalized_balance_map: HashMap<UserId, i64>,
    /// A set of transaction ids that have been finalized. It includes all the transaction ids in the finalized blocks.
    pub finalized_tx_ids: HashSet<TxId>
}

impl BlockTree {
    /// Create a new block tree with the genesis block as the root.
    pub fn new () -> BlockTree {
        let mut bt = BlockTree { 
            all_blocks: HashMap::new(), 
            children_map: HashMap::new(), 
            block_depth: HashMap::new(), 
            root_id: String::new(), 
            working_block_id: String::new(), 
            orphans: HashMap::new(),
            finalized_block_id: String::new(),
            finalized_balance_map: HashMap::new(),
            finalized_tx_ids: HashSet::new()
        };
        let genesis_block = BlockNode::genesis_block();
        bt.all_blocks.insert("0".to_string(), genesis_block.clone());
        bt.block_depth.insert("0".to_string(), 0);
        bt.root_id = "0".to_string();
        bt.working_block_id = "0".to_string();
        for tx in genesis_block.transactions_block.transactions {
            let amount = tx.message.split(" ").collect::<Vec<&str>>()[1].trim_start_matches('$').parse::<i64>().unwrap();
            bt.finalized_balance_map.insert(tx.receiver, amount);
        }
        bt.finalized_block_id = "0".to_string();
        bt
    }

    /// Add a block to the block tree. If the block is not valid to be added to the tree
    /// (i.e. it does not satsify the conditions below), ignore the block. Otherwise, add the block to the BlockTree.
    /// 
    /// 1. The block must have a valid nonce and the hash in the puzzle solution satisfies the difficulty requirement.
    /// 2. The block_id of the block must be equal to the computed hash in the puzzle solution.
    /// 3. The block does not exist in the block tree or the orphan map.
    /// 4. The transactions in the block must be valid. See the `verify_sig` function in the `Transaction` struct for details.
    /// 5. The parent of the block must exist in the block tree. 
    ///     Otherwise, it will be bookkeeped in the orphans map. 
    ///     When the parent block is added to the block tree, the block will be removed from the orphan map and checked against the conditions again.
    /// 6. The transactions in the block must not be duplicated with any transactions in its ancestor blocks.
    /// 7. Each sender in the txs in the block must have enough balance to pay for the transaction.
    ///    Conceptually, the balance of one address is the sum of the money sent to the address minus the money sent from the address 
    ///    when walking from the genesis block to this block, according to the order of the txs in the blocks.
    ///    Mining reward is a constant of $10 (added to the reward_receiver address **AFTER** considering transactions in the block).
    /// 
    /// When a block is successfully added to the block tree, update the related fields in the BlockTree struct 
    /// (e.g., working_block_id, finalized_block_id, finalized_balance_map, finalized_tx_ids, block_depth, children_map, all_blocks, etc)
    pub fn add_block(&mut self, block: BlockNode, leading_zero_len: u16) -> () {
        // Please fill in the blank
        // 1. Check if nonce exists, and leading zero is the same in block_id
        // 2. Create a struct puzzle with the block, serialize and concat with nonce, check if block id is correct
        todo!();
        
    }


    /// Get the block node by the block id if exists. Otherwise, return None.
    pub fn get_block(&self, block_id: BlockId) -> Option<BlockNode> {
        // Please fill in the blank
        todo!();
        
    }

    /// Get the finalized blocks on the longest path after the given block id, from the oldest to the most recent.
    /// The given block id should be any of the ancestors of the current finalized block id or the current finalized block id itself.
    /// If it is not the case, the function will panic (i.e. we do not consider inconsistent block tree caused by attacks in this project)
    pub fn get_finalized_blocks_since(&self, since_block_id: BlockId) -> Vec<BlockNode> {
        // Please fill in the blank
        todo!();
        
    }

    /// Get the pending transactions on the longest chain that are confirmed but not finalized.
    pub fn get_pending_finalization_txs(&self) -> Vec<Transaction> {
        // Please fill in the blank
        todo!();
        
    }

    /// Get status information of the BlockTree for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the BlockTree. 
        // It should be displayed in the Client UI eventually.
        todo!();
        
    }
}

/// The struct representing a puzzle for the miner to solve. The puzzle is to find a nonce such that when concatenated
/// with the serialized json string of this `Puzzle` struct, the sha256 hash of the result has the required leading zero length.
#[derive(Serialize)]
pub struct Puzzle {
    pub parent: BlockId,
    pub merkle_root: String,
    pub reward_receiver: UserId
}

/// The struct representing a block header. Each `BlockNode` has one `BlockNodeHeader`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockNodeHeader {
    /// The block id of the parent block.
    pub parent: BlockId,
    /// The merkle root of the transactions in the block.
    pub merkle_root: String,
    /// The timestamp of the block. For genesis block, it is 0. For other blocks, greater or equal to 1 is considered valid.
    pub timestamp: u64,
    /// The block id of the block (the block id is the sha256 hash of the concatination of the nonce and a `Puzzle` derived from the block)
    pub block_id: BlockId,
    /// The nonce is the solution found by the miner for the `Puzzle` derived from this block.
    pub nonce: String,
    /// The reward receiver of the block.
    pub reward_receiver: UserId,
}

/// The struct representing a block node.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BlockNode {
    /// The header of the block.
    pub header: BlockNodeHeader,
    /// The transactions in the block.
    pub transactions_block: Transactions,
}

impl BlockNode {
    /// Create the genesis block that contains the initial transactions 
    /// (give $299792458 to the address of Alice `MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==`)
    pub fn genesis_block() -> BlockNode {
        let header = BlockNodeHeader {
            parent: "0".to_string(),
            merkle_root: "0".to_string(),
            timestamp: 0,
            block_id: "0".to_string(),
            nonce: "0".to_string(),
            reward_receiver: "GENESIS".to_string(),
        };

        let transactions_block = Transactions {
            transactions: vec![
                Transaction::new("GENESIS".to_owned(), "MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==".to_string(), "SEND $299792458".to_owned(), "GENESIS".to_owned())],
            merkle_tree: MerkleTree { hashes: vec![] }, // Skip merkle tree generation for genesis block
        };

        BlockNode {
            header,
            transactions_block,
        }
    }

    /// Check for block validity based solely on this block (not considering its validity inside a block tree).
    /// Return a tuple of (bool, String) where the bool is true if the block is valid and false otherwise.
    /// The string is the re-computed block id.
    /// The following need to be checked:
    /// 1. The block_id in the block header is indeed the sha256 hash of the concatenation of the nonce and the serialized json string of the `Puzzle` struct derived from the block.
    /// 2. All the transactions in the block are valid.
    /// 3. The merkle root in the block header is indeed the merkle root of the transactions in the block.
    pub fn validate_block(&self, leading_zero_len: u16) -> (bool, BlockId) {
        // Please fill in the blank
        // Check if the block_id starting zeros match with leading_zero_len
        if !self.header.block_id.starts_with(&"0".repeat(leading_zero_len.into())) {
            return (false, self.header.block_id.clone());
        }

        // Serialize a puzzle, concat with nonce and compare hash. Return with bool and computed hash
        let puzzle = Puzzle {
            parent: self.header.parent.clone(),
            merkle_root: self.header.merkle_root.clone(),
            reward_receiver: self.header.reward_receiver.clone()
        };

        let concated_puzzle = format!("{}{}", self.header.nonce, serde_json::to_string(&puzzle).unwrap());
        let mut hasher = Sha256::new();
        hasher.update(concated_puzzle);
        let result = hasher.finalize();
        let hash: String = format!("{:x}", result);

        ((hash == self.header.block_id.clone()), hash)
    }
}

