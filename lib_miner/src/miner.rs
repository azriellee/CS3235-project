// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file implements the Miner struct and related methods. 
// The miner has one key task: to solve a given puzzle (a string) with specified number of threads and difficulty levels.
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./lib.rs to understand the expected behavior of the miner.

use std::{thread, convert};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, RwLock};
use rand_pcg::Pcg32;
use rand::{Rng, SeedableRng, distributions::{Alphanumeric, DistString}};
use sha2::{Sha256, Digest};


// A miner that solve puzzles.
pub struct Miner {
    /// number of threads used to solve the puzzle in parallel
    thread_count: u16,

    /// number of leading "0"s expected in the resulting hash string in hex format.
    /// e.g. if leading_zero_len is 3, then the hash string should start with "000"
    /// and the difficulty level is 3.
    leading_zero_len: u16,

    /// whether the miner is running or not
    is_running: bool
}

type BlockId = String;

/// The struct to represent a puzzle solution returned by the miner.
pub struct PuzzleSolution {
    /// the puzzle string
    pub puzzle: String,
    /// the nonce string that should be prepended to the puzzle string for computing the hash
    pub nonce: String,
    /// the sha256 hash of (nonce || puzzle) in hex format
    pub hash: BlockId
}

impl Miner {
    // constructor
    pub fn new () -> Miner {
        Miner { 
            thread_count: 0,
            leading_zero_len: 0,
            is_running: false
        }
    }
    
    pub fn generate_hash(p: String, nonce_len: &u16, thread_seed: u64) -> (String, String) {
        let mut rng = Pcg32::seed_from_u64(thread_seed);
        let nonce_string = Alphanumeric.sample_string(&mut rng, *nonce_len as usize);
        let combined_string = format!("{}{}",nonce_string, p);
        let mut hasher = Sha256::new();
        hasher.update(combined_string.as_bytes());
        let result = hasher.finalize();
        let hex_string = result.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        (nonce_string, hex_string)
    }

    /// The method to solve a puzzle with specified number of threads and difficulty levels.
    /// This method is a function on the class (without `self` as the 1st argument). The first parameter is a smart pointer to a miner instance.
    /// - `miner_p`: the smart pointer to the miner instance
    /// - `puzzle`: the puzzle string
    /// - `nonce_len`: the length of the nonce string in the solution. The nonce string should be randomly generated from the alphanumeric characters A-Z, a-z and 0-9.
    /// - `leading_zero_len`: the number of leading "0"s expected in the resulting hash string in hex format.
    /// - `thread_count`: the number of threads to be used for solving the puzzle in parallel.
    /// - `thread_0_seed`: the seed for the random number generator for the first thread. The seed for the second thread should be `thread_0_seed + 1`, and so on.
    /// - `cancellation_token`: a smart pointer to a boolean value. If the value is set to true, all threads should stop even if they have not found a solution.
    /// - return: an optional value with the solution if the puzzle is solved, or None if the puzzle is cancelled.
    pub fn solve_puzzle(miner_p: Arc<Mutex<Miner>>, puzzle: String, nonce_len: u16, leading_zero_len: u16, thread_count: u16, thread_0_seed: u64, cancellation_token: Arc<RwLock<bool>>) -> Option<PuzzleSolution> {
        
        // Please fill in the blank
        // In this function, you are expected to start multiple threads for solving the puzzle.
        // The threads should be spawned and joined in this function.
        // If any of the threads finds a solution, other threads should stop.
        // Additionally, if the cancellation_token is set to true, all threads should stop.
        // The purpose of the cancellation_token is to allow the miner to stop the computation when other nodes have already solved the exact same puzzle.
        let mut miner = miner_p.lock().unwrap();
        miner.is_running = true;
        miner.leading_zero_len = leading_zero_len;
        miner.thread_count = thread_count;
        drop(miner);

        let mut threads = Vec::new();
        for i in 0..thread_count {
            let cancel = cancellation_token.clone();
            let p = puzzle.clone();
            let miner_clone = miner_p.clone();
            let thread_handle = thread::spawn(move || {
                let mut seed = thread_0_seed + i as u64;
                while !*cancel.read().unwrap() {
                    let nonce_and_hash = Miner::generate_hash(p.clone(), &nonce_len, seed);
                    if nonce_and_hash.1.starts_with(&"0".repeat(leading_zero_len as usize)) {
                        let solution = PuzzleSolution {
                            puzzle: p.clone(),
                            nonce: nonce_and_hash.0,
                            hash: nonce_and_hash.1,
                        };
                        return Some(solution);
                    }
                    seed += thread_count as u64;
                }
                // another node found the solution
                miner_clone.lock().unwrap().is_running = false;
                None
            });
            threads.push(thread_handle);
        }
        for t in threads {
            if let Some(solution) = t.join().unwrap() {
                let mut miner = miner_p.lock().unwrap();
                miner.is_running = false;
                return Some(solution);
            }
        }
        None
    } 
    
    /// Get status information of the miner for debug printing.
    pub fn get_status(&self) -> BTreeMap<String, String> {
        // Please fill in the blank
        // For debugging purpose, you can return any dictionary of strings as the status of the miner. 
        // It should be displayed in the Client UI eventually.
        let mut status = BTreeMap::new();
        status.insert("is_running".to_string(), self.is_running.to_string());
        if self.is_running == true {
            status.insert("#thread".to_string(), self.thread_count.to_string());
            status.insert("difficulty".to_string(), self.leading_zero_len.to_string());
        } else {
            status.insert("#thread".to_string(), "0".to_string());
            status.insert("difficulty".to_string(), "0".to_string());
        }
        status
    }
}