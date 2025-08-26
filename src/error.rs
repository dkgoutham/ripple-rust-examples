use std::fmt;

#[derive(Debug)]
pub enum RippleError {
    XRPLClient(String),
    Transaction(String),
    Verification(String),
    Wallet(String),
    Network(String),
    InvalidInput(String),
}

impl fmt::Display for RippleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RippleError::XRPLClient(msg) => write!(f, "XRPL Client Error: {}", msg),
            RippleError::Transaction(msg) => write!(f, "Transaction Error: {}", msg),
            RippleError::Verification(msg) => write!(f, "Verification Error: {}", msg),
            RippleError::Wallet(msg) => write!(f, "Wallet Error: {}", msg),
            RippleError::Network(msg) => write!(f, "Network Error: {}", msg),
            RippleError::InvalidInput(msg) => write!(f, "Invalid Input: {}", msg),
        }
    }
}

impl std::error::Error for RippleError {}

impl From<xrpl::asynch::exceptions::XRPLHelperException> for RippleError {
    fn from(err: xrpl::asynch::exceptions::XRPLHelperException) -> Self {
        RippleError::XRPLClient(format!("{:?}", err))
    }
}

impl From<xrpl::wallet::exceptions::XRPLWalletException> for RippleError {
    fn from(err: xrpl::wallet::exceptions::XRPLWalletException) -> Self {
        RippleError::Wallet(format!("{:?}", err))
    }
}

impl From<xrpl::asynch::clients::exceptions::XRPLClientException> for RippleError {
    fn from(err: xrpl::asynch::clients::exceptions::XRPLClientException) -> Self {
        RippleError::XRPLClient(format!("{:?}", err))
    }
}
