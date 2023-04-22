// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

/// This is the main file of the bin_nakamoto executable.
/// It is a simple command-line program that can be used to interact with the Blockchain
/// It reads commands from stdin and writes responses to stdout to facilitate IPC communication with bin_client eventually.
/// However, you can also run it directly from the command line to test it.
/// You can see detailed instructions in the comments below.

mod nakamoto;
use lib_chain::block::{Transaction, Signature, BlockTree}; 
use lib_tx_pool::pool::{TxPool}; 
use lib_network::netchannel::{NetAddress};
use nakamoto::Nakamoto;

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::fs;
use serde::{Serialize, Deserialize};

// Read a string from a file (to help you debug)
fn read_string_from_file(filepath: &str) -> String {
    let contents = fs::read_to_string(filepath)
        .expect(&("Cannot read ".to_owned() + filepath));
    contents
}

// Append a string to a file (to help you debug)
fn append_string_to_file(filepath: &str, content: String) {
    // if not exists, create file
    if !std::path::Path::new(filepath).exists() {
        fs::File::create(filepath).unwrap();
    }
    fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(filepath)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();
}

/// This enum represents IPC messsage requests from the stdin
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReq {
    /// Initialize the Nakamoto instance using the given (blocktree_json, tx_pool_json, config_json)
    Initialize(String, String, String),
    /// Get the balance of the given address (user_id)
    GetAddressBalance(String),
    /// Publish a transaction to the network (data_string, signature)
    PublishTx(String, Signature),
    /// Get the block data of the given block_id
    RequestBlock(String),
    /// Get the network status (for debugging)
    RequestNetStatus,
    /// Get the chain status (for debugging)
    RequestChainStatus,
    /// Get the miner status (for debugging)
    RequestMinerStatus,
    /// Get the tx pool status (for debugging)
    RequestTxPoolStatus,
    /// Get the state serialization (including BlockTree and TxPool)
    RequestStateSerialization,
    /// Quit the program
    Quit,
}

/// This enum represents IPC messsage responses to the stdout
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageResp {
    /// The Nakamoto instance has been initialized (responding to Initialize)
    Initialized,
    /// The transaction has been published (responding to PublishTx)
    PublishTxDone,
    /// The balance of the given address (user_id, balance)
    AddressBalance(String, i64),
    /// The block data of the given block_id (block_data)
    BlockData(String),
    /// The network status as a dictionary of strings (for debugging)
    NetStatus(BTreeMap<String, String>),
    /// The chain status as a dictionary of strings (for debugging)
    ChainStatus(BTreeMap<String, String>),
    /// The miner status as a dictionary of strings (for debugging)
    MinerStatus(BTreeMap<String, String>),
    /// The tx pool status as a dictionary of strings (for debugging)
    TxPoolStatus(BTreeMap<String, String>),
    /// The state serialization (blocktree_json_string, tx_pool_json_string)
    StateSerialization(String, String),
    /// The program is quitting (responding to Quit)
    Quitting,
    /// This is not an actual response, but an arbitrary notification message for debugging
    Notify(String), 
}

fn main() {
    // bin_nakamoto has only one optional argument: the path to the seccomp policy file
    // If the argument is provided, bin_nakamoto will read and apply the seccomp policy at the beginning of the program
    // Otherwise, it will proceed to the normal execution
    let maybe_policy_path = std::env::args().nth(1);
    if let Some(policy_path) = maybe_policy_path {
        // Please fill in the blank
        // If the first param is provided, read the seccomp config and apply it
        
    }

    // The main logic of the bin_nakamoto starts here
    // It reads IPC calls from stdin and write IPC responses to stdout in a loop.
    // The first IPC call should be Initialize, whose parameters are serialized BlockTree, TxPool, and Config.
    // After that, there can be artitrary number of IPC calls, including GetAddressBalance, PublishTx, RequestBlock, RequestNetStatus, RequestChainStatus, RequestMinerStatus, RequestTxPoolStatus, RequestStateSerialization, etc.
    // Eventually, the program will quit when receiving a Quit IPC call.
    // Please fill in the blank
    // Loop over stdin and handle IPC messages
    
    //println!("StartCheck!!");
    println!("{}", serde_json::to_string(&IPCMessageResp::Notify("# Waiting for IPC Requests ...".to_string())).unwrap());
    let mut raw_data = String::new();
    let mut nakamoto_node : Nakamoto;

    //Initialise
    io::stdin().read_line(&mut raw_data).expect("wtf"); //need handle err here
    let parsed_input : IPCMessageReq = serde_json::from_str(raw_data.as_str()).unwrap(); 
    println!("{}", serde_json::to_string(&IPCMessageResp::Notify("[Main] Start receiving trans thread".to_string())).unwrap());
    match parsed_input {
        IPCMessageReq::Initialize(chain_info, tx_info, config_info) => {
            nakamoto_node = Nakamoto::create_nakamoto(chain_info, tx_info, config_info);
            println!("{}", serde_json::to_string(&IPCMessageResp::Initialized).unwrap());
        }
        _ => return, //Terminate if first thing is not an IPCMessageReq::Initialize(x,y,z);
    }
    
    let mut is_running : bool = true;
    while is_running {
        // println!("Reading In Progress...");
        raw_data = "".to_string();
        io::stdin().read_line(&mut raw_data).expect("wtf"); //need handle err here

        //println!("What is read: \n{}\n", raw_data);

        let parsed_input: IPCMessageReq = serde_json::from_str(raw_data.as_str()).unwrap(); //need handle err here
        //Does the matching.
        match parsed_input {
            IPCMessageReq::RequestChainStatus => {
                //println!("{}", serde_json::to_string(&IPCMessageResp::Notify("printing chain status".to_string())).unwrap());
                let status = nakamoto_node.get_chain_status();
                println!("{}", serde_json::to_string(&IPCMessageResp::ChainStatus(status)).unwrap());
            }
            IPCMessageReq::RequestMinerStatus => {
                //println!("{}", serde_json::to_string(&IPCMessageResp::Notify("printing miner status".to_string())).unwrap());
                let status = nakamoto_node.get_miner_status();
                println!("{}", serde_json::to_string(&IPCMessageResp::MinerStatus(status)).unwrap());
            }
            IPCMessageReq::RequestNetStatus => {
                //println!("{}", serde_json::to_string(&IPCMessageResp::Notify("printing net status".to_string())).unwrap());
                let status = nakamoto_node.get_network_status();
                println!("{}", serde_json::to_string(&IPCMessageResp::NetStatus(status)).unwrap());
            }
            IPCMessageReq::RequestTxPoolStatus => {
                //println!("{}", serde_json::to_string(&IPCMessageResp::Notify("printing pool status".to_string())).unwrap());
                let status = nakamoto_node.get_txpool_status();
                println!("{}", serde_json::to_string(&IPCMessageResp::TxPoolStatus(status)).unwrap());
            }
            IPCMessageReq::GetAddressBalance(user_id) => {
                //println!("{}", serde_json::to_string(&IPCMessageResp::Notify("printing balance".to_string())).unwrap());
                let user_balance = nakamoto_node.chain_p.lock().unwrap().get_balance(user_id.to_string());
                println!("{}", serde_json::to_string(&IPCMessageResp::AddressBalance(user_id.to_string(), user_balance)).unwrap());
            }
            IPCMessageReq::PublishTx(data, sig) => {
                //println!("{}", serde_json::to_string(&IPCMessageResp::Notify("Published has arrived, doing now".to_string())).unwrap());
                let tx_info: Vec<String> = serde_json::from_str(&data).unwrap();
                let tx_sig: String = sig;

                let transaction = Transaction {
                    sender: tx_info[0].to_string(),
                    receiver: tx_info[1].to_string(),
                    message: tx_info[2].to_string(),
                    sig: tx_sig,
                };

                let is_verified = transaction.verify_sig();
                if is_verified {
                    nakamoto_node.publish_tx(transaction);
                    println!("{}", serde_json::to_string(&IPCMessageResp::PublishTxDone).unwrap());
                }
            }
            IPCMessageReq::RequestBlock(block_id) => {
                let block_node = nakamoto_node.chain_p.lock().unwrap().get_block(block_id).unwrap(); //need handle error
                println!("{}", serde_json::to_string(&IPCMessageResp::BlockData(serde_json::to_string(&block_node).unwrap())).unwrap());
            }
            IPCMessageReq::RequestStateSerialization => {
                let chain_info = nakamoto_node.get_serialized_chain();
                let txpool_info = nakamoto_node.get_serialized_txpool();
                println!("{}", serde_json::to_string(&IPCMessageResp::StateSerialization(chain_info, txpool_info)).unwrap());
            }
            IPCMessageReq::Quit => {
                is_running = false;
                println!("{}\n", serde_json::to_string(&IPCMessageResp::Quitting).unwrap());
            }
            _ => {
                println!("{}", serde_json::to_string(&IPCMessageResp::Notify(raw_data)).unwrap());
            }
        }    
    }
}

