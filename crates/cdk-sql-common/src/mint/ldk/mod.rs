use crate::column_as_string;
use crate::common::migrate;
use crate::database::{ConnectionWithTransaction, DatabaseExecutor};
use async_trait::async_trait;
use cdk_common::database::{Error, MintKVStore};
use lightning::util::persist::KVStore;
use migrations::MIGRATIONS;
use std::sync::Arc;
mod migrations;
use crate::pool::{DatabasePool, Pool, PooledResource};
use crate::stmt::query;

/// Mint SQL Database
#[derive(Clone)]
pub struct SQLLdkDatabase<RM>
where
    RM: DatabasePool + 'static,
{
    pool: Arc<Pool<RM>>,
    inner: Arc<dyn MintKVStore<Err = Error> + Send + Sync>,
}
#[async_trait]
impl<D> KVStore for SQLLdkDatabase<D>
where
    D: DatabasePool + Send + Sync,
    D::Connection: DatabaseExecutor,
{
    fn read(
        &self,
        primary_namespace: &str,
        secondary_namespace: &str,
        key: &str,
    ) -> Result<Vec<u8>, bitcoin::io::Error> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match self
                    .inner
                    .begin_transaction().await.map_err(|e| bitcoin::io::Error::new(bitcoin::io::ErrorKind::Other, e))?
                    .kv_read(primary_namespace, secondary_namespace, key)
                    .await
                {
                    Ok(Some(bytes)) => Ok(bytes),
                    Ok(None) => Err(bitcoin::io::Error::new(
                        bitcoin::io::ErrorKind::NotFound,
                        "Key not found",
                    )),
                    Err(e) => Err(bitcoin::io::Error::new(
                        bitcoin::io::ErrorKind::Other,
                        e,
                    )),
                }
            })
        })
    }

    fn write(
        &self,
        primary_namespace: &str,
        secondary_namespace: &str,
        key: &str,
        buf: &[u8],
    ) -> Result<(), bitcoin::io::Error> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut tx = self
                    .inner
                    .begin_transaction()
                    .await
                    .map_err(|e| bitcoin::io::Error::new(bitcoin::io::ErrorKind::Other, e))?;
                tx.kv_write(primary_namespace, secondary_namespace, key, buf)
                    .await
                    .map_err(|e| bitcoin::io::Error::new(bitcoin::io::ErrorKind::Other, e))?;
                tx.commit()
                    .await
                    .map_err(|e| bitcoin::io::Error::new(bitcoin::io::ErrorKind::Other, e))
            })
        })
    }

    fn remove(
        &self,
        primary_namespace: &str,
        secondary_namespace: &str,
        key: &str,
        _lazy: bool,
    ) -> Result<(), bitcoin::io::Error> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut tx = self
                    .inner
                    .begin_transaction()
                    .await
                    .map_err(|e| bitcoin::io::Error::new(bitcoin::io::ErrorKind::Other, e))?;
                tx.kv_remove(primary_namespace, secondary_namespace, key)
                    .await
                    .map_err(|e| bitcoin::io::Error::new(bitcoin::io::ErrorKind::Other, e))?;
                tx.commit()
                    .await
                    .map_err(|e| bitcoin::io::Error::new(bitcoin::io::ErrorKind::Other, e))
            })
        })
    }

    fn list(
        &self,
        primary_namespace: &str,
        secondary_namespace: &str,
    ) -> Result<Vec<String>, bitcoin::io::Error> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.inner
                    .begin_transaction().await.map_err(|e| bitcoin::io::Error::new(bitcoin::io::ErrorKind::Other, e))?
                    .kv_list(primary_namespace, secondary_namespace)
                    .await
                    .map_err(|e| bitcoin::io::Error::new(bitcoin::io::ErrorKind::Other, e))
            })
        })
    }
}

impl<RM> SQLLdkDatabase<RM>
where
    RM: DatabasePool + 'static,
{
    /// Creates a new instance
    pub async fn new<X>(db: X, inner: Arc<dyn MintKVStore<Err = Error> + Send + Sync>) -> Result<Self, Error>
    where
        X: Into<RM::Config>,
    {
        let pool = Pool::new(db.into());

        Self::migrate(pool.get().map_err(|e| Error::Database(Box::new(e)))?).await?;

        Ok(Self { pool, inner })
    }

    /// Migrate
    async fn migrate(conn: PooledResource<RM>) -> Result<(), Error> {
        let tx = ConnectionWithTransaction::new(conn).await?;
        //migrate(&tx, RM::Connection::name(), crate::mint::ldk::MIGRATIONS).await?;
        tx.commit().await?;
        Ok(())
    }

    #[inline(always)]
    async fn fetch_from_config<R>(&self, id: &str) -> Result<R, Error>
    where
        R: serde::de::DeserializeOwned,
    {
        let conn = self.pool.get().map_err(|e| Error::Database(Box::new(e)))?;
        let value = column_as_string!(query(r#"SELECT value FROM config WHERE id = :id LIMIT 1"#)?
            .bind("id", id.to_owned())
            .pluck(&*conn)
            .await?
            .ok_or(Error::UnknownQuoteTTL)?);

        Ok(serde_json::from_str(&value)?)
    }
}
