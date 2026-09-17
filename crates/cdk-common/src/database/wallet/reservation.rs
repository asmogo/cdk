//! Validation shared by atomic reservation/import implementations.
use super::super::Error;
use crate::wallet::ProofInfo;
use crate::State;

/// Build a reserved input without replacing another operation's coin.
///
/// Backends must read `stored` and commit the returned record atomically. If no
/// record exists, they must insert without overwriting a concurrent import.
pub fn reserve_supplied_proof(
    supplied: &ProofInfo,
    stored: Option<ProofInfo>,
    operation_id: &uuid::Uuid,
) -> Result<ProofInfo, Error> {
    if supplied.y != supplied.proof.y()?
        || supplied.state != State::Reserved
        || supplied.used_by_operation != Some(*operation_id)
    {
        return Err(Error::ProofNotUnspent);
    }
    let Some(mut current) = stored else {
        return Ok(supplied.clone());
    };
    let mut comparable = supplied.proof.clone();
    comparable.witness = current.proof.witness.clone();
    comparable.dleq = current.proof.dleq.clone();
    if current.state != State::Unspent
        || current.used_by_operation.is_some()
        || current.y != supplied.y
        || current.mint_url != supplied.mint_url
        || current.unit != supplied.unit
        || current.proof != comparable
    {
        return Err(Error::ProofNotUnspent);
    }
    current.state = State::Reserved;
    current.used_by_operation = Some(*operation_id);
    current.proof.witness = supplied.proof.witness.clone();
    Ok(current)
}
