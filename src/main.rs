use anyhow::Result;
use dotenvy::dotenv;
use ripple_task::{XRPLManager, wallet_from_seed};
use std::env;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting XRPL Rust Library Demo");
    println!("=====================================");

    // Load environment variables
    if let Err(_) = dotenvy::dotenv() {
        eprintln!("Error: .env file not found in current directory");
        eprintln!("Please create a .env file with the following format:");
        eprintln!("USER1_SEED=your_first_testnet_seed_here");
        eprintln!("USER2_SEED=your_second_testnet_seed_here");
        std::process::exit(1);
    }

    let user1_seed = std::env::var("USER1_SEED")
        .map_err(|_| anyhow::anyhow!("USER1_SEED not found in .env file"))?;
    let user2_seed = std::env::var("USER2_SEED")
        .map_err(|_| anyhow::anyhow!("USER2_SEED not found in .env file"))?;

    // Create XRPL manager
    let xrpl = XRPLManager::new_testnet().await?;

    println!("=====================================");

    // Demo 1: XRP Transfer
    println!("\n1: XRP Transfer");
    println!(".....................");

    demo_xrp_transfer(&xrpl, &user1_seed, &user2_seed).await?;

    println!("\nWaiting 10 seconds for transaction to settle...");
    sleep(Duration::from_secs(10)).await;

    println!("=====================================");

    // Demo 2: Issued Token Transfer
    println!("\n2: Issued Token Transfer");
    println!(".......................");

    demo_issued_token_transfer(&xrpl, &user1_seed, &user2_seed).await?;

    println!("\nWaiting 10 seconds for transaction to settle...");
    sleep(Duration::from_secs(10)).await;

    println!("=====================================");

    // Demo 3: Offline Signing
    println!("\n3: Offline Signing & Submission");
    println!("......................................");

    demo_offline_signing(&xrpl, &user1_seed, &user2_seed).await?;

    println!("\nAll demos completed successfully!");
    Ok(())
}

async fn demo_xrp_transfer(xrpl: &XRPLManager, user1_seed: &str, user2_seed: &str) -> Result<()> {
    let user1_wallet = wallet_from_seed(user1_seed)?;
    let user2_wallet = wallet_from_seed(user2_seed)?;

    println!("User1 (Sender): {}", user1_wallet.classic_address);
    println!("User2 (Receiver): {}", user2_wallet.classic_address);

    let amount_drops = 100;
    println!("\nSending {} drops from User1 to User2...", amount_drops);

    match xrpl
        .send_xrp(user1_seed, &user2_wallet.classic_address, amount_drops)
        .await
    {
        Ok(tx_hash) => {
            println!("XRP transfer successful!");

            println!("Waiting 5 seconds before verification...");
            sleep(Duration::from_secs(5)).await;

            println!("\nVerifying XRP transfer...");
            match xrpl
                .verify_transfer(
                    &tx_hash,
                    &user1_wallet.classic_address,
                    &user2_wallet.classic_address,
                    &amount_drops.to_string(),
                    None,
                )
                .await
            {
                Ok(true) => println!("XRP transfer verification successful!"),
                Ok(false) => println!("XRP transfer verification failed!"),
                Err(e) => println!("Error during verification: {}", e),
            }
        }
        Err(e) => println!("XRP transfer failed: {}", e),
    }

    Ok(())
}

async fn demo_issued_token_transfer(
    xrpl: &XRPLManager,
    user1_seed: &str,
    user2_seed: &str,
) -> Result<()> {
    let user1_wallet = wallet_from_seed(user1_seed)?;
    let user2_wallet = wallet_from_seed(user2_seed)?;

    println!("User1 (Token Issuer): {}", user1_wallet.classic_address);
    println!("User2 (Token Receiver): {}", user2_wallet.classic_address);

    let currency_code = "TST";
    let trust_limit = "1000";

    println!(
        "\nUser2 setting up trustline for {} tokens...",
        currency_code
    );
    match xrpl
        .setup_trustline(
            user2_seed,
            &user1_wallet.classic_address,
            currency_code,
            trust_limit,
        )
        .await
    {
        Ok(trustline_tx_hash) => {
            println!("Trustline setup successful! Hash: {}", trustline_tx_hash);

            println!("Waiting 10 seconds for trustline to be processed...");
            sleep(Duration::from_secs(10)).await;

            let token_amount = "100";
            println!(
                "\nUser1 issuing {} {} tokens to User2...",
                token_amount, currency_code
            );

            match xrpl
                .send_issued_token(
                    user1_seed,
                    &user2_wallet.classic_address,
                    currency_code,
                    token_amount,
                )
                .await
            {
                Ok(token_tx_hash) => {
                    println!("Token issuance successful! Hash: {}", token_tx_hash);

                    println!("Waiting 5 seconds before verification...");
                    sleep(Duration::from_secs(5)).await;

                    println!("\nVerifying token transfer...");
                    match xrpl
                        .verify_transfer(
                            &token_tx_hash,
                            &user1_wallet.classic_address,
                            &user2_wallet.classic_address,
                            token_amount,
                            Some(currency_code),
                        )
                        .await
                    {
                        Ok(true) => println!("Token transfer verification successful!"),
                        Ok(false) => println!("Token transfer verification failed!"),
                        Err(e) => println!("Error during verification: {}", e),
                    }
                }
                Err(e) => println!("Token issuance failed: {}", e),
            }
        }
        Err(e) => println!("Trustline setup failed: {}", e),
    }

    Ok(())
}

async fn demo_offline_signing(xrpl: &XRPLManager, user1_seed: &str, user2_seed: &str) -> Result<()> {
    let user1_wallet = wallet_from_seed(user1_seed)?;
    let user2_wallet = wallet_from_seed(user2_seed)?;

    println!("User1 (Sender): {}", user1_wallet.classic_address);
    println!("User2 (Receiver): {}", user2_wallet.classic_address);

    let amount_drops = 75;
    
    println!("\n=== TRUE OFFLINE SIGNING WORKFLOW ===");
    
    println!("\n1: Gather transaction parameters (Connection A - Online)");
    println!("--------------------------------------------------------------");
    let params = match xrpl.gather_transaction_params(&user1_wallet.classic_address).await {
        Ok(p) => {
            println!("Transaction parameters gathered successfully from Connection A");
            p
        }
        Err(e) => {
            println!("Failed to gather parameters: {}", e);
            return Err(e);
        }
    };

    println!("\n2: Sign transaction OFFLINE (No network connection)");  
    println!("---------------------------------------------------------");
    println!("Simulating air-gapped environment...");
    
    let signed_blob = match XRPLManager::offline_sign_transaction(
        user1_seed,
        &user2_wallet.classic_address,
        xrpl::models::Amount::XRPAmount(xrpl::models::XRPAmount(std::borrow::Cow::Owned(amount_drops.to_string()))),
        params,
    ) {
        Ok(blob) => {
            println!("Transaction signed successfully in OFFLINE environment!");
            println!("Key point: No network calls were made during signing phase");
            blob
        }
        Err(e) => {
            println!("Offline signing failed: {}", e);
            return Err(e);
        }
    };

    println!("\n3: Submit signed blob (Connection B - Different connection)");
    println!("----------------------------------------------------------------");
    let xrpl2 = XRPLManager::create_second_connection().await?;
    println!("Created separate Connection B for submission");

    match xrpl2.submit_signed_blob(&signed_blob).await {
        Ok(tx_hash) => {
            println!("Signed blob submitted successfully via Connection B!");
            
            println!("Waiting 5 seconds before verification...");
            sleep(Duration::from_secs(5)).await;

            println!("\n4: Verify transaction on ledger");
            println!("------------------------------------");
            match xrpl2
                .verify_transfer(
                    &tx_hash,
                    &user1_wallet.classic_address,
                    &user2_wallet.classic_address,
                    &amount_drops.to_string(),
                    None,
                )
                .await
            {
                Ok(true) => println!("Offline signed transaction verified successfully!"),
                Ok(false) => println!("Offline signed transaction verification failed!"),
                Err(e) => println!("Error during verification: {}", e),
            }
        }
        Err(e) => {
            println!("Blob submission failed: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}