use crate::web3::wallet_info::WalletInfo;
use sp_core::{crypto::Ss58Codec, sr25519, Pair};
use sp_runtime::AccountId32;

pub struct WalletHandler {}
impl WalletHandler {
    pub fn generate_wallet() -> WalletInfo {
        // Generate a new keypair with mnemonic phrase
        let (pair, phrase, _) = sr25519::Pair::generate_with_phrase(None);

        // Get the SS58 encoded public address from the public key1100
        let public_key = pair.public().to_ss58check();

        // Convert private key to hex format
        let private_key = hex::encode(pair.to_raw_vec());

        // Convert the public key to a 32-byte array and then to AccountId32
        let public_key_bytes: [u8; 32] = pair.public().0; // Extract the public key bytes
        let account_id = AccountId32::from(public_key_bytes);

        // Get the SS58 encoded address (from AccountId32)
        let address = account_id.to_string();

        // Return the struct with all the information
        WalletInfo {
            mnemonic: phrase,
            private_key,
            public_key,
            address,
        }
    }
}