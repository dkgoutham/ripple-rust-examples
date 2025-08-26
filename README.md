# Ripple Task

## Functionalities Implemented

### Core XRP Operations
- **XRP Transfer** – Send native XRP between accounts with automatic fee and sequence autofill.
- **Transaction Verification** – Query the ledger and validate transaction details after submission.

### Issued Token Operations
- **Trustline Setup** – Create a trustline so the receiver can hold an issued currency.
- **Token Issuance** – Send an issued currency from issuer → receiver.
- **Token Transfer Verification** – Validate transactions (issuer, currency code, amount, destination).


## Step-by-Step Flow

### XRP Transfer Process
1.  **Create wallets from seeds**
    - Use `Wallet::new()` to derive wallets from the testnet seeds.
2.  **Construct a Payment**
    - Build a `Payment` with the XRP amount, e.g. `XRPAmount(Cow::Owned(amount))`.
3.  **Sign & Submit with Autofill**
    - Call `sign_and_submit()` with `autofill: true` to automatically set `Fee` and `Sequence`.
4.  **Capture the Tx Hash**
    - Read the transaction hash from the submit response JSON.
5.  **Verify on Ledger**
    - Query the transaction by hash and compare `Account`, `Destination`, and `Amount` fields.

### Issued Token Process
1.  **Trustline Creation (Receiver → Issuer)**
    - The receiver submits a `TrustSet` with a currency limit to the issuer.
2.  **Token Issuance (Issuer → Receiver)**
    - The issuer submits a `Payment` using an `IssuedCurrencyAmount` (with issuer, currency, value).
3.  **Verification**
    - Fetch the tx by hash and confirm issuer, currency, amount, and destination are correct.



## Prerequisites
- Rust (stable) and Cargo
- `.env` file 
- Access to XRPL Testnet)



## Setup

1.  **Get Testnet Seeds**
    - Use the official XRPL Testnet faucet to generate two accounts (issuer and receiver):
    - [https://xrpl.org/xrp-testnet-faucet.html](https://xrpl.org/xrp-testnet-faucet.html)

2.  **Create a `.env` file**
    - Put your actual Testnet seeds here:
    ```env
    # .env
    USER1_SEED=your_actual_user1_seed_here
    USER2_SEED=your_actual_user2_seed_here
    ```
    - `USER1` can be the issuer and `USER2` the receiver.


## Build & Run
```bash
cargo build
cargo run