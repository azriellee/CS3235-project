// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This is the main file for the bin_wallet binary
// It is a simple command-line program that can be used to sign and verify messages
// It reads from stdin and writes to stdout to facilitate IPC communication with bin_client eventually.
// However, you can run it directly from the command line to test it.
// You can see detailed instructions in the comments below.

mod wallet;
use std::fs;
use std::io;
use std::io::Write;
use std::os::linux::raw;
use serde::{Serialize, Deserialize};

use seccompiler;
use seccompiler::{BpfProgram, BpfMap};

/// Read a string from a file (help with debugging)
fn read_string_from_file(filepath: &str) -> String {
    let contents = fs::read_to_string(filepath)
        .expect(&("Cannot read ".to_owned() + filepath));
    contents
}

/// Write a string to a file (to help you debug)
fn write_string_to_file(filepath: &str, content: String) {
    fs::write(filepath, content).expect(&("Cannot write ".to_owned() + filepath));
}

/// Append a string to a file (to help you debug)
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

/// The enum representing IPC message requests from the stdin
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageReq {
    /// Quit the execution
    Quit,
    /// Initialize the wallet by deserializing the provided json string
    Initialize(String),
    /// Sign the provided data string using the private key
    SignRequest(String),
    /// Verify the provided (`data_string`, `signature_in_base64`) using the public key
    VerifyRequest(String, String),
    /// Get the user info
    GetUserInfo
}

/// The enum representing IPC message responses to the stdout
#[derive(Serialize, Deserialize, Debug, Clone)]
enum IPCMessageResp {
    /// The wallet has been initialized
    Initialized,
    /// The wallet is quitting normally
    Quitting,
    /// The response to a sign request (DataString, Signature)
    SignResponse(String, String),
    /// The response to a verify request (isSuccess, DataString)
    VerifyResponse(bool, String),
    /// The response to the get user info request (username, user_id). User Id is transformed from the public key.
    UserInfo(String, String)
}

fn main() {
    // bin_wallet has only one optional argument: the path to the seccomp policy file
    // If the argument is provided, bin_wallet will read and apply the seccomp policy at the beginning of the program
    // Otherwise, it will proceed to the normal execution
    
    // let maybe_policy_path = std::env::args().nth(1);
    // if let Some(policy_path) = maybe_policy_path {
    //     // Please fill in the blank
    //     // If the first param is provided, read the seccomp config and apply it
    //     let json_input = read_string_from_file(&policy_path);
    //     let filter_map: BpfMap = seccompiler::compile_from_json(
    //         json_input.as_bytes(),
    //         std::env::consts::ARCH.try_into().unwrap(),
    //     )
    //     .unwrap();
    //     let filter = filter_map.get("main_thread").unwrap() ;
    //     seccompiler::apply_filter(filter).unwrap();
    // }

    // The main logic of the bin_wallet starts here
    // It reads IPC calls from stdin and write IPC responses to stdout in a loop.
    // The first IPC call is always the Initialize call with the wallet data provided.
    // After that, there can be arbitrary number of SignRequest, VerifyRequest, and GetUserInfo calls.
    // Eventually, the Quit call will be received and the program will exit.
    
    //use serde_json::Result;
    use wallet::Wallet;
    //println!("StartCheck");
    let mut raw_data = String::new();
    //let mut parsedInput : IPCMessageReq = serde_json::from_str(rawData)?; 
    let user_wallet : Wallet;

    //Initialise
    io::stdin().read_line(&mut raw_data).expect("wtf");
    let parsed_input : IPCMessageReq = serde_json::from_str(raw_data.as_str()).unwrap(); 
    match parsed_input {
        IPCMessageReq::Initialize(init_info) => {
            user_wallet = serde_json::from_str(&init_info.as_str()).unwrap();
            println!("{}", serde_json::to_string(&IPCMessageResp::Initialized).unwrap());
        }
        _ => return, //Terminate if first thing is not an IPCMessageReq::Initialize();
    }
    
    let mut is_running: bool = true;
    while is_running {
        //Read the damn input. Expected to fit IPCMessageReq.
        //println!("Reading In Progress...\n");
        raw_data = "".to_string();
        io::stdin().read_line(&mut raw_data).expect("Failed to read input.");
        //println!("What is read: \n{}\n", raw_data);
        let parsed_input : IPCMessageReq = serde_json::from_str(raw_data.as_str()).unwrap(); 
        //println!("Matching In Progress...\n");
        //Does the matching.
        match parsed_input {
            IPCMessageReq::GetUserInfo => {
                //println!("Executing GetUserInfo\n");
                let user_name = user_wallet.get_user_name();
                let user_id = user_wallet.get_user_id();
                println!("{}", serde_json::to_string(&IPCMessageResp::UserInfo(user_name, user_id)).unwrap());
            }
            IPCMessageReq::SignRequest(first_string) => {
                let signature = user_wallet.sign(first_string.as_str());
                println!("{}", serde_json::to_string(&IPCMessageResp::SignResponse(first_string, signature)).unwrap());
            }
            IPCMessageReq::VerifyRequest(data_string, signature_string) => {
                let is_correct : bool = user_wallet.verify(data_string.as_str(), signature_string.as_str());
                println!("{}", serde_json::to_string(&IPCMessageResp::VerifyResponse(is_correct, data_string)).unwrap());
            }
            IPCMessageReq::Quit => {
                is_running = false;
                println!("{}\n", serde_json::to_string(&IPCMessageResp::Quitting).unwrap());
            }
            _ => panic!(),
        }

    }
 
    // Please fill in the blank
    //todo!();
    
    
}

#[cfg(test)]
mod test {
    use crate::{wallet::Wallet, write_string_to_file, IPCMessageReq, IPCMessageResp, read_string_from_file};

    /// This test generates a new wallet and writes it to a file.
    #[test]
    fn generate_new_wallet() {
        let bin_wallet = Wallet::new("Haha".to_string(), 384);
        let bin_wallet_str = serde_json::to_string_pretty(&bin_wallet).unwrap();
        write_string_to_file("../tests/_secrets/Wallet.new.json", bin_wallet_str);
    }

    /// This test reads a wallet from a file and uses it to sign and verify a message.
    #[test]
    fn test_bin_wallet_signing_and_verifying() {
        let bin_wallet: Wallet = serde_json::from_str(&read_string_from_file("../tests/_secrets/Wallet.A.json")).unwrap();
        println!("Private key Pem:\n{}\n", bin_wallet.priv_key_pem);
        println!("Public key Pem:\n{}\n", bin_wallet.pub_key_pem);
        let msg = "hello world";
        let sig64 = bin_wallet.sign(msg);

        let verify_result = bin_wallet.verify(msg, &sig64);
        println!("msg: {}\nsig64: {}\nverify: {}", msg, sig64, verify_result);

    }

    /// This test reads a wallet from a file and uses it to verify a message signed by a reference implementation.
    #[test] 
    fn test_bin_wallet_verifying_alice() {
        let msg = "[\"MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ==\",\"MDgCMQDOpK8YWmcg8ffNF/O7xlBDq/DBdoUnc4yyWrV0y/X3LF+dddjaGksXzGl3tHskpgkCAwEAAQ==\",\"SEND $300   // By Alice   // 1678250102871\"]".to_string();
        let sig = "l8gsKxmAUzhgqbVqGlXaO69+Qhr87QthvZjUbYZXvnb+tanxCi8wm3c5UjHZ+HKm".to_string();
        let bin_wallet: Wallet = serde_json::from_str(&read_string_from_file("../tests/_secrets/Wallet.A.json")).unwrap();
        let verify_result = bin_wallet.verify(&msg, &sig);
        println!("msg: {}\nsig64: {}\nverify: {}", msg, sig, verify_result);
        assert!(verify_result);

        let sig2 = "58gsKxmAUzhgqbVqGlXaO69+Qhr87QthvZjUbYZXvnb+tanxCi8wm3c5UjHZ+HKm".to_string();
        let verify_result = bin_wallet.verify(&msg, &sig2);
        assert!(!verify_result);
    }
}