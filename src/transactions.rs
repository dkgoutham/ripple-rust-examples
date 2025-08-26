use anyhow::Result;
use std::borrow::Cow;
use xrpl::{
    asynch::{
        clients::{AsyncWebSocketClient, WebSocketOpen},
        transaction::sign_and_submit,
    },
    models::{
        Amount, IssuedCurrencyAmount, XRPAmount,
        transactions::{payment::Payment, trust_set::TrustSet},
    },
    wallet::Wallet,
};

type XRPLClientType =
    AsyncWebSocketClient<xrpl::asynch::clients::SingleExecutorMutex, WebSocketOpen>;

/// Send XRP from one account to another
pub async fn send_xrp(
    client: &XRPLClientType,
    user1_secret: &str,
    user2_address: &str,
    amount_drops: u64,
) -> Result<String> {
    println!("Preparing XRP transfer...");
    println!("\nFrom seed: {}...", &user1_secret[..8]);
    println!("\nTo address: {}", user2_address);
    println!("\nAmount: {} drops", amount_drops);

    let wallet =
        Wallet::new(user1_secret, 0).map_err(|e| anyhow::anyhow!("Wallet error: {:?}", e))?;
    println!("\nFrom address: {}", wallet.classic_address);

    let xrp_amount = XRPAmount(Cow::Owned(amount_drops.to_string()));

    let mut payment = Payment::new(
        Cow::Owned(wallet.classic_address.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Amount::XRPAmount(xrp_amount),
        Cow::Owned(user2_address.to_string()),
        None,
        None,
        None,
        None,
        None,
    );

    println!("Submitting XRP transaction...");

    let result = sign_and_submit(&mut payment, client, &wallet, true, false)
        .await
        .map_err(|e| anyhow::anyhow!("Transaction error: {:?}", e))?;

    let tx_hash = result
        .tx_json
        .get("hash")
        .and_then(|h| h.as_str())
        .unwrap_or("")
        .to_string();

    println!("XRP transaction submitted successfully!");
    println!("\nTransaction hash: {}", tx_hash);
    println!("\nEngine result: {}", result.engine_result);
    Ok(tx_hash)
}

pub async fn setup_trustline(
    client: &XRPLClientType,
    user_secret: &str,
    issuer_address: &str,
    currency_code: &str,
    limit: &str,
) -> Result<String> {
    println!("Setting up trustline...");
    println!("\nUser seed: {}...", &user_secret[..8]);
    println!("\nIssuer: {}", issuer_address);
    println!("\nCurrency: {}", currency_code);
    println!("\nLimit: {}", limit);

    let wallet =
        Wallet::new(user_secret, 0).map_err(|e| anyhow::anyhow!("Wallet error: {:?}", e))?;
    println!("\nUser address: {}", wallet.classic_address);

    let limit_amount = IssuedCurrencyAmount::new(
        Cow::Owned(currency_code.to_string()),
        Cow::Owned(issuer_address.to_string()),
        Cow::Owned(limit.to_string()),
    );

    let mut trust_set = TrustSet::new(
        Cow::Owned(wallet.classic_address.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        limit_amount,
        None,
        None,
    );

    println!("Submitting trustline transaction...");

    let result = sign_and_submit(&mut trust_set, client, &wallet, true, false)
        .await
        .map_err(|e| anyhow::anyhow!("Transaction error: {:?}", e))?;

    let tx_hash = result
        .tx_json
        .get("hash")
        .and_then(|h| h.as_str())
        .unwrap_or("")
        .to_string();

    println!("Trustline transaction submitted successfully!");
    println!("\nTransaction hash: {}", tx_hash);
    println!("\nEngine result: {}", result.engine_result);
    Ok(tx_hash)
}

pub async fn send_issued_token(
    client: &XRPLClientType,
    issuer_secret: &str,
    user_address: &str,
    currency_code: &str,
    amount: &str,
) -> Result<String> {
    println!("Preparing issued token transfer...");
    println!("\nIssuer seed: {}...", &issuer_secret[..8]);
    println!("\nTo address: {}", user_address);
    println!("\nCurrency: {}", currency_code);
    println!("\nAmount: {}", amount);

    let wallet =
        Wallet::new(issuer_secret, 0).map_err(|e| anyhow::anyhow!("Wallet error: {:?}", e))?;
    println!("\nIssuer address: {}", wallet.classic_address);

    let issued_amount = IssuedCurrencyAmount::new(
        Cow::Owned(currency_code.to_string()),
        Cow::Owned(wallet.classic_address.clone()),
        Cow::Owned(amount.to_string()),
    );

    let mut payment = Payment::new(
        Cow::Owned(wallet.classic_address.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Amount::IssuedCurrencyAmount(issued_amount),
        Cow::Owned(user_address.to_string()),
        None,
        None,
        None,
        None,
        None,
    );

    println!("Submitting issued token transaction...");

    let result = sign_and_submit(&mut payment, client, &wallet, true, false)
        .await
        .map_err(|e| anyhow::anyhow!("Transaction error: {:?}", e))?;

    let tx_hash = result
        .tx_json
        .get("hash")
        .and_then(|h| h.as_str())
        .unwrap_or("")
        .to_string();

    println!("Issued token transaction submitted successfully!");
    println!("\nTransaction hash: {}", tx_hash);
    println!("\nEngine result: {}", result.engine_result);
    Ok(tx_hash)
}
