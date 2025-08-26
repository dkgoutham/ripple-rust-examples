pub mod client;
pub mod error;
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
