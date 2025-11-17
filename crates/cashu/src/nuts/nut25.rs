//! Bolt12
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::payment_method::{
    MeltQuoteMethodFields, MeltQuoteResponseFields, MintQuoteMethodFields, MintQuoteResponseFields,
};
use super::{MeltOptions, PublicKey};
use crate::Amount;

/// NUT18 Error
#[derive(Debug, Error)]
pub enum Error {
    /// Unknown Quote State
    #[error("Unknown quote state")]
    UnknownState,
    /// Amount overflow
    #[error("Amount Overflow")]
    AmountOverflow,
    /// Publickey not defined
    #[error("Publickey not defined")]
    PublickeyUndefined,
}

/// Bolt12 mint quote request
pub type MintQuoteBolt12Request = super::nut04::MintQuoteRequest<Bolt12MintRequestFields>;

/// Bolt12 mint quote response
pub type MintQuoteBolt12Response<Q> = super::nut04::MintQuoteResponse<Q, Bolt12MintResponseFields>;

/// Bolt12 melt quote request
pub type MeltQuoteBolt12Request = super::nut05::MeltQuoteRequest<Bolt12MeltRequestFields>;

// ============================================================================
// Generic Payment Method Field Implementations for Bolt12
// ============================================================================

/// Bolt12-specific fields for mint quote requests
///
/// These fields are flattened into the top-level JSON when used with
/// [`MintQuoteRequest<Bolt12MintRequestFields>`](super::nut04::MintQuoteRequest).
///
/// ## Fields
/// - `description`: Optional memo to include with the offer
/// - `pubkey`: Public key (REQUIRED for Bolt12 per NUT-24)
///
/// ## Example JSON
/// ```json
/// {
///   "amount": 1000,
///   "unit": "sat",
///   "description": "Payment for services",
///   "pubkey": "02..."
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Bolt12MintRequestFields {
    /// Memo to create the offer with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Pubkey (REQUIRED for Bolt12)
    pub pubkey: PublicKey,
}

impl MintQuoteMethodFields for Bolt12MintRequestFields {
    fn validate(&self) -> Result<(), String> {
        // Pubkey is required and enforced by the type system
        Ok(())
    }
}

/// Bolt12-specific fields for mint quote responses
///
/// These fields are flattened into the top-level JSON when used with
/// [`MintQuoteResponse<Q, Bolt12MintResponseFields>`](super::nut04::MintQuoteResponse).
///
/// ## Fields
/// - `pubkey`: Public key (echoed from request)
/// - `amount_paid`: Total amount that has been paid toward this quote
/// - `amount_issued`: Total amount that has been issued for this quote
///
/// ## Example JSON
/// ```json
/// {
///   "quote": "abc123",
///   "request": "lno1...",
///   "unit": "sat",
///   "state": "PAID",
///   "expiry": 1234567890,
///   "pubkey": "02...",
///   "amount_paid": 1000,
///   "amount_issued": 950
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Bolt12MintResponseFields {
    /// Pubkey (echoed from request)
    pub pubkey: PublicKey,
    /// Amount that has been paid toward this quote
    pub amount_paid: Amount,
    /// Amount that has been issued for this quote
    pub amount_issued: Amount,
}

impl MintQuoteResponseFields for Bolt12MintResponseFields {
    fn validate(&self) -> Result<(), String> {
        // Ensure issued amount doesn't exceed paid amount
        if self.amount_issued > self.amount_paid {
            return Err("Issued amount cannot exceed paid amount".to_string());
        }
        Ok(())
    }
}

/// Bolt12-specific fields for melt quote requests
///
/// These fields are flattened into the top-level JSON when used with
/// [`MeltQuoteRequest<Bolt12MeltRequestFields>`](super::nut05::MeltQuoteRequest).
///
/// ## Fields
/// - `options`: Optional payment options
///
/// ## Example JSON
/// ```json
/// {
///   "request": "lno1...",
///   "unit": "sat"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Bolt12MeltRequestFields {
    /// Payment Options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<MeltOptions>,
}

impl MeltQuoteMethodFields for Bolt12MeltRequestFields {
    fn validate(&self) -> Result<(), String> {
        // Validation is handled by the options enum itself
        Ok(())
    }
}

/// Bolt12-specific fields for melt quote responses
///
/// These fields are flattened into the top-level JSON when used with
/// [`MeltQuoteResponse<Q, Bolt12MeltResponseFields>`](super::nut05::MeltQuoteResponse).
///
/// ## Fields
/// - `fee_reserve`: Amount reserved to cover routing fees
/// - `payment_preimage`: Proof of payment (only present after successful payment)
///
/// ## Example JSON
/// ```json
/// {
///   "quote": "abc123",
///   "amount": 1000,
///   "unit": "sat",
///   "state": "PAID",
///   "expiry": 1234567890,
///   "fee_reserve": 50,
///   "payment_preimage": "abc123..."
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Bolt12MeltResponseFields {
    /// Fee reserve for the payment
    pub fee_reserve: Amount,
    /// Payment preimage (proof of payment)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_preimage: Option<String>,
}

impl MeltQuoteResponseFields for Bolt12MeltResponseFields {
    fn validate(&self) -> Result<(), String> {
        // No specific validation required for Bolt12 melt responses
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nuts::nut04::MintQuoteRequest;
    use crate::CurrencyUnit;

    #[test]
    fn test_bolt12_mint_request_fields_json_compat() {
        // Test that the generic type serializes correctly
        let amount = Amount::from(1000u64);
        let unit = CurrencyUnit::Sat;
        let description = Some("test".to_string());
        let pubkey = PublicKey::from_slice(&[
            2, 121, 190, 102, 126, 249, 220, 187, 172, 85, 160, 98, 149, 206, 135, 11, 7, 2, 155,
            252, 219, 45, 206, 40, 217, 89, 242, 129, 91, 22, 248, 23, 152,
        ])
        .unwrap();

        // Generic type (now MintQuoteBolt12Request is an alias to this)
        let request = MintQuoteRequest::new(
            amount,
            unit.clone(),
            Bolt12MintRequestFields {
                description: description.clone(),
                pubkey,
            },
        );

        // Verify JSON structure
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["amount"], 1000);
        assert_eq!(json["unit"], "sat");
        assert_eq!(json["description"], "test");
        assert!(json["pubkey"].is_string());
    }

    #[test]
    fn test_bolt12_field_traits() {
        let pubkey = PublicKey::from_slice(&[
            2, 121, 190, 102, 126, 249, 220, 187, 172, 85, 160, 98, 149, 206, 135, 11, 7, 2, 155,
            252, 219, 45, 206, 40, 217, 89, 242, 129, 91, 22, 248, 23, 152,
        ])
        .unwrap();

        let mint_request_fields = Bolt12MintRequestFields {
            description: Some("test".to_string()),
            pubkey,
        };
        assert!(MintQuoteMethodFields::validate(&mint_request_fields).is_ok());

        let mint_response_fields = Bolt12MintResponseFields {
            pubkey,
            amount_paid: Amount::from(1000u64),
            amount_issued: Amount::from(950u64),
        };
        assert!(MintQuoteResponseFields::validate(&mint_response_fields).is_ok());

        let melt_request_fields = Bolt12MeltRequestFields { options: None };
        assert!(MeltQuoteMethodFields::validate(&melt_request_fields).is_ok());

        let melt_response_fields = Bolt12MeltResponseFields {
            fee_reserve: Amount::from(50u64),
            payment_preimage: None,
        };
        assert!(MeltQuoteResponseFields::validate(&melt_response_fields).is_ok());
    }

    #[test]
    fn test_bolt12_validation() {
        let pubkey = PublicKey::from_slice(&[
            2, 121, 190, 102, 126, 249, 220, 187, 172, 85, 160, 98, 149, 206, 135, 11, 7, 2, 155,
            252, 219, 45, 206, 40, 217, 89, 242, 129, 91, 22, 248, 23, 152,
        ])
        .unwrap();

        // Test that issued cannot exceed paid
        let invalid_response = Bolt12MintResponseFields {
            pubkey,
            amount_paid: Amount::from(900u64),
            amount_issued: Amount::from(1000u64), // More than paid!
        };
        assert!(MintQuoteResponseFields::validate(&invalid_response).is_err());

        let valid_response = Bolt12MintResponseFields {
            pubkey,
            amount_paid: Amount::from(1000u64),
            amount_issued: Amount::from(950u64),
        };
        assert!(MintQuoteResponseFields::validate(&valid_response).is_ok());
    }
}
