use super::*;
use crate::wallet::test_utils::{
    create_test_db, create_test_wallet_with_mock, test_keyset_id, test_melt_quote, test_mint_url,
    test_proof_info, MockMintConnector,
};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn supplied_proofs_cannot_steal_another_reservation() {
    let db = create_test_db().await;
    let quote = test_melt_quote();
    db.add_melt_quote(quote.clone()).await.unwrap();
    let mut proof = test_proof_info(test_keyset_id(), 2048, test_mint_url());
    proof.state = State::Reserved;
    proof.used_by_operation = Some(Uuid::new_v4());
    db.update_proofs(vec![proof.clone()], vec![]).await.unwrap();
    let wallet = create_test_wallet_with_mock(db.clone(), Arc::new(MockMintConnector::new())).await;
    let result = wallet
        .prepare_melt_proofs(&quote.id, vec![proof.proof.clone()], HashMap::new())
        .await;
    assert!(
        result.is_err(),
        "preparation must reject an already-owned proof"
    );
    assert_eq!(
        db.get_proofs_by_ys(vec![proof.y]).await.unwrap(),
        vec![proof]
    );
    assert!(db
        .get_melt_quote(&quote.id)
        .await
        .unwrap()
        .unwrap()
        .used_by_operation
        .is_none());
}

#[tokio::test]
async fn supplied_external_proof_can_be_prepared_and_cancelled() {
    let db = create_test_db().await;
    let quote = test_melt_quote();
    db.add_melt_quote(quote.clone()).await.unwrap();
    let proof = test_proof_info(test_keyset_id(), 2048, test_mint_url());
    let wallet = create_test_wallet_with_mock(db.clone(), Arc::new(MockMintConnector::new())).await;
    let prepared = wallet
        .prepare_melt_proofs(&quote.id, vec![proof.proof.clone()], HashMap::new())
        .await
        .unwrap();
    let operation = prepared.operation_id();
    let reserved = db.get_proofs_by_ys(vec![proof.y]).await.unwrap().remove(0);
    assert_eq!(reserved.state, State::Reserved);
    assert_eq!(reserved.used_by_operation, Some(operation));
    assert_eq!(reserved.proof, proof.proof);
    prepared.cancel().await.unwrap();
    let released = db.get_proofs_by_ys(vec![proof.y]).await.unwrap().remove(0);
    assert_eq!(released.state, State::Unspent);
    assert!(released.used_by_operation.is_none());
    assert!(db
        .get_melt_quote(&quote.id)
        .await
        .unwrap()
        .unwrap()
        .used_by_operation
        .is_none());
    assert!(db.get_saga(&operation).await.unwrap().is_none());
}
