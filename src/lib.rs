pub mod client;
pub mod error;
pub mod offline_signing;
pub mod transactions;
pub mod verification;

use anyhow::Result;
use xrpl::{
    asynch::clients::{AsyncWebSocketClient, WebSocketOpen},
    wallet::Wallet,
};

pub use error::RippleError;

pub use xrpl::{
    asynch::clients::client::XRPLClient, models::transactions::CommonTransactionBuilder,
};

pub struct XRPLManager {
    client: AsyncWebSocketClient<xrpl::asynch::clients::SingleExecutorMutex, WebSocketOpen>,
}

impl XRPLManager {
    pub async fn new_testnet() -> Result<Self> {
        println!("Connecting to XRPL Testnet...");
        let url = url::Url::parse("wss://s.altnet.rippletest.net:51233")?;
        let client = AsyncWebSocketClient::open(url).await?;
        println!("Connected to XRPL Testnet");

        Ok(Self { client })
    }

    // Part 1 functionality
    pub async fn send_xrp(
        &self,
        user1_secret: &str,
        user2_address: &str,
        amount_drops: u64,
    ) -> Result<String> {
        transactions::send_xrp(&self.client, user1_secret, user2_address, amount_drops).await
    }

    pub async fn send_issued_token(
        &self,
        user1_secret: &str,
        user2_address: &str,
        currency_code: &str,
        amount: &str,
    ) -> Result<String> {
        transactions::send_issued_token(
            &self.client,
            user1_secret,
            user2_address,
            currency_code,
            amount,
        )
        .await
    }

    pub async fn setup_trustline(
        &self,
        user_secret: &str,
        issuer_address: &str,
        currency_code: &str,
        limit: &str,
    ) -> Result<String> {
        transactions::setup_trustline(
            &self.client,
            user_secret,
            issuer_address,
            currency_code,
            limit,
        )
        .await
    }

    pub async fn verify_transfer(
        &self,
        tx_hash: &str,
        expected_from: &str,
        expected_to: &str,
        expected_amount: &str,
        currency_code: Option<&str>,
    ) -> Result<bool> {
        verification::verify_transfer(
            &self.client,
            tx_hash,
            expected_from,
            expected_to,
            expected_amount,
            currency_code,
        )
        .await
    }

    // Part 2 functionality - True offline signing
    pub async fn gather_transaction_params(&self, account_address: &str) -> Result<offline_signing::OfflineTransactionParams> {
        offline_signing::gather_transaction_params(&self.client, account_address).await
    }

    pub fn offline_sign_transaction(
        user_secret: &str,
        to_address: &str,
        amount: xrpl::models::Amount<'static>,
        params: offline_signing::OfflineTransactionParams,
    ) -> Result<String> {
        offline_signing::offline_sign_transaction(user_secret, to_address, amount, params)
    }

    pub async fn submit_signed_blob(&self, signed_blob: &str) -> Result<String> {
        offline_signing::submit_signed_blob(&self.client, signed_blob).await
    }

    // High-level workflows
    pub async fn offline_xrp_workflow(
        &self,
        offline_client: &XRPLManager,
        user_secret: &str,
        to_address: &str,
        amount_drops: u64,
    ) -> Result<String> {
        offline_signing::offline_xrp_workflow(&self.client, &offline_client.client, user_secret, to_address, amount_drops).await
    }

    pub async fn offline_token_workflow(
        &self,
        offline_client: &XRPLManager,
        user_secret: &str,
        to_address: &str,
        currency_code: &str,
        amount: &str,
    ) -> Result<String> {
        offline_signing::offline_token_workflow(&self.client, &offline_client.client, user_secret, to_address, currency_code, amount).await
    }

    // Utility to create a second connection
    pub async fn create_second_connection() -> Result<XRPLManager> {
        Self::new_testnet().await
    }
}

pub fn create_test_wallet() -> Result<Wallet> {
    let wallet =
        Wallet::create(None).map_err(|e| anyhow::anyhow!("Wallet creation error: {:?}", e))?;
    println!("Created new wallet: {}", wallet.classic_address);
    println!("   Seed: {}", wallet.seed);
    Ok(wallet)
}

pub fn wallet_from_seed(seed: &str) -> Result<Wallet> {
    let wallet =
        Wallet::new(seed, 0).map_err(|e| anyhow::anyhow!("Wallet creation error: {:?}", e))?;
    println!("Loaded wallet: {}", wallet.classic_address);
    Ok(wallet)
}
