use crate::client;
use anyhow::Result;
use serde_json::Value;
use xrpl::{
    asynch::clients::{AsyncWebSocketClient, WebSocketOpen},
    models::results::tx::TxVersionMap,
};

type XRPLClientType =
    AsyncWebSocketClient<xrpl::asynch::clients::SingleExecutorMutex, WebSocketOpen>;

pub async fn verify_transfer(
    client: &XRPLClientType,
    tx_hash: &str,
    expected_from: &str,
    expected_to: &str,
    expected_amount: &str,
    currency_code: Option<&str>,
) -> Result<bool> {
    println!("Verifying transfer...");
    println!("\nTransaction: {}", tx_hash);
    println!("\nExpected from: {}", expected_from);
    println!("\nExpected to: {}", expected_to);
    println!("\nExpected amount: {}", expected_amount);
    if let Some(currency) = currency_code {
        println!("\nExpected currency: {}", currency);
    } else {
        println!("\nExpected currency: XRP");
    }

    let tx_result = client::get_transaction(client, tx_hash).await?;

    let tx_json = match &tx_result {
        TxVersionMap::Default(tx) => &tx.tx_json,
        TxVersionMap::V1(tx_v1) => &tx_v1.tx_json,
    };

    if let Some(transaction_type) = tx_json.get("TransactionType") {
        if transaction_type != "Payment" {
            println!("Transaction is not a Payment transaction");
            return Ok(false);
        }
    } else {
        println!("Transaction type not found");
        return Ok(false);
    }

    println!("Verifying Payment transaction...");

    let actual_from = tx_json
        .get("Account")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Account field not found"))?;

    if actual_from != expected_from {
        println!(
            "Sender mismatch: expected {}, got {}",
            expected_from, actual_from
        );
        return Ok(false);
    }
    println!("Sender verified: {}", actual_from);

    let actual_to = tx_json
        .get("Destination")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Destination field not found"))?;

    if actual_to != expected_to {
        println!(
            "Destination mismatch: expected {}, got {}",
            expected_to, actual_to
        );
        return Ok(false);
    }
    println!("Destination verified: {}", actual_to);

    let amount_field = tx_json
        .get("Amount")
        .ok_or_else(|| anyhow::anyhow!("Amount field not found"))?;

    match amount_field {
        Value::String(amount_str) => {
            if currency_code.is_some() {
                println!("Expected issued currency but got XRP");
                return Ok(false);
            }
            if amount_str != expected_amount {
                println!(
                    "XRP amount mismatch: expected {}, got {}",
                    expected_amount, amount_str
                );
                return Ok(false);
            }
            println!("XRP amount verified: {} drops", amount_str);
        }
        Value::Object(amount_obj) => match currency_code {
            Some(expected_currency) => {
                let actual_currency = amount_obj
                    .get("currency")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Currency field not found"))?;

                let actual_amount = amount_obj
                    .get("value")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Value field not found"))?;

                let actual_issuer = amount_obj
                    .get("issuer")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Issuer field not found"))?;

                if actual_currency != expected_currency {
                    println!(
                        "Currency mismatch: expected {}, got {}",
                        expected_currency, actual_currency
                    );
                    return Ok(false);
                }
                if actual_amount != expected_amount {
                    println!(
                        "Amount mismatch: expected {}, got {}",
                        expected_amount, actual_amount
                    );
                    return Ok(false);
                }
                if actual_issuer != expected_from {
                    println!(
                        "Issuer mismatch: expected {}, got {}",
                        expected_from, actual_issuer
                    );
                    return Ok(false);
                }
                println!(
                    "Issued currency verified: {} {} (issuer: {})",
                    actual_amount, actual_currency, actual_issuer
                );
            }
            None => {
                println!("Expected XRP but got issued currency");
                return Ok(false);
            }
        },
        _ => {
            println!("Invalid amount format");
            return Ok(false);
        }
    }

    println!("Transfer verification successful!");
    Ok(true)
}
