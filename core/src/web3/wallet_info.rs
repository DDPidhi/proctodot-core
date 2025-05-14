use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub mnemonic: String,
    pub private_key: String,
    pub public_key: String,
    pub address: String,
}