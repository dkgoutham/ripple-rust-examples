use anyhow::Result;
use std::borrow::Cow;
use xrpl::{
    asynch::clients::{AsyncWebSocketClient, WebSocketOpen, client::XRPLClient},
    models::{
        requests::{LedgerIndex, account_info::AccountInfo, tx::Tx},
        results::{account_info::AccountInfoVersionMap, tx::TxVersionMap},
    },
};

type XRPLClientType =
    AsyncWebSocketClient<xrpl::asynch::clients::SingleExecutorMutex, WebSocketOpen>;

pub async fn get_account_info(
    client: &XRPLClientType,
    account: &str,
) -> Result<AccountInfoVersionMap<'static>> {
    println!("Getting account info for: {}", account);

    let request = AccountInfo::new(
        None,
        Cow::Owned(account.to_string()),
        None,
        Some(LedgerIndex::Str(Cow::Owned("validated".to_string()))),
        None,
        None,
        None,
    );

    let response = client.request_impl(request.into()).await?;

    match response.result {
        Some(xrpl::models::results::XRPLResult::AccountInfo(info)) => {
            println!("Account info retrieved");
            Ok(info)
        }
        _ => {
            println!("Unexpected response type");
            Err(anyhow::anyhow!("Unexpected response type"))
        }
    }
}

pub async fn get_transaction(
    client: &XRPLClientType,
    tx_hash: &str,
) -> Result<TxVersionMap<'static>> {
    println!("Getting transaction: {}", tx_hash);

    let request = Tx::new(
        None,
        None,
        None,
        None,
        Some(Cow::Owned(tx_hash.to_string())),
    );

    let response = client.request_impl(request.into()).await?;

    match response.result {
        Some(xrpl::models::results::XRPLResult::Tx(tx)) => {
            println!("Transaction retrieved");
            Ok(tx)
        }
        _ => {
            println!("Unexpected response type for transaction");
            Err(anyhow::anyhow!("Unexpected response type"))
        }
    }
}
