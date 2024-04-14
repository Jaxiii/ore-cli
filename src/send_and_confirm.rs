use std::{
    io::{stdout, Write},
    time::Duration,
};

use solana_client::{
    client_error::{ClientError, ClientErrorKind, Result as ClientResult},
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig},
};
use solana_program::instruction::Instruction;
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    compute_budget::ComputeBudgetInstruction,
    signature::{Signature, Signer},
    transaction::Transaction,
};
use solana_transaction_status::{TransactionConfirmationStatus, UiTransactionEncoding};

use crate::{/*utils::read_current_value*/Miner};

const SIMULATION_RETRIES: usize = 1;
const GATEWAY_RETRIES: usize = 2;
const CONFIRM_RETRIES: usize = 5;

const CONFIRM_DELAY: u64 = 10000;
const GATEWAY_DELAY: u64 = 10000;

impl Miner {
    pub async fn send_and_confirm(
        &self,
        ixs: &[Instruction],
        dynamic_cus: bool,
        skip_confirm: bool,
        hash_proof: Option<String>,
    ) -> ClientResult<Signature> {
        let mut stdout = stdout();
        let signer = self.signer();
        let jito_client = RpcClient::new_with_commitment(self.jito_client.to_owned(), CommitmentConfig::processed());
        let client = RpcClient::new_with_commitment(self.cluster.to_owned(), CommitmentConfig::processed());

        // Build tx - TODO move to its own thread
        let (mut hash, mut slot) = client
            .get_latest_blockhash_with_commitment(CommitmentConfig::finalized())
            .await
            .unwrap();
        let mut send_cfg = RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(CommitmentLevel::Confirmed),
            encoding: Some(UiTransactionEncoding::Base64),
            max_retries: None,
            min_context_slot: Some(slot),
        };
        let mut tx = Transaction::new_with_payer(ixs, Some(&signer.pubkey()));

        // Simulate if necessary
        if dynamic_cus {
            let mut sim_attempts = 0;
            'simulate: loop {
                // let current_value = read_current_value().await.expect("Failed to read current value");
                // if hash_proof != Some(current_value.clone()) {
                //     println!("Proof data does not match the current value. Another instance may have already submitted the transaction.");
                //     // Decide on your action here. You might want to break the loop or return early.
                //     return Err(ClientError {
                //         request: None,
                //         kind: ClientErrorKind::Custom("Proof data changed, stopping operation.".into()),
                //     });
                // }
                let sim_res = client
                    .simulate_transaction_with_config(
                        &tx,
                        RpcSimulateTransactionConfig {
                            sig_verify: false,
                            replace_recent_blockhash: true,
                            commitment: Some(CommitmentConfig::confirmed()),
                            encoding: Some(UiTransactionEncoding::Base64),
                            accounts: None,
                            min_context_slot: None,
                            inner_instructions: false,
                        },
                    )
                    .await;
                match sim_res {
                    Ok(sim_res) => {
                        if let Some(err) = sim_res.value.err {
                            println!("Simulaton error: {:?}", err);
                            sim_attempts += 1;
                            if sim_attempts.gt(&SIMULATION_RETRIES) {
                                return Err(ClientError {
                                    request: None,
                                    kind: ClientErrorKind::Custom("Simulation failed".into()),
                                });
                            }
                        } else if let Some(units_consumed) = sim_res.value.units_consumed {
                            println!("Dynamic CUs: {:?}", units_consumed);
                            let cu_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(
                                units_consumed as u32 + 1000,
                            );
                            let cu_price_ix =
                                ComputeBudgetInstruction::set_compute_unit_price(self.priority_fee);
                            let mut final_ixs = vec![];
                            final_ixs.extend_from_slice(&[cu_budget_ix, cu_price_ix]);
                            final_ixs.extend_from_slice(ixs);
                            tx = Transaction::new_with_payer(&final_ixs, Some(&signer.pubkey()));
                            break 'simulate;
                        }
                    }
                    Err(err) => {
                        println!("Simulaton error: {:?}", err);
                        sim_attempts += 1;
                        if sim_attempts.gt(&SIMULATION_RETRIES) {
                            return Err(ClientError {
                                request: None,
                                kind: ClientErrorKind::Custom("Simulation failed".into()),
                            });
                        }
                    }
                }
            }
        }

        // Submit tx
        tx.sign(&[&signer], hash);

        let mut attempts = 0;
        // let mut current_value = read_current_value().await.expect("Failed to read current value");
        // if hash_proof != Some(current_value.clone()) {
        //     println!("Proof data does not match the current value. Another instance may have already submitted the transaction.");
        //     // Decide on your action here. You might want to break the loop or return early.
        //     return Err(ClientError {
        //         request: None,
        //         kind: ClientErrorKind::Custom("Proof data changed, stopping operation.".into()),
        //     });
        // }
        loop {
            if self.jito_enable {
                // current_value = read_current_value().await.expect("Failed to read current value");
                // if hash_proof != Some(current_value.clone()) {
                //     println!("Proof data does not match the current value. Another instance may have already submitted the transaction.");
                //     // Decide on your action here. You might want to break the loop or return early.
                //     return Err(ClientError {
                //         request: None,
                //         kind: ClientErrorKind::Custom("Proof data changed, stopping operation.".into()),
                //     });
                // }
                match jito_client.send_transaction_with_config(&tx, send_cfg).await {
                    Err(_err) => {
                        // println!("JITO Error: {:?}", err);
                        continue
                    }
                    _ => {}
                };
            }
            // current_value = read_current_value().await.expect("Failed to read current value");
            // if hash_proof != Some(current_value.clone()) {
            //     println!("Proof data does not match the current value. Another instance may have already submitted the transaction.");
            //     // Decide on your action here. You might want to break the loop or return early.
            //     return Err(ClientError {
            //         request: None,
            //         kind: ClientErrorKind::Custom("Proof data changed, stopping operation.".into()),
            //     });
            // }
            match client.send_transaction_with_config(&tx, send_cfg).await {
                Ok(sig) => {
                    println!("{:?}", sig);

                    if skip_confirm {
                        return Ok(sig);
                    }
                    for _ in 0..CONFIRM_RETRIES {
                        // Read the current value from the file
                        // current_value = read_current_value().await.expect("Failed to read current value");
                        // if hash_proof != Some(current_value.clone()) {
                        //     println!("Proof data does not match the current value. Another instance may have already submitted the transaction.");
                        //     // Decide on your action here. You might want to break the loop or return early.
                        //     return Err(ClientError {
                        //         request: None,
                        //         kind: ClientErrorKind::Custom("Proof data changed, stopping operation.".into()),
                        //     });
                        // }
                        std::thread::sleep(Duration::from_millis(CONFIRM_DELAY));
                        match client.get_signature_statuses(&[sig]).await {
                            Ok(signature_statuses) => {
                                println!("Confirmation: {:?}", signature_statuses.value[0]);
                                            // Check if the existing proof data matches the current value
                                // current_value = read_current_value().await.expect("Failed to read current value");
                                // if hash_proof != Some(current_value.clone()) {
                                //     println!("Proof data does not match the current value. Another instance may have already submitted the transaction.");
                                //     // Decide on your action here. You might want to break the loop or return early.
                                //     return Err(ClientError {
                                //         request: None,
                                //         kind: ClientErrorKind::Custom("Proof data changed, stopping operation.".into()),
                                //     });
                                // }
                                for signature_status in signature_statuses.value {
                                    if let Some(signature_status) = signature_status.as_ref() {
                                        if signature_status.confirmation_status.is_some() {
                                            let current_commitment = signature_status
                                                .confirmation_status
                                                .as_ref()
                                                .unwrap();
                                            match current_commitment {
                                                TransactionConfirmationStatus::Processed => {}
                                                TransactionConfirmationStatus::Confirmed
                                                | TransactionConfirmationStatus::Finalized => {
                                                    println!("Transaction landed!");
                                                    std::thread::sleep(Duration::from_millis(
                                                        GATEWAY_DELAY,
                                                    ));
                                                    return Ok(sig);
                                                }
                                            }
                                        } else {
                                            println!("No status");
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                println!("Confirmation - Error: {:?}", err);
                            }
                        }
                    }
                }
                // Handle submit errors
                Err(err) => {
                    match err.kind {
                        ClientErrorKind::RpcError(rpc_error) => {
                            // Check if the RPC error matches the specific simulation failure we're interested in
                            if rpc_error.to_string().contains("custom program error: 0x3") {
                                // Detected the specific simulation failure error
                                println!("Detected specific simulation failure: {:?}", rpc_error);
                                // Act upon the detected error, e.g., log, retry with adjustments, or break
                                return Err(ClientError {
                                    request: None,
                                    kind: ClientErrorKind::Custom("Detected specific simulation failure".into()),
                                });
                            }
                        },
                        _ => {
                            // Handle other types of errors
                            println!("Submit error: {:?}", err);
                           
                        }
                    }
                }
            }
            stdout.flush().ok();

            // Retry
            std::thread::sleep(Duration::from_millis(200));
            (hash, slot) = client
                .get_latest_blockhash_with_commitment(CommitmentConfig::finalized())
                .await
                .unwrap();
            send_cfg = RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: Some(CommitmentLevel::Processed),
                encoding: Some(UiTransactionEncoding::Base64),
                max_retries: None,
                min_context_slot: Some(slot),
            };
            tx.sign(&[&signer], hash);
            attempts += 1;
            if attempts > GATEWAY_RETRIES {
                break Err(ClientError {
                    request: None,
                    kind: ClientErrorKind::Custom("Max retries".into()),
                });
            }
        }
    }
}
