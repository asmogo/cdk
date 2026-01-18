mod auth_connector;
mod auth_wallet;

pub use auth_connector::AuthMintConnector;
pub use auth_wallet::AuthWallet;
use cdk_common::{Amount, AuthProof, AuthToken, Proofs};
use tracing::instrument;

use super::Wallet;
use crate::error::Error;

impl Wallet {
    /// Mint blind auth tokens
    #[instrument(skip_all)]
    pub async fn mint_blind_auth(&self, amount: Amount) -> Result<Proofs, Error> {
        println!("[DEBUG] mint_blind_auth: called with amount: {}", amount);

        let auth_wallet_guard = self.auth_wallet.read().await;
        match auth_wallet_guard.as_ref() {
            Some(auth_wallet) => {
                println!("[DEBUG] mint_blind_auth: auth_wallet is set, minting blind auth tokens");
                let result = auth_wallet.mint_blind_auth(amount).await;
                match &result {
                    Ok(proofs) => println!(
                        "[INFO] mint_blind_auth: successfully minted {} blind auth proofs",
                        proofs.len()
                    ),
                    Err(e) => println!("[ERROR] mint_blind_auth: failed to mint: {:?}", e),
                }
                result
            }
            None => {
                println!("[ERROR] mint_blind_auth: auth_wallet is None - returning AuthSettingsUndefined error");
                Err(Error::AuthSettingsUndefined)
            }
        }
    }

    /// Get unspent auth proofs
    #[instrument(skip_all)]
    pub async fn get_unspent_auth_proofs(&self) -> Result<Vec<AuthProof>, Error> {
        println!("[DEBUG] get_unspent_auth_proofs: called");

        let auth_wallet_guard = self.auth_wallet.read().await;
        println!("[DEBUG] get_unspent_auth_proofs: acquired auth_wallet read lock");

        match auth_wallet_guard.as_ref() {
            Some(auth_wallet) => {
                println!("[DEBUG] get_unspent_auth_proofs: auth_wallet is set, calling get_unspent_auth_proofs");
                let result = auth_wallet.get_unspent_auth_proofs().await;
                match &result {
                    Ok(proofs) => println!(
                        "[INFO] get_unspent_auth_proofs: successfully retrieved {} auth proofs",
                        proofs.len()
                    ),
                    Err(e) => println!(
                        "[ERROR] get_unspent_auth_proofs: error from auth_wallet: {:?}",
                        e
                    ),
                }
                result
            }
            None => {
                println!("[ERROR] get_unspent_auth_proofs: auth_wallet is None - returning AuthSettingsUndefined error");
                println!("[ERROR] get_unspent_auth_proofs: You need to call set_auth_client() or initialize_auth() before using auth features");
                Err(Error::AuthSettingsUndefined)
            }
        }
    }

    /// Set Clear Auth Token (CAT) for authentication
    #[instrument(skip_all)]
    pub async fn set_cat(&self, cat: String) -> Result<(), Error> {
        println!("[DEBUG] set_cat: called with token length: {}", cat.len());

        let auth_wallet = self.auth_wallet.read().await;
        match auth_wallet.as_ref() {
            Some(auth_wallet) => {
                println!("[DEBUG] set_cat: auth_wallet is set, setting CAT token");
                let result = auth_wallet.set_auth_token(AuthToken::ClearAuth(cat)).await;
                match &result {
                    Ok(_) => println!("[INFO] set_cat: successfully set CAT token"),
                    Err(e) => println!("[ERROR] set_cat: failed to set CAT token: {:?}", e),
                }
                result
            }
            None => {
                println!("[WARN] set_cat: auth_wallet is None, CAT token not set (this is OK if you haven't initialized auth yet)");
                Ok(())
            }
        }
    }

    /// Set refresh for authentication
    #[instrument(skip_all)]
    pub async fn set_refresh_token(&self, refresh_token: String) -> Result<(), Error> {
        println!(
            "[DEBUG] set_refresh_token: called with token length: {}",
            refresh_token.len()
        );

        let auth_wallet = self.auth_wallet.read().await;
        match auth_wallet.as_ref() {
            Some(auth_wallet) => {
                println!("[DEBUG] set_refresh_token: auth_wallet is set, setting refresh token");
                auth_wallet.set_refresh_token(Some(refresh_token)).await;
                println!("[INFO] set_refresh_token: successfully set refresh token");
            }
            None => {
                println!("[WARN] set_refresh_token: auth_wallet is None, refresh token not set");
            }
        }
        Ok(())
    }

    /// Refresh CAT token
    #[instrument(skip(self))]
    pub async fn refresh_access_token(&self) -> Result<(), Error> {
        println!("[DEBUG] refresh_access_token: called");

        let auth_wallet = self.auth_wallet.read().await;
        match auth_wallet.as_ref() {
            Some(auth_wallet) => {
                println!("[DEBUG] refresh_access_token: auth_wallet is set, refreshing token");
                let result = auth_wallet.refresh_access_token().await;
                match &result {
                    Ok(_) => {
                        println!("[INFO] refresh_access_token: successfully refreshed access token")
                    }
                    Err(e) => println!(
                        "[ERROR] refresh_access_token: failed to refresh token: {:?}",
                        e
                    ),
                }
                result
            }
            None => {
                println!("[WARN] refresh_access_token: auth_wallet is None, cannot refresh token");
                Ok(())
            }
        }
    }

    /// Set the auth client (AuthWallet) for this wallet
    ///
    /// This allows updating the auth wallet without recreating the wallet.
    /// Also updates the client's auth wallet to keep them in sync.
    #[instrument(skip_all)]
    pub async fn set_auth_client(&self, auth_wallet: Option<AuthWallet>) {
        match &auth_wallet {
            Some(_) => {
                println!("[INFO] set_auth_client: setting auth_wallet (enabling auth features)")
            }
            None => {
                println!("[INFO] set_auth_client: clearing auth_wallet (disabling auth features)")
            }
        }

        let mut auth_wallet_guard = self.auth_wallet.write().await;
        *auth_wallet_guard = auth_wallet.clone();
        println!("[DEBUG] set_auth_client: updated wallet's auth_wallet");

        // Also update the client's auth wallet to keep them in sync
        self.client.set_auth_wallet(auth_wallet).await;
        println!("[DEBUG] set_auth_client: updated client's auth_wallet");
    }

    /// Get the auth wallet
    pub async fn get_auth_wallet(&self) -> Option<AuthWallet> {
        self.auth_wallet.read().await.clone()
    }
}
