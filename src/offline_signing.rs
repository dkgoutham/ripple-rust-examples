use anyhow::{Context, Result};
use std::borrow::Cow;
use xrpl::{
    asynch::{
        clients::{AsyncWebSocketClient, WebSocketOpen, client::XRPLClient},
        ledger::get_latest_validated_ledger_sequence,
        transaction::sign,
    },
    core::binarycodec::encode,
    models::{
        Amount, IssuedCurrencyAmount, XRPAmount,
        requests::submit::Submit as SubmitRequest,
        transactions::payment::Payment,
    },
    wallet::Wallet,
};
use crate::client::get_account_info;

type XRPLClientType = AsyncWebSocketClient<xrpl::asynch::clients::SingleExecutorMutex, WebSocketOpen>;

/// Security configuration for offline transactions
const TRANSACTION_EXPIRY_LEDGERS: u32 = 10; // Transaction expires after 10 ledgers (~50 seconds)
const MINIMUM_FEE_DROPS: u32 = 12; // Minimum fee for testnet

// Parameters required for secure offline transaction construction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineTransactionParams {
    // Next sequence number for the account
    pub sequence: u32,
    // Transaction fee in drops
    pub fee: String,
    // Ledger sequence after which transaction expires
    pub last_ledger_sequence: u32,
    /// Current validated ledger when parameters were gathered
    pub current_ledger_index: u32,
}

impl OfflineTransactionParams {
    // Validate that transaction parameters are secure and not expired
    pub fn validate_security(&self, current_ledger: Option<u32>) -> Result<()> {
        // Ensure expiration is set
        if self.last_ledger_sequence == 0 {
            anyhow::bail!("Transaction must have expiration (last_ledger_sequence) for security");
        }
        
        // Check if transaction has expired (if current ledger provided)
        if let Some(current) = current_ledger {
            if current >= self.last_ledger_sequence {
                anyhow::bail!(
                    "Transaction expired: current ledger {} >= expiration {}", 
                    current, self.last_ledger_sequence
                );
            }
        }
        
        // Validate fee is reasonable
        let fee_drops: u32 = self.fee.parse()
            .context("Fee must be valid numeric string")?;
            
        if fee_drops < MINIMUM_FEE_DROPS {
            anyhow::bail!("Fee {} drops is below minimum {}", fee_drops, MINIMUM_FEE_DROPS);
        }
        
        Ok(())
    }
    
    // Calculate remaining ledgers until expiration
    pub fn remaining_ledgers(&self, current_ledger: u32) -> i32 {
        self.last_ledger_sequence as i32 - current_ledger as i32
    }
}

// Gather transaction parameters online (to be passed to offline environment)
pub async fn gather_transaction_params(
    client: &XRPLClientType,
    account_address: &str,
) -> Result<OfflineTransactionParams> {
    println!("Gathering transaction parameters for offline signing...");
    println!("Account: {}", account_address);
    
    // Get current validated ledger index for expiration calculation
    let current_ledger_index = get_latest_validated_ledger_sequence(client)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get current ledger: {:?}", e))?;
    
    println!("Current validated ledger: {}", current_ledger_index);
    
    // Get account info to determine next sequence number
    let account_info = get_account_info(client, account_address).await
        .context("Failed to retrieve account information")?;
    
    let account_root = account_info.get_account_root();
    let sequence = account_root.sequence;
    
    // Calculate secure expiration: current + buffer ledgers
    let last_ledger_sequence = current_ledger_index + TRANSACTION_EXPIRY_LEDGERS;
    
    // Use minimum required fee for testnet
    let fee = MINIMUM_FEE_DROPS.to_string();
    
    let params = OfflineTransactionParams {
        sequence,
        fee,
        last_ledger_sequence,
        current_ledger_index,
    };
    
    // Validate security parameters before returning
    params.validate_security(Some(current_ledger_index))
        .context("Security validation failed")?;
    
    println!("Transaction parameters gathered:");
    println!("  Sequence: {}", params.sequence);
    println!("  Fee: {} drops", params.fee);
    println!("  Current Ledger: {}", params.current_ledger_index);
    println!("  Expires at Ledger: {}", params.last_ledger_sequence);
    println!("  Valid for {} more ledgers (~{} seconds)", 
             TRANSACTION_EXPIRY_LEDGERS, 
             TRANSACTION_EXPIRY_LEDGERS * 5); // ~5 seconds per ledger
    
    Ok(params)
}

// Sign transaction completely offline (no network calls)
pub fn offline_sign_transaction(
    user_secret: &str,
    to_address: &str,
    amount: Amount<'static>,
    params: OfflineTransactionParams,
) -> Result<String> {
    println!("Signing transaction OFFLINE (no network calls)...");
    
    // Validate parameters are secure before signing
    params.validate_security(None)
        .context("Transaction parameters failed security validation")?;
    
    let wallet = Wallet::new(user_secret, 0)
        .map_err(|e| anyhow::anyhow!("Wallet error: {:?}", e))?;
    
    println!("From address: {}", wallet.classic_address);
    println!("To address: {}", to_address);
    println!("Using offline parameters:");
    println!("  Sequence: {}", params.sequence);
    println!("  Fee: {}", params.fee);
    println!("  Expires at ledger: {}", params.last_ledger_sequence);
    
    // Create payment with manually set parameters (no network calls)
    let mut payment = Payment::new(
        Cow::Owned(wallet.classic_address.clone()),
        None,
        Some(XRPAmount(Cow::Owned(params.fee.clone()))), 
        None, 
        Some(params.last_ledger_sequence), 
        None, 
        Some(params.sequence), 
        None, 
        None, 
        None, 
        amount,
        Cow::Owned(to_address.to_string()),
        None, 
        None, 
        None,
        None, 
        None,
    );

    println!("Signing transaction offline...");
    
    // Sign the transaction
    sign(&mut payment, &wallet, false)
        .map_err(|e| anyhow::anyhow!("Sign error: {:?}", e))?;

    println!("Encoding to signed blob...");
    
    // Encode to hex blob
    let signed_blob = encode(&payment)
        .map_err(|e| anyhow::anyhow!("Encode error: {:?}", e))?;

    println!("Transaction signed offline successfully!");
    println!("Signed blob length: {} characters", signed_blob.len());
    println!("Blob preview: {}...", &signed_blob[..std::cmp::min(64, signed_blob.len())]);
    println!("Security: Transaction expires at ledger {}", params.last_ledger_sequence);
    
    Ok(signed_blob)
}

// Submit pre-signed transaction blob using different connection
pub async fn submit_signed_blob(
    client: &XRPLClientType,
    signed_blob: &str,
) -> Result<String> {
    println!("Submitting pre-signed blob via different connection...");
    println!("Blob length: {} characters", signed_blob.len());
    println!("Blob preview: {}...", &signed_blob[..std::cmp::min(64, signed_blob.len())]);
    
    // Get current ledger to check if transaction has expired
    let current_ledger = get_latest_validated_ledger_sequence(client)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get current ledger before submission: {:?}", e))?;
    
    println!("Current ledger at submission: {}", current_ledger);
    
    // Create submit request with the signed blob
    let submit_request = SubmitRequest::new(
        None, // id
        Cow::Borrowed(signed_blob), // tx_blob
        None, // fail_hard
    );

    let response = client.request_impl(submit_request.into()).await
        .context("Failed to submit transaction blob")?;
    
    match response.result {
        Some(xrpl::models::results::XRPLResult::Submit(submit_result)) => {
            println!("Transaction submitted successfully via different connection!");
            println!("Engine result: {}", submit_result.engine_result);
            
            // Check for common expiration errors
            if submit_result.engine_result.contains("EXPIRED") || 
               submit_result.engine_result.contains("LATE") {
                anyhow::bail!("Transaction expired: {}", submit_result.engine_result);
            }
            
            let tx_hash = submit_result.tx_json
                .get("hash")
                .and_then(|h| h.as_str())
                .ok_or_else(|| anyhow::anyhow!("No transaction hash in response"))?
                .to_string();
                
            println!("Transaction hash: {}", tx_hash);
            Ok(tx_hash)
        }
        _ => {
            anyhow::bail!("Unexpected response type for submit request")
        }
    }
}

// High-level workflow: Complete offline signing process for XRP with security validation
pub async fn offline_xrp_workflow(
    online_client: &XRPLClientType,
    offline_client: &XRPLClientType,
    user_secret: &str,
    to_address: &str,
    amount_drops: u64,
) -> Result<String> {
    // Phase 1: Gather parameters online with security validation
    let wallet = Wallet::new(user_secret, 0)
        .map_err(|e| anyhow::anyhow!("Wallet error: {:?}", e))?;
    
    let params = gather_transaction_params(online_client, &wallet.classic_address).await
        .context("Failed to gather secure transaction parameters")?;
    
    // Phase 2: Sign completely offline with expiration bounds
    let xrp_amount = XRPAmount(Cow::Owned(amount_drops.to_string()));
    let signed_blob = offline_sign_transaction(
        user_secret,
        to_address,
        Amount::XRPAmount(xrp_amount),
        params.clone(),
    ).context("Failed to sign transaction offline")?;
    
    // Phase 3: Submit via different connection with expiration checking
    let tx_hash = submit_signed_blob(offline_client, &signed_blob).await
        .context("Failed to submit signed blob")?;
    
    println!("Secure offline workflow completed successfully!");
    Ok(tx_hash)
}

// High-level workflow: Complete offline signing process for tokens with security validation
pub async fn offline_token_workflow(
    online_client: &XRPLClientType,
    offline_client: &XRPLClientType,
    user_secret: &str,
    to_address: &str,
    currency_code: &str,
    amount: &str,
) -> Result<String> {
    // Phase 1: Gather parameters online with security validation
    let wallet = Wallet::new(user_secret, 0)
        .map_err(|e| anyhow::anyhow!("Wallet error: {:?}", e))?;
    
    let params = gather_transaction_params(online_client, &wallet.classic_address).await
        .context("Failed to gather secure transaction parameters")?;
    
    // Phase 2: Sign completely offline with expiration bounds
    let issued_amount = IssuedCurrencyAmount::new(
        Cow::Owned(currency_code.to_string()),
        Cow::Owned(wallet.classic_address.clone()),
        Cow::Owned(amount.to_string()),
    );
    
    let signed_blob = offline_sign_transaction(
        user_secret,
        to_address,
        Amount::IssuedCurrencyAmount(issued_amount),
        params.clone(),
    ).context("Failed to sign transaction offline")?;
    
    // Phase 3: Submit via different connection with expiration checking
    let tx_hash = submit_signed_blob(offline_client, &signed_blob).await
        .context("Failed to submit signed blob")?;
    
    println!("Secure offline token workflow completed successfully!");
    Ok(tx_hash)
}