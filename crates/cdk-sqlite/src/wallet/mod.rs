//! SQLite Wallet Database

use std::path::PathBuf;
use std::sync::Arc;

use cdk_common::database::Error;
use cdk_sql_common::database::DatabaseExecutor;
use cdk_sql_common::pool::{Pool, PooledResource};
use cdk_sql_common::stmt::{Column, SqlPart, Statement};
use cdk_sql_common::SQLWalletDatabase;
use rusqlite::CachedStatement;

use crate::common::{create_sqlite_pool, from_sqlite, to_sqlite, SqliteConnectionManager};

pub mod memory;

/// Simple Sqlite wapper, since the wallet may not need rusqlite with concurrency, a shared instance
/// may be enough
#[derive(Debug)]
pub struct SimpleAsyncRusqlite(Arc<Pool<SqliteConnectionManager>>);

impl SimpleAsyncRusqlite {
    fn get_stmt<'a>(
        &self,
        conn: &'a PooledResource<SqliteConnectionManager>,
        statement: Statement,
    ) -> Result<CachedStatement<'a>, Error> {
        let (sql, placeholder_values) = statement.to_sql()?;
        let mut stmt = conn
            .prepare_cached(&sql)
            .map_err(|e| Error::Database(Box::new(e)))?;

        for (i, value) in placeholder_values.into_iter().enumerate() {
            stmt.raw_bind_parameter(i + 1, to_sqlite(value))
                .map_err(|e| Error::Database(Box::new(e)))?;
        }

        Ok(stmt)
    }
}

#[async_trait::async_trait]
impl DatabaseExecutor for SimpleAsyncRusqlite {
    fn name() -> &'static str {
        "sqlite"
    }

    async fn execute(&self, statement: Statement) -> Result<usize, Error> {
        let conn = self.0.get().map_err(|e| Error::Database(Box::new(e)))?;
        let mut stmt = self
            .get_stmt(&conn, statement)
            .map_err(|e| Error::Database(Box::new(e)))?;

        Ok(stmt
            .raw_execute()
            .map_err(|e| Error::Database(Box::new(e)))?)
    }

    async fn fetch_one(&self, statement: Statement) -> Result<Option<Vec<Column>>, Error> {
        let conn = self.0.get().map_err(|e| Error::Database(Box::new(e)))?;
        let mut stmt = self
            .get_stmt(&conn, statement)
            .map_err(|e| Error::Database(Box::new(e)))?;

        let columns = stmt.column_count();

        let mut rows = stmt.raw_query();
        rows.next()
            .map_err(|e| Error::Database(Box::new(e)))?
            .map(|row| {
                (0..columns)
                    .map(|i| row.get(i).map(from_sqlite))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()
            .map_err(|e| Error::Database(Box::new(e)))
    }

    async fn fetch_all(&self, statement: Statement) -> Result<Vec<Vec<Column>>, Error> {
        let conn = self.0.get().map_err(|e| Error::Database(Box::new(e)))?;
        let mut stmt = self
            .get_stmt(&conn, statement)
            .map_err(|e| Error::Database(Box::new(e)))?;

        let columns = stmt.column_count();

        let mut rows = stmt.raw_query();
        let mut results = vec![];

        while let Some(row) = rows.next().map_err(|e| Error::Database(Box::new(e)))? {
            results.push(
                (0..columns)
                    .map(|i| row.get(i).map(from_sqlite))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| Error::Database(Box::new(e)))?,
            )
        }

        Ok(results)
    }

    async fn pluck(&self, statement: Statement) -> Result<Option<Column>, Error> {
        let conn = self.0.get().map_err(|e| Error::Database(Box::new(e)))?;
        let mut stmt = self
            .get_stmt(&conn, statement)
            .map_err(|e| Error::Database(Box::new(e)))?;

        let mut rows = stmt.raw_query();
        rows.next()
            .map_err(|e| Error::Database(Box::new(e)))?
            .map(|row| row.get(0usize).map(from_sqlite))
            .transpose()
            .map_err(|e| Error::Database(Box::new(e)))
    }

    async fn batch(&self, mut statement: Statement) -> Result<(), Error> {
        let conn = self.0.get().map_err(|e| Error::Database(Box::new(e)))?;

        let sql = {
            let part = statement
                .parts
                .pop()
                .ok_or(Error::Internal("Empty SQL".to_owned()))?;

            if !statement.parts.is_empty() || matches!(part, SqlPart::Placeholder(_, _)) {
                return Err(Error::Internal(
                    "Invalid usage, batch does not support placeholders".to_owned(),
                ));
            }

            if let SqlPart::Raw(sql) = part {
                sql
            } else {
                unreachable!()
            }
        };

        conn.execute_batch(&sql)
            .map_err(|e| Error::Database(Box::new(e)))
    }
}

impl From<PathBuf> for SimpleAsyncRusqlite {
    fn from(value: PathBuf) -> Self {
        SimpleAsyncRusqlite(create_sqlite_pool(value.to_str().unwrap_or_default(), None))
    }
}

impl From<&str> for SimpleAsyncRusqlite {
    fn from(value: &str) -> Self {
        SimpleAsyncRusqlite(create_sqlite_pool(value, None))
    }
}

impl From<(&str, &str)> for SimpleAsyncRusqlite {
    fn from((value, pass): (&str, &str)) -> Self {
        SimpleAsyncRusqlite(create_sqlite_pool(value, Some(pass.to_owned())))
    }
}

impl From<(PathBuf, &str)> for SimpleAsyncRusqlite {
    fn from((value, pass): (PathBuf, &str)) -> Self {
        SimpleAsyncRusqlite(create_sqlite_pool(
            value.to_str().unwrap_or_default(),
            Some(pass.to_owned()),
        ))
    }
}

impl From<(&str, String)> for SimpleAsyncRusqlite {
    fn from((value, pass): (&str, String)) -> Self {
        SimpleAsyncRusqlite(create_sqlite_pool(value, Some(pass)))
    }
}

impl From<(PathBuf, String)> for SimpleAsyncRusqlite {
    fn from((value, pass): (PathBuf, String)) -> Self {
        SimpleAsyncRusqlite(create_sqlite_pool(
            value.to_str().unwrap_or_default(),
            Some(pass),
        ))
    }
}

impl From<&PathBuf> for SimpleAsyncRusqlite {
    fn from(value: &PathBuf) -> Self {
        SimpleAsyncRusqlite(create_sqlite_pool(value.to_str().unwrap_or_default(), None))
    }
}

/// Mint SQLite implementation with rusqlite
pub type WalletSqliteDatabase = SQLWalletDatabase<SimpleAsyncRusqlite>;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use cdk_common::database::WalletDatabase;
    use cdk_common::nuts::{ProofDleq, State};
    use cdk_common::secret::Secret;

    use crate::WalletSqliteDatabase;

    #[tokio::test]
    #[cfg(feature = "sqlcipher")]
    async fn test_sqlcipher() {
        use cdk_common::mint_url::MintUrl;
        use cdk_common::MintInfo;

        use super::*;
        let path = std::env::temp_dir()
            .to_path_buf()
            .join(format!("cdk-test-{}.sqlite", uuid::Uuid::new_v4()));
        let db = WalletSqliteDatabase::new((path, "password".to_string()))
            .await
            .unwrap();

        let mint_info = MintInfo::new().description("test");
        let mint_url = MintUrl::from_str("https://mint.xyz").unwrap();

        db.add_mint(mint_url.clone(), Some(mint_info.clone()))
            .await
            .unwrap();

        let res = db.get_mint(mint_url).await.unwrap();
        assert_eq!(mint_info, res.clone().unwrap());
        assert_eq!("test", &res.unwrap().description.unwrap());
    }

    #[tokio::test]
    async fn test_proof_with_dleq() {
        use cdk_common::common::ProofInfo;
        use cdk_common::mint_url::MintUrl;
        use cdk_common::nuts::{CurrencyUnit, Id, Proof, PublicKey, SecretKey};
        use cdk_common::Amount;

        // Create a temporary database
        let path = std::env::temp_dir()
            .to_path_buf()
            .join(format!("cdk-test-dleq-{}.sqlite", uuid::Uuid::new_v4()));

        #[cfg(feature = "sqlcipher")]
        let db = WalletSqliteDatabase::new((path, "password".to_string()))
            .await
            .unwrap();

        #[cfg(not(feature = "sqlcipher"))]
        let db = WalletSqliteDatabase::new(path).await.unwrap();

        // Create a proof with DLEQ
        let keyset_id = Id::from_str("00deadbeef123456").unwrap();
        let mint_url = MintUrl::from_str("https://example.com").unwrap();
        let secret = Secret::new("test_secret_for_dleq");

        // Create DLEQ components
        let e = SecretKey::generate();
        let s = SecretKey::generate();
        let r = SecretKey::generate();

        let dleq = ProofDleq::new(e.clone(), s.clone(), r.clone());

        let mut proof = Proof::new(
            Amount::from(64),
            keyset_id,
            secret,
            PublicKey::from_hex(
                "02deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
            )
            .unwrap(),
        );

        // Add DLEQ to the proof
        proof.dleq = Some(dleq);

        // Create ProofInfo
        let proof_info =
            ProofInfo::new(proof, mint_url.clone(), State::Unspent, CurrencyUnit::Sat).unwrap();

        // Store the proof in the database
        db.update_proofs(vec![proof_info.clone()], vec![])
            .await
            .unwrap();

        // Retrieve the proof from the database
        let retrieved_proofs = db
            .get_proofs(
                Some(mint_url),
                Some(CurrencyUnit::Sat),
                Some(vec![State::Unspent]),
                None,
            )
            .await
            .unwrap();

        // Verify we got back exactly one proof
        assert_eq!(retrieved_proofs.len(), 1);

        // Verify the DLEQ data was preserved
        let retrieved_proof = &retrieved_proofs[0];
        assert!(retrieved_proof.proof.dleq.is_some());

        let retrieved_dleq = retrieved_proof.proof.dleq.as_ref().unwrap();

        // Verify DLEQ components match what we stored
        assert_eq!(retrieved_dleq.e.to_string(), e.to_string());
        assert_eq!(retrieved_dleq.s.to_string(), s.to_string());
        assert_eq!(retrieved_dleq.r.to_string(), r.to_string());
    }

    #[tokio::test]
    async fn test_mint_quote_payment_method_read_and_write() {
        use cdk_common::mint_url::MintUrl;
        use cdk_common::nuts::{CurrencyUnit, MintQuoteState, PaymentMethod};
        use cdk_common::wallet::MintQuote;
        use cdk_common::Amount;

        // Create a temporary database
        let path = std::env::temp_dir().to_path_buf().join(format!(
            "cdk-test-migration-{}.sqlite",
            uuid::Uuid::new_v4()
        ));

        #[cfg(feature = "sqlcipher")]
        let db = WalletSqliteDatabase::new((path, "password".to_string()))
            .await
            .unwrap();

        #[cfg(not(feature = "sqlcipher"))]
        let db = WalletSqliteDatabase::new(path).await.unwrap();

        // Test PaymentMethod variants
        let mint_url = MintUrl::from_str("https://example.com").unwrap();
        let payment_methods = vec![
            PaymentMethod::Bolt11,
            PaymentMethod::Bolt12,
            PaymentMethod::Custom("custom".to_string()),
        ];

        for (i, payment_method) in payment_methods.iter().enumerate() {
            let quote = MintQuote {
                id: format!("test_quote_{}", i),
                mint_url: mint_url.clone(),
                amount: Some(Amount::from(100)),
                unit: CurrencyUnit::Sat,
                request: "test_request".to_string(),
                state: MintQuoteState::Unpaid,
                expiry: 1000000000,
                secret_key: None,
                payment_method: payment_method.clone(),
                amount_issued: Amount::from(0),
                amount_paid: Amount::from(0),
            };

            // Store the quote
            db.add_mint_quote(quote.clone()).await.unwrap();

            // Retrieve and verify
            let retrieved = db.get_mint_quote(&quote.id).await.unwrap().unwrap();
            assert_eq!(retrieved.payment_method, *payment_method);
            assert_eq!(retrieved.amount_issued, Amount::from(0));
            assert_eq!(retrieved.amount_paid, Amount::from(0));
        }
    }
}
