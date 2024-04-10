use std::{fs::File, io::{Read, Write}, path::Path};

use cached::proc_macro::cached;
use ore::{
    self,
    state::{Proof, Treasury},
    utils::AccountDeserialize,
    MINT_ADDRESS, PROOF, TREASURY_ADDRESS,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey, sysvar};
use solana_sdk::{clock::Clock, commitment_config::CommitmentConfig};
use spl_associated_token_account::get_associated_token_address;

pub async fn get_treasury(cluster: String) -> Treasury {
    let client = RpcClient::new_with_commitment(cluster, CommitmentConfig::processed());
    let data = client
        .get_account_data(&TREASURY_ADDRESS)
        .await
        .expect("Failed to get treasury account");
    *Treasury::try_from_bytes(&data).expect("Failed to parse treasury account")
}

pub async fn get_proof(cluster: String, authority: Pubkey) -> Proof {
    print!("Getting proof...\n");
    let client = RpcClient::new_with_commitment(cluster, CommitmentConfig::processed());
    let proof_address = proof_pubkey(authority);

    let data = client
        .get_account_data(&proof_address)
        .await
        .expect("Failed to get miner account");

    let proof = Proof::try_from_bytes(&data).expect("Failed to parse miner account");

    // Attempt to read the existing proof data from the file
    match File::open("proof_data.txt") {
        Ok(mut file) => {
            let mut existing_proof_data = String::new();
            if let Ok(_) = file.read_to_string(&mut existing_proof_data) {
                // Compare the existing proof data with the current proof data
                if existing_proof_data == proof.hash.to_string() {
                    println!("Existing proof data matches the current proof data. Skipping write.");
                    return *proof;
                }
            }
        },
        Err(e) => println!("No existing proof data file found or error reading file: {:?}", e),
    }

    // Write the proof data to the file if it's different from the existing data or no file was found
    let mut file = File::create("proof_data.txt").expect("Failed to create file");
    file.write_all(proof.hash.to_string().to_string().as_bytes())
        .expect("Failed to write data to file");

    *Proof::try_from_bytes(&data).expect("Failed to parse miner account")
}

pub async fn get_clock_account(cluster: String) -> Clock {
    let client = RpcClient::new_with_commitment(cluster, CommitmentConfig::processed());
    let data = client
        .get_account_data(&sysvar::clock::ID)
        .await
        .expect("Failed to get miner account");
    bincode::deserialize::<Clock>(&data).expect("Failed to deserialize clock")
}

#[cached]
pub fn proof_pubkey(authority: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[PROOF, authority.as_ref()], &ore::ID).0
}

#[cached]
pub fn treasury_tokens_pubkey() -> Pubkey {
    get_associated_token_address(&TREASURY_ADDRESS, &MINT_ADDRESS)
}

pub async fn read_current_value() -> tokio::io::Result<String> {
    let file_path = "proof_data.txt";
    if Path::new(file_path).exists() {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents.trim().to_string()) // Trim whitespace and return
    } else {
        Ok(String::new()) // Return an empty string if the file does not exist
    }
}