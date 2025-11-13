//! Bolt11

use std::fmt;
use std::str::FromStr;

use lightning_invoice::Bolt11Invoice;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use thiserror::Error;

use super::payment_method::{
    MeltQuoteMethodFields, MeltQuoteResponseFields, MintQuoteMethodFields,
    MintQuoteResponseFields,
};
use super::{BlindSignature, CurrencyUnit, MeltQuoteState, Mpp, PublicKey};
#[cfg(feature = "mint")]
use crate::quote_id::QuoteId;
use crate::Amount;

/// NUT023 Error
#[derive(Debug, Error)]
pub enum Error {
    /// Unknown Quote State
    #[error("Unknown Quote State")]
    UnknownState,
    /// Amount overflow
    #[error("Amount overflow")]
    AmountOverflow,
    /// Invalid Amount
    #[error("Invalid Request")]
    InvalidAmountRequest,
}

/// Mint quote request [NUT-04]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct MintQuoteBolt11Request {
    /// Amount
    pub amount: Amount,
    /// Unit wallet would like to pay with
    pub unit: CurrencyUnit,
    /// Memo to create the invoice with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// NUT-19 Pubkey
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey: Option<PublicKey>,
}

/// Possible states of a quote
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema), schema(as = MintQuoteState))]
pub enum QuoteState {
    /// Quote has not been paid
    #[default]
    Unpaid,
    /// Quote has been paid and wallet can mint
    Paid,
    /// ecash issued for quote
    Issued,
}

impl fmt::Display for QuoteState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unpaid => write!(f, "UNPAID"),
            Self::Paid => write!(f, "PAID"),
            Self::Issued => write!(f, "ISSUED"),
        }
    }
}

impl FromStr for QuoteState {
    type Err = Error;

    fn from_str(state: &str) -> Result<Self, Self::Err> {
        match state {
            "PAID" => Ok(Self::Paid),
            "UNPAID" => Ok(Self::Unpaid),
            "ISSUED" => Ok(Self::Issued),
            _ => Err(Error::UnknownState),
        }
    }
}

/// Mint quote response [NUT-04]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[serde(bound = "Q: Serialize + DeserializeOwned")]
pub struct MintQuoteBolt11Response<Q> {
    /// Quote Id
    pub quote: Q,
    /// Payment request to fulfil
    pub request: String,
    /// Amount
    // REVIEW: This is now required in the spec, we should remove the option once all mints update
    pub amount: Option<Amount>,
    /// Unit
    // REVIEW: This is now required in the spec, we should remove the option once all mints update
    pub unit: Option<CurrencyUnit>,
    /// Quote State
    pub state: QuoteState,
    /// Unix timestamp until the quote is valid
    pub expiry: Option<u64>,
    /// NUT-19 Pubkey
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey: Option<PublicKey>,
}
impl<Q: ToString> MintQuoteBolt11Response<Q> {
    /// Convert the MintQuote with a quote type Q to a String
    pub fn to_string_id(&self) -> MintQuoteBolt11Response<String> {
        MintQuoteBolt11Response {
            quote: self.quote.to_string(),
            request: self.request.clone(),
            state: self.state,
            expiry: self.expiry,
            pubkey: self.pubkey,
            amount: self.amount,
            unit: self.unit.clone(),
        }
    }
}

#[cfg(feature = "mint")]
impl From<MintQuoteBolt11Response<QuoteId>> for MintQuoteBolt11Response<String> {
    fn from(value: MintQuoteBolt11Response<QuoteId>) -> Self {
        Self {
            quote: value.quote.to_string(),
            request: value.request,
            state: value.state,
            expiry: value.expiry,
            pubkey: value.pubkey,
            amount: value.amount,
            unit: value.unit.clone(),
        }
    }
}

/// BOLT11 melt quote request [NUT-23]
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct MeltQuoteBolt11Request {
    /// Bolt11 invoice to be paid
    #[cfg_attr(feature = "swagger", schema(value_type = String))]
    pub request: Bolt11Invoice,
    /// Unit wallet would like to pay with
    pub unit: CurrencyUnit,
    /// Payment Options
    pub options: Option<MeltOptions>,
}

/// Melt Options
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub enum MeltOptions {
    /// Mpp Options
    Mpp {
        /// MPP
        mpp: Mpp,
    },
    /// Amountless options
    Amountless {
        /// Amountless
        amountless: Amountless,
    },
}

impl MeltOptions {
    /// Create new [`MeltOptions::Mpp`]
    pub fn new_mpp<A>(amount: A) -> Self
    where
        A: Into<Amount>,
    {
        Self::Mpp {
            mpp: Mpp {
                amount: amount.into(),
            },
        }
    }

    /// Create new [`MeltOptions::Amountless`]
    pub fn new_amountless<A>(amount_msat: A) -> Self
    where
        A: Into<Amount>,
    {
        Self::Amountless {
            amountless: Amountless {
                amount_msat: amount_msat.into(),
            },
        }
    }

    /// Payment amount
    pub fn amount_msat(&self) -> Amount {
        match self {
            Self::Mpp { mpp } => mpp.amount,
            Self::Amountless { amountless } => amountless.amount_msat,
        }
    }
}

/// Amountless payment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Amountless {
    /// Amount to pay in msat
    pub amount_msat: Amount,
}

impl MeltQuoteBolt11Request {
    /// Amount from [`MeltQuoteBolt11Request`]
    ///
    /// Amount can either be defined in the bolt11 invoice,
    /// in the request for an amountless bolt11 or in MPP option.
    pub fn amount_msat(&self) -> Result<Amount, Error> {
        let MeltQuoteBolt11Request {
            request,
            unit: _,
            options,
            ..
        } = self;

        match options {
            None => Ok(request
                .amount_milli_satoshis()
                .ok_or(Error::InvalidAmountRequest)?
                .into()),
            Some(MeltOptions::Mpp { mpp }) => Ok(mpp.amount),
            Some(MeltOptions::Amountless { amountless }) => {
                let amount = amountless.amount_msat;
                if let Some(amount_msat) = request.amount_milli_satoshis() {
                    if amount != amount_msat.into() {
                        return Err(Error::InvalidAmountRequest);
                    }
                }
                Ok(amount)
            }
        }
    }
}

/// Melt quote response [NUT-05]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
#[serde(bound = "Q: Serialize")]
pub struct MeltQuoteBolt11Response<Q> {
    /// Quote Id
    pub quote: Q,
    /// The amount that needs to be provided
    pub amount: Amount,
    /// The fee reserve that is required
    pub fee_reserve: Amount,
    /// Whether the request haas be paid
    // TODO: To be deprecated
    /// Deprecated
    pub paid: Option<bool>,
    /// Quote State
    pub state: MeltQuoteState,
    /// Unix timestamp until the quote is valid
    pub expiry: u64,
    /// Payment preimage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_preimage: Option<String>,
    /// Change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<Vec<BlindSignature>>,
    /// Payment request to fulfill
    // REVIEW: This is now required in the spec, we should remove the option once all mints update
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<String>,
    /// Unit
    // REVIEW: This is now required in the spec, we should remove the option once all mints update
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<CurrencyUnit>,
}

impl<Q: ToString> MeltQuoteBolt11Response<Q> {
    /// Convert a `MeltQuoteBolt11Response` with type Q (generic/unknown) to a
    /// `MeltQuoteBolt11Response` with `String`
    pub fn to_string_id(self) -> MeltQuoteBolt11Response<String> {
        MeltQuoteBolt11Response {
            quote: self.quote.to_string(),
            amount: self.amount,
            fee_reserve: self.fee_reserve,
            paid: self.paid,
            state: self.state,
            expiry: self.expiry,
            payment_preimage: self.payment_preimage,
            change: self.change,
            request: self.request,
            unit: self.unit,
        }
    }
}

#[cfg(feature = "mint")]
impl From<MeltQuoteBolt11Response<QuoteId>> for MeltQuoteBolt11Response<String> {
    fn from(value: MeltQuoteBolt11Response<QuoteId>) -> Self {
        Self {
            quote: value.quote.to_string(),
            amount: value.amount,
            fee_reserve: value.fee_reserve,
            paid: value.paid,
            state: value.state,
            expiry: value.expiry,
            payment_preimage: value.payment_preimage,
            change: value.change,
            request: value.request,
            unit: value.unit,
        }
    }
}

// A custom deserializer is needed until all mints
// update some will return without the required state.
impl<'de, Q: DeserializeOwned> Deserialize<'de> for MeltQuoteBolt11Response<Q> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let quote: Q = serde_json::from_value(
            value
                .get("quote")
                .ok_or(serde::de::Error::missing_field("quote"))?
                .clone(),
        )
        .map_err(|_| serde::de::Error::custom("Invalid quote if string"))?;

        let amount = value
            .get("amount")
            .ok_or(serde::de::Error::missing_field("amount"))?
            .as_u64()
            .ok_or(serde::de::Error::missing_field("amount"))?;
        let amount = Amount::from(amount);

        let fee_reserve = value
            .get("fee_reserve")
            .ok_or(serde::de::Error::missing_field("fee_reserve"))?
            .as_u64()
            .ok_or(serde::de::Error::missing_field("fee_reserve"))?;

        let fee_reserve = Amount::from(fee_reserve);

        let paid: Option<bool> = value.get("paid").and_then(|p| p.as_bool());

        let state: Option<String> = value
            .get("state")
            .and_then(|s| serde_json::from_value(s.clone()).ok());

        let (state, paid) = match (state, paid) {
            (None, None) => return Err(serde::de::Error::custom("State or paid must be defined")),
            (Some(state), _) => {
                let state: MeltQuoteState = MeltQuoteState::from_str(&state)
                    .map_err(|_| serde::de::Error::custom("Unknown state"))?;
                let paid = state == MeltQuoteState::Paid;

                (state, paid)
            }
            (None, Some(paid)) => {
                let state = if paid {
                    MeltQuoteState::Paid
                } else {
                    MeltQuoteState::Unpaid
                };
                (state, paid)
            }
        };

        let expiry = value
            .get("expiry")
            .ok_or(serde::de::Error::missing_field("expiry"))?
            .as_u64()
            .ok_or(serde::de::Error::missing_field("expiry"))?;

        let payment_preimage: Option<String> = value
            .get("payment_preimage")
            .and_then(|p| serde_json::from_value(p.clone()).ok());

        let change: Option<Vec<BlindSignature>> = value
            .get("change")
            .and_then(|b| serde_json::from_value(b.clone()).ok());

        let request: Option<String> = value
            .get("request")
            .and_then(|r| serde_json::from_value(r.clone()).ok());

        let unit: Option<CurrencyUnit> = value
            .get("unit")
            .and_then(|u| serde_json::from_value(u.clone()).ok());

        Ok(Self {
            quote,
            amount,
            fee_reserve,
            paid: Some(paid),
            state,
            expiry,
            payment_preimage,
            change,
            request,
            unit,
        })
    }
}

// ============================================================================
// Generic Payment Method Field Implementations for Bolt11
// ============================================================================

/// Bolt11-specific fields for mint quote requests
///
/// These fields are flattened into the top-level JSON when used with
/// [`MintQuoteRequest<Bolt11MintRequestFields>`](super::nut04::MintQuoteRequest).
///
/// ## Fields
/// - `description`: Optional memo to include in the Lightning invoice
/// - `pubkey`: Optional NUT-19 public key for authentication
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
pub struct Bolt11MintRequestFields {
    /// Memo to create the invoice with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// NUT-19 Pubkey for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey: Option<PublicKey>,
}

impl MintQuoteMethodFields for Bolt11MintRequestFields {
    fn validate(&self) -> Result<(), String> {
        // No specific validation required for Bolt11 mint requests
        Ok(())
    }
}

/// Bolt11-specific fields for mint quote responses
///
/// These fields are flattened into the top-level JSON when used with
/// [`MintQuoteResponse<Q, Bolt11MintResponseFields>`](super::nut04::MintQuoteResponse).
///
/// ## Fields
/// - `pubkey`: Optional NUT-19 public key (echoed from request if provided)
///
/// ## Example JSON
/// ```json
/// {
///   "quote": "abc123",
///   "request": "lnbc10u1...",
///   "unit": "sat",
///   "state": "UNPAID",
///   "expiry": 1234567890,
///   "pubkey": "02..."
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Bolt11MintResponseFields {
    /// NUT-19 Pubkey (echoed from request)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pubkey: Option<PublicKey>,
}

impl MintQuoteResponseFields for Bolt11MintResponseFields {
    fn validate(&self) -> Result<(), String> {
        // No specific validation required for Bolt11 mint responses
        Ok(())
    }
}

/// Bolt11-specific fields for melt quote requests
///
/// These fields are flattened into the top-level JSON when used with
/// [`MeltQuoteRequest<Bolt11MeltRequestFields>`](super::nut05::MeltQuoteRequest).
///
/// ## Fields
/// - `options`: Optional payment options (MPP or Amountless)
///
/// ## Example JSON (with MPP)
/// ```json
/// {
///   "request": "lnbc10u1...",
///   "unit": "sat",
///   "mpp": {
///     "amount": 1000
///   }
/// }
/// ```
///
/// ## Example JSON (with Amountless)
/// ```json
/// {
///   "request": "lnbc1...",
///   "unit": "sat",
///   "amountless": {
///     "amount_msat": 1000000
///   }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Bolt11MeltRequestFields {
    /// Payment Options (MPP or Amountless)
    #[serde(flatten)]
    pub options: Option<MeltOptions>,
}

impl MeltQuoteMethodFields for Bolt11MeltRequestFields {
    fn validate(&self) -> Result<(), String> {
        // Validation is handled by the options enum itself
        Ok(())
    }
}

/// Bolt11-specific fields for melt quote responses
///
/// These fields are flattened into the top-level JSON when used with
/// [`MeltQuoteResponse<Q, Bolt11MeltResponseFields>`](super::nut05::MeltQuoteResponse).
///
/// ## Fields
/// - `fee_reserve`: Amount reserved to cover Lightning routing fees
/// - `payment_preimage`: Proof of payment (only present after successful payment)
/// - `change`: Blind signatures for overpayment change (only if applicable)
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
pub struct Bolt11MeltResponseFields {
    /// Fee reserve for the Lightning payment
    pub fee_reserve: Amount,
    /// Payment preimage (proof of payment)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_preimage: Option<String>,
    /// Change proofs from overpayment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<Vec<BlindSignature>>,
}

impl MeltQuoteResponseFields for Bolt11MeltResponseFields {
    fn validate(&self) -> Result<(), String> {
        // No specific validation required for Bolt11 melt responses
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nuts::nut04::MintQuoteRequest;
    use crate::nuts::nut05::MeltQuoteRequest;
    use crate::nuts::nut23::QuoteState as MintQuoteState;

    #[test]
    fn test_bolt11_mint_request_fields_json_compat() {
        // Test that the new generic type produces the same JSON as the old type
        let amount = Amount::from(1000u64);
        let unit = CurrencyUnit::Sat;
        let description = Some("test".to_string());
        let pubkey = None;

        // Old type
        let old_request = MintQuoteBolt11Request {
            amount,
            unit: unit.clone(),
            description: description.clone(),
            pubkey,
        };

        // New generic type
        let new_request = MintQuoteRequest::new(
            amount,
            unit,
            Bolt11MintRequestFields { description, pubkey },
        );

        // Both should serialize to identical JSON
        let old_json = serde_json::to_value(&old_request).unwrap();
        let new_json = serde_json::to_value(&new_request).unwrap();
        
        assert_eq!(old_json, new_json, "JSON serialization should be identical");
    }

    #[test]
    fn test_bolt11_melt_request_fields_serialization() {
        // Test that Bolt11MeltRequestFields serializes correctly
        let request_str = "lnbc100n1pnvpufspp5djn8hrq49r8cghwye9kqw752qjncwyfnrprhprpqk43mwcy4yfsqdq5g9kxy7fqd9h8vmmfvdjscqzzsxqyz5vqsp5uhpjt36rj75pl7jq2sshaukzfkt7uulj456s4mh7uy7l6vx7lvxs9qxpqysgqedwz08acmqwtk8g4vkwm2w78suwt2qyzz6jkkwcgrjm3r3hs6fskyhvud4fan3keru7emjm8ygqpcrwtlmhfjfmer3afs5hhwamgr4cqtactdq".to_string();
        
        let mpp_options = Some(MeltOptions::Mpp {
            mpp: Mpp {
                amount: Amount::from(1000u64),
            },
        });
        
        // New generic type with MPP
        let new_request = MeltQuoteRequest::new(
            request_str.clone(),
            CurrencyUnit::Sat,
            Bolt11MeltRequestFields {
                options: mpp_options,
            },
        );

        let json = serde_json::to_value(&new_request).unwrap();
        
        // Verify all required fields are present
        assert_eq!(json["unit"], "sat");
        assert_eq!(json["request"], request_str);
        // With flatten, the MPP fields are at top level
        assert_eq!(json["mpp"]["amount"], 1000);
        
        // Test deserialization round-trip
        let deserialized: MeltQuoteRequest<Bolt11MeltRequestFields> =
            serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.unit, CurrencyUnit::Sat);
        assert_eq!(deserialized.request, request_str);
    }

    #[test]
    fn test_bolt11_field_traits() {
        // Test that field types implement the required traits
        let mint_request_fields = Bolt11MintRequestFields {
            description: Some("test".to_string()),
            pubkey: None,
        };
        assert!(MintQuoteMethodFields::validate(&mint_request_fields).is_ok());

        let mint_response_fields = Bolt11MintResponseFields { pubkey: None };
        assert!(MintQuoteResponseFields::validate(&mint_response_fields).is_ok());

        let melt_request_fields = Bolt11MeltRequestFields { options: None };
        assert!(MeltQuoteMethodFields::validate(&melt_request_fields).is_ok());

        let melt_response_fields = Bolt11MeltResponseFields {
            fee_reserve: Amount::from(50u64),
            payment_preimage: None,
            change: None,
        };
        assert!(MeltQuoteResponseFields::validate(&melt_response_fields).is_ok());
    }
}
