//! Atomic reservation of supplied encrypted coins.
use super::*;
use cdk_common::database::wallet::reserve_supplied_proof;
use std::collections::HashSet;

impl SupabaseWalletDatabase {
    pub(super) async fn reserve_supplied_atomic(
        &self,
        inputs: Vec<ProofInfo>,
        operation_id: &uuid::Uuid,
    ) -> Result<(), DatabaseError> {
        if inputs.is_empty() {
            return Ok(());
        }
        let mut unique = HashSet::new();
        for input in &inputs {
            if !unique.insert(input.y) {
                return Err(DatabaseError::ProofNotUnspent);
            }
        }
        let ys = inputs
            .iter()
            .map(|p| hex::encode(p.y.to_bytes()))
            .collect::<Vec<_>>();
        let (status, text) = self
            .get_request(&format!("rest/v1/proof?y=in.({})", ys.join(",")))
            .await?;
        if !status.is_success() {
            return Err(DatabaseError::Internal(format!(
                "reserve_supplied_proofs: read failed: HTTP {status}"
            )));
        }
        let mut stored: HashMap<_, _> = Self::parse_response::<ProofTable>(&text)?
            .unwrap_or_default()
            .into_iter()
            .map(|p| (p.y.clone(), p))
            .collect();
        let mut transitions = Vec::with_capacity(inputs.len());
        for input in inputs {
            let (before, current) = match stored.remove(&hex::encode(input.y.to_bytes())) {
                Some(table) => {
                    let expected = serde_json::to_value(&table)?;
                    let mut decoded = table;
                    self.decrypt_proof_table(&mut decoded).await;
                    (expected, Some(decoded.try_into()?))
                }
                None => (serde_json::Value::Null, None),
            };
            let reserved = reserve_supplied_proof(&input, current, operation_id)?;
            let mut after: ProofTable = reserved.try_into()?;
            after.secret = hex::encode(self.encrypt(after.secret.as_bytes()).await?);
            let c_bytes =
                hex::decode(&after.c).map_err(|e| DatabaseError::Internal(e.to_string()))?;
            after.c = hex::encode(self.encrypt(&c_bytes).await?);
            transitions.push(serde_json::json!({"before": before, "after": after}));
        }
        let params = serde_json::json!({"p_proofs": transitions, "p_operation_id": operation_id.to_string()});
        self.call_rpc("reserve_supplied_proofs_atomic", &params.to_string())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;
    use serde_json::json;

    fn proof() -> ProofInfo {
        ProofInfo::new(
            cdk_common::Proof {
                amount: 64.into(),
                keyset_id: "00916bbf7ef91a36".parse().unwrap(),
                secret: Secret::generate(),
                c: cdk_common::SecretKey::generate().public_key(),
                witness: None,
                dleq: None,
                p2pk_e: None,
            },
            "https://mint.example".parse().unwrap(),
            State::Unspent,
            CurrencyUnit::Sat,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn reserve_supplied_sends_expected_ciphertext_in_single_rpc() {
        let mut server = mockito::Server::new_async().await;
        let db =
            SupabaseWalletDatabase::new(Url::parse(&server.url()).unwrap(), "test-key".to_owned())
                .await
                .unwrap();
        *db.encryption_key.write().await = Some([7u8; 32].into());
        let original = proof();
        let operation = uuid::Uuid::new_v4();
        let mut encrypted: ProofTable = original.clone().try_into().unwrap();
        encrypted.secret = hex::encode(db.encrypt(encrypted.secret.as_bytes()).await.unwrap());
        encrypted.c = hex::encode(
            db.encrypt(&hex::decode(&encrypted.c).unwrap())
                .await
                .unwrap(),
        );
        let expected = serde_json::to_value(encrypted).unwrap();
        let read = server
            .mock("GET", "/rest/v1/proof")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body(json!([expected.clone()]).to_string())
            .create_async()
            .await;
        let rpc = server.mock("POST", "/rest/v1/rpc/reserve_supplied_proofs_atomic")
            .match_body(Matcher::PartialJson(json!({"p_operation_id":operation.to_string(),"p_proofs":[{"before":expected,"after":{"state":"RESERVED","used_by_operation":operation.to_string()}}]})))
            .with_status(204).create_async().await;
        let mut supplied = original;
        supplied.state = State::Reserved;
        supplied.used_by_operation = Some(operation);
        db.reserve_supplied_proofs(vec![supplied], &operation)
            .await
            .unwrap();
        read.assert_async().await;
        rpc.assert_async().await;
    }

    #[tokio::test]
    async fn reserve_supplied_rpc_conflict_has_no_upsert_fallback() {
        let mut server = mockito::Server::new_async().await;
        let db =
            SupabaseWalletDatabase::new(Url::parse(&server.url()).unwrap(), "test-key".to_owned())
                .await
                .unwrap();
        *db.encryption_key.write().await = Some([7u8; 32].into());
        let operation = uuid::Uuid::new_v4();
        let mut supplied = proof();
        supplied.state = State::Reserved;
        supplied.used_by_operation = Some(operation);
        let read = server
            .mock("GET", "/rest/v1/proof")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body("[]")
            .create_async()
            .await;
        let rpc = server
            .mock("POST", "/rest/v1/rpc/reserve_supplied_proofs_atomic")
            .expect(1)
            .with_status(409)
            .with_body("reservation conflict")
            .create_async()
            .await;
        let fallback = server
            .mock("POST", "/rest/v1/rpc/update_proofs_atomic")
            .expect(0)
            .create_async()
            .await;
        assert!(db
            .reserve_supplied_proofs(vec![supplied], &operation)
            .await
            .is_err());
        read.assert_async().await;
        rpc.assert_async().await;
        fallback.assert_async().await;
    }
}
