use std::collections::HashMap;
use std::sync::Arc;

use cdk_supabase::SupabaseWalletDatabase as CdkSupabaseWalletDatabase;
use url::Url;

use crate::{
    CurrencyUnit, FfiError, Id, KeySet, KeySetInfo, Keys, MeltQuote, MintInfo, MintQuote, MintUrl,
    ProofInfo, ProofState, PublicKey, SpendingConditions, Transaction, TransactionDirection,
    TransactionId, WalletDatabase,
};

#[derive(uniffi::Object)]
pub struct WalletSupabaseDatabase {
    inner: Arc<CdkSupabaseWalletDatabase>,
}

#[uniffi::export]
impl WalletSupabaseDatabase {
    /// Create a new Supabase-backed wallet database
    /// Requires cdk-ffi to be built with feature "supabase".
    /// Example URL: "https://your-project.supabase.co"
    /// The API key should be your Supabase anon or service role key
    #[cfg(feature = "supabase")]
    #[uniffi::constructor]
    pub fn new(url: String, api_key: String) -> Result<Arc<Self>, FfiError> {
        let parsed_url = Url::parse(&url).map_err(|e| FfiError::Database { msg: e.to_string() })?;

        let inner = CdkSupabaseWalletDatabase::new(parsed_url, api_key);

        Ok(Arc::new(WalletSupabaseDatabase {
            inner: Arc::new(inner),
        }))
    }
}

#[uniffi::export(async_runtime = "tokio")]
#[async_trait::async_trait]
impl WalletDatabase for WalletSupabaseDatabase {
    // ========== Read methods ==========

    async fn get_proofs_by_ys(&self, ys: Vec<PublicKey>) -> Result<Vec<ProofInfo>, FfiError> {
        let cdk_ys: Vec<cdk::nuts::PublicKey> = ys.into_iter().map(Into::into).collect();
        let proofs = self
            .inner
            .get_proofs_by_ys(cdk_ys)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        Ok(proofs.into_iter().map(Into::into).collect())
    }

    async fn get_mint(&self, mint_url: MintUrl) -> Result<Option<MintInfo>, FfiError> {
        let cdk_mint_url: cdk::mint_url::MintUrl = mint_url.into();
        let mint = self
            .inner
            .get_mint(cdk_mint_url)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        Ok(mint.map(Into::into))
    }

    async fn get_mints(&self) -> Result<HashMap<MintUrl, Option<MintInfo>>, FfiError> {
        let mints = self
            .inner
            .get_mints()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        Ok(mints
            .into_iter()
            .map(|(k, v)| (k.into(), v.map(Into::into)))
            .collect())
    }

    async fn get_mint_keysets(
        &self,
        mint_url: MintUrl,
    ) -> Result<Option<Vec<KeySetInfo>>, FfiError> {
        let cdk_mint_url: cdk::mint_url::MintUrl = mint_url.into();
        let keysets = self
            .inner
            .get_mint_keysets(cdk_mint_url)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        Ok(keysets.map(|ks| ks.into_iter().map(Into::into).collect()))
    }

    async fn get_keyset_by_id(&self, keyset_id: Id) -> Result<Option<KeySetInfo>, FfiError> {
        let cdk_id: cdk::nuts::Id = keyset_id.into();
        let keyset = self
            .inner
            .get_keyset_by_id(&cdk_id)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        Ok(keyset.map(Into::into))
    }

    async fn get_mint_quote(&self, quote_id: String) -> Result<Option<MintQuote>, FfiError> {
        let quote = self
            .inner
            .get_mint_quote(&quote_id)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        Ok(quote.map(|q| q.try_into()).transpose()?)
    }

    async fn get_mint_quotes(&self) -> Result<Vec<MintQuote>, FfiError> {
        let quotes = self
            .inner
            .get_mint_quotes()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        quotes
            .into_iter()
            .map(|q| q.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    async fn get_unissued_mint_quotes(&self) -> Result<Vec<MintQuote>, FfiError> {
        let quotes = self
            .inner
            .get_unissued_mint_quotes()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        quotes
            .into_iter()
            .map(|q| q.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    async fn get_melt_quote(&self, quote_id: String) -> Result<Option<MeltQuote>, FfiError> {
        let quote = self
            .inner
            .get_melt_quote(&quote_id)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        Ok(quote.map(|q| q.try_into()).transpose()?)
    }

    async fn get_melt_quotes(&self) -> Result<Vec<MeltQuote>, FfiError> {
        let quotes = self
            .inner
            .get_melt_quotes()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        quotes
            .into_iter()
            .map(|q| q.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    async fn get_keys(&self, id: Id) -> Result<Option<Keys>, FfiError> {
        let cdk_id: cdk::nuts::Id = id.into();
        let keys = self
            .inner
            .get_keys(&cdk_id)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        Ok(keys.map(Into::into))
    }

    async fn get_proofs(
        &self,
        mint_url: Option<MintUrl>,
        unit: Option<CurrencyUnit>,
        state: Option<Vec<ProofState>>,
        spending_conditions: Option<Vec<SpendingConditions>>,
    ) -> Result<Vec<ProofInfo>, FfiError> {
        let cdk_mint_url = mint_url.map(Into::into);
        let cdk_unit = unit.map(Into::into);
        let cdk_state = state.map(|s| s.into_iter().map(Into::into).collect());
        let cdk_spending_conditions =
            spending_conditions.map(|sc| sc.into_iter().map(Into::into).collect());

        let proofs = self
            .inner
            .get_proofs(cdk_mint_url, cdk_unit, cdk_state, cdk_spending_conditions)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        Ok(proofs.into_iter().map(Into::into).collect())
    }

    async fn get_balance(
        &self,
        mint_url: Option<MintUrl>,
        unit: Option<CurrencyUnit>,
        state: Option<Vec<ProofState>>,
    ) -> Result<u64, FfiError> {
        let cdk_mint_url = mint_url.map(Into::into);
        let cdk_unit = unit.map(Into::into);
        let cdk_state = state.map(|s| s.into_iter().map(Into::into).collect());

        self.inner
            .get_balance(cdk_mint_url, cdk_unit, cdk_state)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn get_transaction(
        &self,
        transaction_id: TransactionId,
    ) -> Result<Option<Transaction>, FfiError> {
        let cdk_tx_id: cdk_common::wallet::TransactionId = transaction_id.into();
        let tx = self
            .inner
            .get_transaction(cdk_tx_id)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        Ok(tx.map(|t| t.try_into()).transpose()?)
    }

    async fn list_transactions(
        &self,
        mint_url: Option<MintUrl>,
        direction: Option<TransactionDirection>,
        unit: Option<CurrencyUnit>,
    ) -> Result<Vec<Transaction>, FfiError> {
        let cdk_mint_url = mint_url.map(Into::into);
        let cdk_direction = direction.map(Into::into);
        let cdk_unit = unit.map(Into::into);

        let txs = self
            .inner
            .list_transactions(cdk_mint_url, cdk_direction, cdk_unit)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;
        txs.into_iter()
            .map(|t| t.try_into())
            .collect::<Result<Vec<_>, _>>()
    }

    async fn kv_read(
        &self,
        primary_namespace: String,
        secondary_namespace: String,
        key: String,
    ) -> Result<Option<Vec<u8>>, FfiError> {
        self.inner
            .kv_read(&primary_namespace, &secondary_namespace, &key)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn kv_list(
        &self,
        primary_namespace: String,
        secondary_namespace: String,
    ) -> Result<Vec<String>, FfiError> {
        self.inner
            .kv_list(&primary_namespace, &secondary_namespace)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn kv_write(
        &self,
        primary_namespace: String,
        secondary_namespace: String,
        key: String,
        value: Vec<u8>,
    ) -> Result<(), FfiError> {
        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::KVStoreTransaction;
        tx.kv_write(&primary_namespace, &secondary_namespace, &key, &value)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn kv_remove(
        &self,
        primary_namespace: String,
        secondary_namespace: String,
        key: String,
    ) -> Result<(), FfiError> {
        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::KVStoreTransaction;
        tx.kv_remove(&primary_namespace, &secondary_namespace, &key)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    // ========== Write methods ==========

    async fn update_proofs(
        &self,
        added: Vec<ProofInfo>,
        removed_ys: Vec<PublicKey>,
    ) -> Result<(), FfiError> {
        let cdk_added: Vec<cdk_common::common::ProofInfo> =
            added.into_iter().map(Into::into).collect();
        let cdk_removed_ys: Vec<cdk::nuts::PublicKey> =
            removed_ys.into_iter().map(Into::into).collect();

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.update_proofs(cdk_added, cdk_removed_ys)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn update_proofs_state(
        &self,
        ys: Vec<PublicKey>,
        state: ProofState,
    ) -> Result<(), FfiError> {
        let cdk_ys: Vec<cdk::nuts::PublicKey> = ys.into_iter().map(Into::into).collect();
        let cdk_state: cdk::nuts::State = state.into();

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.update_proofs_state(cdk_ys, cdk_state)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn add_transaction(&self, transaction: Transaction) -> Result<(), FfiError> {
        let cdk_tx: cdk_common::wallet::Transaction = transaction.try_into()?;

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.add_transaction(cdk_tx)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn remove_transaction(&self, transaction_id: TransactionId) -> Result<(), FfiError> {
        let cdk_tx_id: cdk_common::wallet::TransactionId = transaction_id.into();

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.remove_transaction(cdk_tx_id)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn update_mint_url(
        &self,
        old_mint_url: MintUrl,
        new_mint_url: MintUrl,
    ) -> Result<(), FfiError> {
        let cdk_old: cdk::mint_url::MintUrl = old_mint_url.into();
        let cdk_new: cdk::mint_url::MintUrl = new_mint_url.into();

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.update_mint_url(cdk_old, cdk_new)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn increment_keyset_counter(&self, keyset_id: Id, count: u32) -> Result<u32, FfiError> {
        let cdk_id: cdk::nuts::Id = keyset_id.into();

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        let counter = tx
            .increment_keyset_counter(&cdk_id, count)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        Ok(counter)
    }

    async fn add_mint(
        &self,
        mint_url: MintUrl,
        mint_info: Option<MintInfo>,
    ) -> Result<(), FfiError> {
        let cdk_mint_url: cdk::mint_url::MintUrl = mint_url.into();
        let cdk_mint_info = mint_info.map(Into::into);

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.add_mint(cdk_mint_url, cdk_mint_info)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn remove_mint(&self, mint_url: MintUrl) -> Result<(), FfiError> {
        let cdk_mint_url: cdk::mint_url::MintUrl = mint_url.into();

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.remove_mint(cdk_mint_url)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn add_mint_keysets(
        &self,
        mint_url: MintUrl,
        keysets: Vec<KeySetInfo>,
    ) -> Result<(), FfiError> {
        let cdk_mint_url: cdk::mint_url::MintUrl = mint_url.into();
        let cdk_keysets: Vec<cdk::nuts::KeySetInfo> = keysets.into_iter().map(Into::into).collect();

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.add_mint_keysets(cdk_mint_url, cdk_keysets)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn add_mint_quote(&self, quote: MintQuote) -> Result<(), FfiError> {
        let cdk_quote: cdk_common::wallet::MintQuote = quote.try_into()?;

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.add_mint_quote(cdk_quote)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn remove_mint_quote(&self, quote_id: String) -> Result<(), FfiError> {
        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.remove_mint_quote(&quote_id)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn add_melt_quote(&self, quote: MeltQuote) -> Result<(), FfiError> {
        let cdk_quote: cdk_common::wallet::MeltQuote = quote.try_into()?;

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.add_melt_quote(cdk_quote)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn remove_melt_quote(&self, quote_id: String) -> Result<(), FfiError> {
        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.remove_melt_quote(&quote_id)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn add_keys(&self, keyset: KeySet) -> Result<(), FfiError> {
        let cdk_keyset: cdk::nuts::KeySet = keyset.into();

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.add_keys(cdk_keyset)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }

    async fn remove_keys(&self, id: Id) -> Result<(), FfiError> {
        let cdk_id: cdk::nuts::Id = id.into();

        let mut tx = self
            .inner
            .begin_db_transaction()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DatabaseTransaction;
        tx.remove_keys(&cdk_id)
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })?;

        use cdk_common::database::DbTransactionFinalizer;
        tx.commit()
            .await
            .map_err(|e| FfiError::Database { msg: e.to_string() })
    }
}
