// This file is part of the project for the module CS3235 by Prateek 
// Copyright 2023 Ruishi Li, Bo Wang, and Prateek Saxena.
// Please do not distribute.

// This file implements the Wallet struct and related methods.
// The wallet has one key task: to sign a message using the private key.
// The wallet also has a method to verify the signature for debugging purposes. Verification does not involve the private key.
// The actual verification of the signature should be implemented in the lib_chain module.
// You can see detailed instructions in the comments below.
// You can also look at the unit tests in ./main.rs to understand the expected behavior of the wallet.

use rsa::{RsaPublicKey, RsaPrivateKey};
use rsa::pkcs1::{EncodeRsaPublicKey, EncodeRsaPrivateKey, DecodeRsaPublicKey, DecodeRsaPrivateKey, LineEnding};
use rsa::pkcs1v15::{SigningKey, VerifyingKey};
use rsa::signature::{RandomizedSigner, Signature, Verifier};

use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use base64ct::{Base64, Encoding};

/// A wallet that stores the key pairs. Most importantly, the private key.
/// For the format of the key, you can check the unit test at ./main.rs:test_bin_wallet_signing_and_verifying
/// to see how the key is loaded and used.
#[derive(Serialize, Deserialize)]
pub struct Wallet {
    /// Friendly name of the user. Doesn't matter what it is.
    pub user_name: String,
    /// The private key in PEM format
    pub priv_key_pem: String,
    /// The public key in PEM format
    pub pub_key_pem: String
}

impl Wallet {
    /// Create a new wallet with a given user name and key size.
    /// It will generate a new pair of keys.
    /// During the evaluation, you don't need to generate new keys.
    pub fn new(user_name: String, bits: usize) -> Wallet {
        // Please fill in the blank
        //todo!(); 
        let mut rng = rand::thread_rng();

        // Generate a new key pair
        let private_key = RsaPrivateKey::new(&mut rng, bits).expect("What am I even doing");
        let public_key = RsaPublicKey::from(&private_key);

        // Encode the private key and public key to PEM format
        let pub_key_pem = EncodeRsaPublicKey::to_pkcs1_pem(&public_key, LineEnding::LF).unwrap().to_string();
        let priv_key_pem = EncodeRsaPrivateKey::to_pkcs1_pem(&private_key, LineEnding::LF).unwrap().to_string();
        
        /*Let this be the grave of failed code, a record of my fruitless screams into this god forsaken void
        let priv_key_pem = EncodeRsaPrivateKey::to_pkcs1_pem(&private_key).unwrap()
        let priv_key_pem = EncodeRsaPrivateKey::to_pkcs1_pem(&private_key, LineEnding::LF).unwrap()
        let priv_key_pem = "unable to compute, send faqing help".to_string();
        let priv_key_pem = private_key;
        let priv_key_pem = EncodeRsaPrivateKey::to_pkcs1_pem(&private_key, LineEnding::LF).unwrap().into_inner(); 
        let priv_key_pem2 = EncodeRsaPrivateKey::to_pkcs1_pem(&private_key, LineEnding::LF).unwrap().into_inner().to_string();
        let priv_key_pem = *priv_key_pem2.as_ref();
        let priv_key_pem = EncodeRsaPrivateKey::to_pkcs1_pem(&private_key, LineEnding::LF).unwrap().deref().to_owned();
        let priv_key_pem = private_key.private_key_to_pem();
        let rsa = rsa::generate(bits).unwrap();
        let priv_key_pem : Vec<u8> = rsa.private_key_to_pem().unwrap();
        let priv_key_pem = EncodeRsaPrivateKey::to_pkcs1_pem(&private_key, LineEnding::LF).unwrap()
        let priv_key_pem = "unable to compute, send faqing help".to_string();
        */
       
        // Create a new wallet with the given user name and encoded keys
        Wallet {
            user_name,
            priv_key_pem,
            pub_key_pem,
        }
    }

    /// return the   user name
    pub fn get_user_name(&self) -> String {
        return self.user_name.clone();
    }

    /// return the user id (transformed from the public key)
    pub fn get_user_id(&self) -> String {
        // Please fill in the blank
        // Get user id from the public key by changing the format (strip off the first and last lines and join the middle lines)
        // Pub key format:  "-----BEGIN RSA PUBLIC KEY-----\nMDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JG\npfiZSckCAwEAAQ==\n-----END RSA PUBLIC KEY-----\n"
        // user_id format:  "MDgCMQCqrJ1yIJ7cDQIdTuS+4CkKn/tQPN7bZFbbGCBhvjQxs71f6Vu+sD9eh8JGpfiZSckCAwEAAQ=="
        
        //let y : Result<RsaPublicKey, dyn Error> = DecodeRsaPublicKey::from_pkcs1_pem(&self.pub_key_pem).unwrap();
        //let pub_key : Result<RsaPublicKey> = DecodeRsaPublicKey::from_pkcs1_pem(&self.pub_key_pem);
        //let public_key = RsaPublicKey::from_pkcs1_pem(&self.pub_key_pem).unwrap();
        //return RsaPublicKey::from_bytes(&public_key).unwrap();
        
        let start = "-----BEGIN RSA PUBLIC KEY-----";
        let end = "-----END RSA PUBLIC KEY-----";

        let arr_temp : Vec<&str>  = self.pub_key_pem.split("\n").collect();
        let mut final_str = String::new();

        for message in arr_temp {
            if message != start && message != end {
                final_str.push_str(message);
            }
        }
        //println!("Test:\n", final_str);
        return final_str.to_string()
    }

    /// Sign a message using the private key and return the signature as a Base64 encoded string.
    /// To check if your implementation is correct, you can validate it using the `verify` method below in the unit tests.
    pub fn sign(&self, message: &str) -> String {
        // Please fill in the blank
        
        // Sign the message with the private key, and return the signature in Base64 format
        let private_key = RsaPrivateKey::from_pkcs1_pem(&self.priv_key_pem).unwrap();
        let signing_key = SigningKey::<Sha256>::new(private_key);

        // Hash the message using SHA-256
        //let message_digest = Digest(message);
        //let message_digest = SigningKey::<Sha256>::digest(&message);
        //let mut rng = rand::thread_rng();
        //let signature = signing_key.sign_with_rng(&mut rng, message.as_bytes());
        //let signature64 = Signature::from_bytes((&signature).unwrap());
        let signature = rsa::signature::Signer::sign(&signing_key, message.as_bytes());
        let signature64 = Base64::encode_string(&signature);
        /*
        if self.verify(message, &signature64) {
            println!("Good job!");
        } else {
            println!("Toh.jpg");
        }
         */

        return signature64;
    }

    /// Verify a signature using the public key. The signature is a string in Base64 format.
    pub fn verify(&self, message: &str, signature64: &str) -> bool {
        let public_key = rsa::RsaPublicKey::from_pkcs1_pem(&self.pub_key_pem).unwrap();
        let verifying_key = VerifyingKey::<Sha256>::new(public_key);

        let signature = Base64::decode_vec(&signature64).unwrap();
        let verify_signature = Signature::from_bytes(&signature).unwrap();
        let verify_result = verifying_key.verify(message.as_bytes(), &verify_signature);
        return match verify_result {
            Ok(()) => true,
            Err(e) => {
                //println!("[Signature verification failed]: {}", e);
                false
            }
        }
    }
}

