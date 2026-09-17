//! Contract tests for atomic supplied-proof reservation.
use super::*;

/// Existing and external coins retain their origin and a valid reservation owner.
pub async fn reserve_supplied_preserves_metadata<DB>(db: DB)
where
    DB: Database<Error> + Sync,
{
    let operation = uuid::Uuid::new_v4();
    let mut existing = test_proof_info(test_keyset_id(), 64, test_mint_url());
    existing.derivation_index = Some(23);
    existing.created_by_operation = Some(uuid::Uuid::new_v4());
    db.update_proofs(vec![existing.clone()], vec![])
        .await
        .unwrap();
    let mut supplied = existing.clone();
    supplied.derivation_index = None;
    supplied.created_by_operation = None;
    supplied.state = State::Reserved;
    supplied.used_by_operation = Some(operation);
    supplied.proof.witness = Some(crate::Witness::P2PKWitness(crate::nut11::P2PKWitness {
        signatures: vec!["test-witness".to_owned()],
    }));
    let mut external = test_proof_info(test_keyset_id(), 32, test_mint_url());
    external.state = State::Reserved;
    external.used_by_operation = Some(operation);
    db.reserve_supplied_proofs(vec![supplied.clone(), external.clone()], &operation)
        .await
        .unwrap();
    existing.state = State::Reserved;
    existing.used_by_operation = Some(operation);
    existing.proof.witness = supplied.proof.witness;
    assert_eq!(
        db.get_proofs_by_ys(vec![existing.y]).await.unwrap(),
        vec![existing]
    );
    assert_eq!(
        db.get_proofs_by_ys(vec![external.y]).await.unwrap(),
        vec![external]
    );
}

/// A reservation acquired after a caller's read cannot be replaced by import.
pub async fn reserve_supplied_stale_read<DB>(db: DB)
where
    DB: Database<Error> + Sync,
{
    let existing = test_proof_info(test_keyset_id(), 64, test_mint_url());
    db.update_proofs(vec![existing.clone()], vec![])
        .await
        .unwrap();
    let mut supplied = db
        .get_proofs_by_ys(vec![existing.y])
        .await
        .unwrap()
        .remove(0);
    let owner = uuid::Uuid::new_v4();
    db.reserve_proofs(vec![existing.y], &owner).await.unwrap();
    let competitor = uuid::Uuid::new_v4();
    supplied.state = State::Reserved;
    supplied.used_by_operation = Some(competitor);
    assert!(db
        .reserve_supplied_proofs(vec![supplied], &competitor)
        .await
        .is_err());
    let mut expected = existing;
    expected.state = State::Reserved;
    expected.used_by_operation = Some(owner);
    assert_eq!(
        db.get_proofs_by_ys(vec![expected.y]).await.unwrap(),
        vec![expected]
    );
}

/// An invalid or unavailable last input rolls back earlier imports/reservations.
pub async fn reserve_supplied_rolls_back_batch<DB>(db: DB)
where
    DB: Database<Error> + Sync,
{
    let operation = uuid::Uuid::new_v4();
    let mut inputs = vec![
        test_proof_info(test_keyset_id(), 64, test_mint_url()),
        test_proof_info(test_keyset_id(), 32, test_mint_url()),
    ];
    inputs.sort_by_key(|p| p.y);
    let mut unavailable = inputs[1].clone();
    unavailable.state = State::Spent;
    db.update_proofs(vec![unavailable.clone()], vec![])
        .await
        .unwrap();
    for input in &mut inputs {
        input.state = State::Reserved;
        input.used_by_operation = Some(operation);
    }
    assert!(db
        .reserve_supplied_proofs(inputs.clone(), &operation)
        .await
        .is_err());
    assert!(db
        .get_proofs_by_ys(vec![inputs[0].y])
        .await
        .unwrap()
        .is_empty());
    assert_eq!(
        db.get_proofs_by_ys(vec![inputs[1].y]).await.unwrap(),
        vec![unavailable]
    );
    assert!(db
        .reserve_supplied_proofs(vec![inputs[0].clone(), inputs[0].clone()], &operation)
        .await
        .is_err());
    assert!(db
        .get_proofs_by_ys(vec![inputs[0].y])
        .await
        .unwrap()
        .is_empty());
}

/// Ordinary reservation and supplied import race without ever sharing a coin.
pub async fn reserve_supplied_concurrent<DB>(db: DB)
where
    DB: Database<Error> + Sync,
{
    let existing = test_proof_info(test_keyset_id(), 64, test_mint_url());
    db.update_proofs(vec![existing.clone()], vec![])
        .await
        .unwrap();
    let first = uuid::Uuid::new_v4();
    let second = uuid::Uuid::new_v4();
    let mut supplied = existing.clone();
    supplied.state = State::Reserved;
    supplied.used_by_operation = Some(second);
    let (a, b) = tokio::join!(
        db.reserve_proofs(vec![existing.y], &first),
        db.reserve_supplied_proofs(vec![supplied], &second)
    );
    assert_ne!(a.is_ok(), b.is_ok());
    let winner = if a.is_ok() { first } else { second };
    assert_eq!(
        db.get_proofs_by_ys(vec![existing.y]).await.unwrap()[0].used_by_operation,
        Some(winner)
    );

    let mut fresh = test_proof_info(test_keyset_id(), 32, test_mint_url());
    fresh.state = State::Reserved;
    fresh.used_by_operation = Some(first);
    let mut competing = fresh.clone();
    competing.used_by_operation = Some(second);
    let (a, b) = tokio::join!(
        db.reserve_supplied_proofs(vec![fresh.clone()], &first),
        db.reserve_supplied_proofs(vec![competing], &second)
    );
    assert_ne!(a.is_ok(), b.is_ok());
    let winner = if a.is_ok() { first } else { second };
    assert_eq!(
        db.get_proofs_by_ys(vec![fresh.y]).await.unwrap()[0].used_by_operation,
        Some(winner)
    );
}

/// Changing a supplied coin's bearer content or wallet scope cannot replace it.
pub async fn reserve_supplied_rejects_mismatch<DB>(db: DB)
where
    DB: Database<Error> + Sync,
{
    let existing = test_proof_info(test_keyset_id(), 64, test_mint_url());
    db.update_proofs(vec![existing.clone()], vec![])
        .await
        .unwrap();
    let operation = uuid::Uuid::new_v4();
    for mismatch in 0..3 {
        let mut supplied = existing.clone();
        supplied.state = State::Reserved;
        supplied.used_by_operation = Some(operation);
        match mismatch {
            0 => supplied.mint_url = test_mint_url_2(),
            1 => supplied.proof.amount += Amount::from(1),
            _ => supplied.proof.c = SecretKey::generate().public_key(),
        }
        assert!(db
            .reserve_supplied_proofs(vec![supplied], &operation)
            .await
            .is_err());
        assert_eq!(
            db.get_proofs_by_ys(vec![existing.y]).await.unwrap(),
            vec![existing.clone()]
        );
    }
}
