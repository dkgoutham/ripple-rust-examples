# Ripple Task

## Functionalities Implemented

### Part 1: Core XRP Operations
- **XRP Transfer** – Send native XRP between accounts with automatic fee and sequence autofill.
- **Transaction Verification** – Query the ledger and validate transaction details after submission.
- **Trustline Setup** – Create a trustline so the receiver can hold an issued currency.
- **Token Issuance** – Send an issued currency from issuer → receiver.
- **Token Transfer Verification** – Validate transactions (issuer, currency code, amount, destination).

### Part 2: Secure Offline Signing & Air-Gapped Workflows
- **Parameter Gathering** – Collect sequence numbers and current ledger information online.
- **True Offline Signing** – Sign transactions completely offline with zero network calls.
- **Transaction Expiration** – All offline transactions include mandatory expiration bounds (10 ledgers ~50 seconds).
- **Signed Blob Submission** – Submit pre-signed transaction blobs via different connections.
- **Replay Attack Prevention** – Time-bounded transactions prevent malicious reuse of old signed blobs.
- **Cross-Connection Verification** – Verify transactions submitted via different infrastructure.


## Architecture & Security

- **Air-Gapped Signing** – Signing phase requires zero network connectivity
- **Transaction Expiration** – All offline transactions expire after 10 ledgers for security
- **Infrastructure Separation** – Parameter gathering, signing, and submission can use different connections
- **Comprehensive Validation** – Multi-layer security checks and error handling
- **Replay Attack Prevention** – Expired transactions cannot be resubmitted


## Step-by-Step Flows

### Part 1: XRP Transfer Process
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

### Part 1: Issued Token Process
1.  **Trustline Creation (Receiver → Issuer)**
    - The receiver submits a `TrustSet` with a currency limit to the issuer.
2.  **Token Issuance (Issuer → Receiver)**
    - The issuer submits a `Payment` using an `IssuedCurrencyAmount` (with issuer, currency, value).
3.  **Verification**
    - Fetch the tx by hash and confirm issuer, currency, amount, and destination are correct.

### Part 2: Secure Offline Signing Process
1.  **Parameter Gathering (Connection A - Online)**
    - Call `gather_transaction_params()` to get current ledger index and account sequence.
    - Retrieves validated ledger number using `get_latest_validated_ledger_sequence()`.
    - Calculates expiration: `current_ledger + 10` for security.
2.  **Transaction Construction & Signing (Air-Gapped Environment)**
    - Use `offline_sign_transaction()` with pre-gathered parameters.
    - Constructs `Payment` with manual sequence, fee, and expiration bounds.
    - Signs transaction using `sign()` - zero network calls.
    - Encodes to signed hex blob using `encode()`.
3.  **Blob Submission (Connection B - Different Infrastructure)**
    - Create separate XRPL connection via `create_second_connection()`.
    - Submit signed blob using `submit_signed_blob()`.
    - Validates current ledger to ensure transaction hasn't expired.
4.  **Verification**
    - Query submitted transaction and validate all fields match expectations.



## Prerequisites
- Rust (stable) and Cargo
- `.env` file with testnet seeds
- Access to XRPL Testnet


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
# Build the project
cargo build

# Run all demos (Part 1 + Part 2)
cargo run