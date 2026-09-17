//! Shared proof writes within a caller-owned SQL transaction.
use crate::database::DatabaseExecutor;
use crate::stmt::query;
use cdk_common::database::Error;
use cdk_common::wallet::ProofInfo;

pub(super) async fn write(
    conn: &impl DatabaseExecutor,
    proof: ProofInfo,
    insert_only: bool,
) -> Result<(), Error> {
    let sql = r#"
    INSERT INTO proof
    (y, mint_url, state, spending_condition, unit, amount, keyset_id, secret, c, witness, dleq_e, dleq_s, dleq_r, used_by_operation, created_by_operation, derivation_index, p2pk_e)
    VALUES
    (:y, :mint_url, :state, :spending_condition, :unit, :amount, :keyset_id, :secret, :c, :witness, :dleq_e, :dleq_s, :dleq_r, :used_by_operation, :created_by_operation, :derivation_index, :p2pk_e)
    "#;
    let conflict = r#"
    ON CONFLICT(y) DO UPDATE SET
        mint_url = excluded.mint_url,
        state = excluded.state,
        spending_condition = excluded.spending_condition,
        unit = excluded.unit,
        amount = excluded.amount,
        keyset_id = excluded.keyset_id,
        secret = excluded.secret,
        c = excluded.c,
        witness = excluded.witness,
        dleq_e = excluded.dleq_e,
        dleq_s = excluded.dleq_s,
        dleq_r = excluded.dleq_r,
        used_by_operation = excluded.used_by_operation,
        created_by_operation = excluded.created_by_operation,
        derivation_index = COALESCE(excluded.derivation_index, proof.derivation_index),
        p2pk_e = excluded.p2pk_e
    ;
            "#;
    let statement = if insert_only {
        sql.to_owned()
    } else {
        format!("{sql}{conflict}")
    };
    query(&statement)?
        .bind("y", proof.y.to_bytes().to_vec())
        .bind("mint_url", proof.mint_url.to_string())
        .bind("state", proof.state.to_string())
        .bind(
            "spending_condition",
            proof
                .spending_condition
                .map(|s| serde_json::to_string(&s).ok()),
        )
        .bind("unit", proof.unit.to_string())
        .bind("amount", u64::from(proof.proof.amount) as i64)
        .bind("keyset_id", proof.proof.keyset_id.to_string())
        .bind("secret", proof.proof.secret.to_string())
        .bind("c", proof.proof.c.to_bytes().to_vec())
        .bind(
            "witness",
            proof
                .proof
                .witness
                .and_then(|w| serde_json::to_string(&w).ok()),
        )
        .bind(
            "dleq_e",
            proof
                .proof
                .dleq
                .as_ref()
                .map(|dleq| dleq.e.to_secret_bytes().to_vec()),
        )
        .bind(
            "dleq_s",
            proof
                .proof
                .dleq
                .as_ref()
                .map(|dleq| dleq.s.to_secret_bytes().to_vec()),
        )
        .bind(
            "dleq_r",
            proof
                .proof
                .dleq
                .as_ref()
                .map(|dleq| dleq.r.to_secret_bytes().to_vec()),
        )
        .bind(
            "used_by_operation",
            proof.used_by_operation.map(|id| id.to_string()),
        )
        .bind(
            "created_by_operation",
            proof.created_by_operation.map(|id| id.to_string()),
        )
        .bind("derivation_index", proof.derivation_index.map(i64::from))
        .bind(
            "p2pk_e",
            proof.proof.p2pk_e.as_ref().map(|pk| pk.to_bytes().to_vec()),
        )
        .execute(conn)
        .await?;
    Ok(())
}
